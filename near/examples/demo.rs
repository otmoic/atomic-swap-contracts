#![allow(unused)]
//! Demo client on testnet

use std::io::{self, Write};

#[cfg(feature = "demo")]
use std::time::{Duration, SystemTime};

#[cfg(feature = "demo")]
use base64::decode;
#[cfg(feature = "demo")]
use near_crypto::InMemorySigner;
#[cfg(feature = "demo")]
use near_jsonrpc_client::{methods, JsonRpcClient};
#[cfg(feature = "demo")]
use near_jsonrpc_primitives::types::{query::QueryResponseKind, transactions::TransactionInfo};
#[cfg(feature = "demo")]
use near_primitives::{
    borsh::BorshDeserialize,
    hash::CryptoHash,
    transaction::{Action, FunctionCallAction, Transaction},
    types::{BlockReference, Finality, FunctionArgs},
    views::{
        FinalExecutionOutcomeView, FinalExecutionOutcomeViewEnum, FinalExecutionStatus,
        QueryRequest,
    },
};
#[cfg(feature = "demo")]
use serde_json::json;
#[cfg(feature = "demo")]
use tokio::time;

#[cfg(not(feature = "demo"))]
fn main() {
    println!("Please run this demo app with demo feature");
}

#[cfg(feature = "demo")]
async fn get_hash_and_nonce(
    client: &JsonRpcClient,
    signer: &InMemorySigner,
) -> Result<(CryptoHash, u64), Box<dyn std::error::Error>> {
    let access_key_query_response = client
        .call(methods::query::RpcQueryRequest {
            block_reference: BlockReference::latest(),
            request: near_primitives::views::QueryRequest::ViewAccessKey {
                account_id: signer.account_id.clone(),
                public_key: signer.public_key.clone(),
            },
        })
        .await?;

    match access_key_query_response.kind {
        QueryResponseKind::AccessKey(access_key) => {
            Ok((access_key_query_response.block_hash, access_key.nonce))
        }
        _ => Err("failed to extract current nonce")?,
    }
}

#[cfg(feature = "demo")]
async fn call_and_wait_result(
    client: &JsonRpcClient,
    signer: &InMemorySigner,
    block_hash: CryptoHash,
    nonce: u64,
    action: FunctionCallAction,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let transaction = Transaction {
        signer_id: signer.account_id.clone(),
        public_key: signer.public_key.clone(),
        nonce,
        receiver_id: "near-swap.yanganto.testnet".parse()?,
        block_hash,
        actions: vec![Action::FunctionCall(action)],
    };

    let request = methods::broadcast_tx_async::RpcBroadcastTxAsyncRequest {
        signed_transaction: transaction.sign(signer),
    };

    let sent_at = time::Instant::now();
    let tx_hash = client.call(request).await?;

    loop {
        let response = client
            .call(methods::tx::RpcTransactionStatusRequest {
                transaction_info: TransactionInfo::TransactionId {
                    hash: tx_hash,
                    account_id: signer.account_id.clone(),
                },
            })
            .await;
        let received_at = time::Instant::now();
        let delta = (received_at - sent_at).as_secs();

        if delta > 120 {
            return Err("time limit exceeded for the transaction to be recognized")?;
        }

        match response {
            Err(err) => match err.handler_error() {
                Some(methods::tx::RpcTransactionError::UnknownTransaction { .. }) => {
                    time::sleep(time::Duration::from_secs(2)).await;
                    continue;
                }
                _ => Err(err)?,
            },
            Ok(FinalExecutionOutcomeView { status, .. }) => {
                return match status {
                    FinalExecutionStatus::SuccessValue(s) => {
                        if let Ok(b) = decode(&s) {
                            Ok(b)
                        } else {
                            Err(format!("Fail to decode {s:}").into())
                        }
                    }
                    _ => Err(format!("Response unexpected: {status:?}").into()),
                }
            }
        }
    }
}

#[cfg(feature = "demo")]
async fn check_transfer(
    client: &JsonRpcClient,
    signer: &InMemorySigner,
    transfer_id: [u8; 32],
) -> Result<String, Box<dyn std::error::Error>> {
    let request = methods::query::RpcQueryRequest {
        block_reference: BlockReference::Finality(Finality::Final),
        request: QueryRequest::CallFunction {
            account_id: "near-swap.yanganto.testnet".parse()?,
            method_name: "check".to_string(),
            args: FunctionArgs::from(
                json!({ "transfer_id": transfer_id })
                    .to_string()
                    .into_bytes(),
            ),
        },
    };

    let response = client.call(request).await?;

    if let QueryResponseKind::CallResult(result) = response.kind {
        Ok(String::from_utf8(result.result).map_err(|e| format!("parsing status error:{e}"))?)
    } else {
        Err("result unexpected".into())
    }
}

#[cfg(feature = "demo")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let alice_account_id = prompt("Enter the Alice account ID: ")?.parse()?;
    let alice_secret_key = prompt("Enter the Alice's private key: ")?.parse()?;
    let alice = InMemorySigner::from_secret_key(alice_account_id, alice_secret_key);

    let bob_account_id = prompt("Enter the Bob account ID: ")?.parse()?;
    let bob_secret_key = prompt("Enter the Bob's private key: ")?.parse()?;
    let bob = InMemorySigner::from_secret_key(bob_account_id, bob_secret_key);

    let client = JsonRpcClient::connect("https://archival-rpc.testnet.near.org");

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let twenty_mins_lock = (now + Duration::new(1200, 0)).as_secs();

    println!("==> Alice call fund");
    let (block_hash, nonce) = get_hash_and_nonce(&client, &alice).await?;
    let function_call_action = FunctionCallAction {
        method_name: "fund".to_string(),
        args: json!({
            "sender": alice.account_id.clone(),
            "receiver": bob.account_id.clone(),
            "amount": 2_000_000,
            "hashlock": [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154
            ],
            "timelock": twenty_mins_lock
        })
        .to_string()
        .into_bytes(),
        gas: 100_000_000_000_000, // 100 TeraGas
        deposit: 2_000_001,
    };
    let result =
        call_and_wait_result(&client, &alice, block_hash, nonce + 1, function_call_action).await?;
    let transfer1_id =
        string_to_bytes32(String::from_utf8(result).map_err(|_| "Fail to decode transfer_id")?)?;
    println!("\ttransfer 1 from Alice to Bob");
    println!("\ttransfer 1 id: {transfer1_id:?}");

    time::sleep(time::Duration::from_secs(6)).await;

    println!("==> Bob check status for Alice's transfer");
    let status = check_transfer(&client, &bob, transfer1_id).await?;
    println!("\tstatus: {status:}");

    time::sleep(time::Duration::from_secs(6)).await;

    println!("==> Bob call fund");
    let (block_hash, nonce) = get_hash_and_nonce(&client, &alice).await?;
    let ten_mins_lock = (now + Duration::new(600, 0)).as_secs();
    let function_call_action = FunctionCallAction {
        method_name: "fund".to_string(),
        args: json!({
            "sender": bob.account_id.clone(),
            "receiver": alice.account_id.clone(),
            "amount": 1_000_000,
            "hashlock": [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154
            ],
            "timelock": ten_mins_lock
        })
        .to_string()
        .into_bytes(),
        gas: 100_000_000_000_000,
        deposit: 1_000_001,
    };
    let result =
        call_and_wait_result(&client, &alice, block_hash, nonce + 1, function_call_action).await?;
    let transfer2_id =
        string_to_bytes32(String::from_utf8(result).map_err(|_| "Fail to decode transfer_id")?)?;
    println!("\ttransfer 2 from Bob to Alice");
    println!("\ttransfer 2 id: {transfer2_id:?}");

    time::sleep(time::Duration::from_secs(6)).await;

    println!("==> Alice check status for Bob's transfer");
    let status = check_transfer(&client, &alice, transfer2_id).await?;
    println!("\tstatus: {status:}");

    time::sleep(time::Duration::from_secs(6)).await;

    println!("==> Alice confirms Bob's transfer");
    let (block_hash, nonce) = get_hash_and_nonce(&client, &alice).await?;
    let function_call_action = FunctionCallAction {
        method_name: "confirm".to_string(),
        args: json!({
            "sender": bob.account_id.clone(),
            "receiver": alice.account_id.clone(),
            "amount": 1_000_000,
            "hashlock": [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154
            ],
            "timelock": ten_mins_lock,
            "secret_key": [
                115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
                115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
            ]
        })
        .to_string()
        .into_bytes(),
        gas: 100_000_000_000_000,
        deposit: 0,
    };
    let _result =
        call_and_wait_result(&client, &alice, block_hash, nonce + 1, function_call_action).await?;
    println!("\ttransfer 2 confirmed");

    time::sleep(time::Duration::from_secs(6)).await;

    println!("==> Bob check status for his transfer");
    let status = check_transfer(&client, &bob, transfer2_id).await?;
    println!("\tstatus: {status:}");

    time::sleep(time::Duration::from_secs(6)).await;

    println!("==> Bob confirms Alice's transfer");
    let (block_hash, nonce) = get_hash_and_nonce(&client, &alice).await?;
    let function_call_action = FunctionCallAction {
        method_name: "confirm".to_string(),
        args: json!({
            "sender": alice.account_id.clone(),
            "receiver": bob.account_id.clone(),
            "amount": 2_000_000,
            "hashlock": [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154
            ],
            "timelock": twenty_mins_lock,
            "secret_key": [
                115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
                115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115, 115,
            ]
        })
        .to_string()
        .into_bytes(),
        gas: 100_000_000_000_000,
        deposit: 0,
    };
    let _result = call_and_wait_result(
        &client,
        &bob,
        block_hash,
        nonce + 1,
        function_call_action.clone(),
    )
    .await?;
    println!("\ttransfer 1 confirmed");

    Ok(())
}

fn prompt(query: &str) -> io::Result<String> {
    print!("{}", query);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}

fn string_to_bytes32(s: String) -> Result<[u8; 32], String> {
    if s.chars().nth(0) != Some('[') {
        return Err("incoorect format".into());
    }
    let v: Vec<&str> = s
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .collect();
    if v.len() != 32 {
        Err("incorrect size".into())
    } else {
        let v: Vec<u8> = v
            .iter()
            .map(|n| n.parse::<u8>().unwrap_or_default())
            .collect();
        Ok([
            v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11], v[12], v[13],
            v[14], v[15], v[16], v[17], v[18], v[19], v[20], v[21], v[22], v[23], v[24], v[25],
            v[26], v[27], v[28], v[29], v[30], v[31],
        ])
    }
}

#[test]
fn test_string_to_byte32() {
    let s = "[145,93,61,123,198,167,204,160,187,162,19,2,247,225,3,113,184,252,221,171,158,170,158,43,93,205,22,169,239,19,221,224]";
    let b = [
        145u8, 93, 61, 123, 198, 167, 204, 160, 187, 162, 19, 2, 247, 225, 3, 113, 184, 252, 221,
        171, 158, 170, 158, 43, 93, 205, 22, 169, 239, 19, 221, 224,
    ];
    assert_eq!(string_to_bytes32(s.into()).unwrap(), b);
}
