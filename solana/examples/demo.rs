#![allow(unused)]
//! Demo client on testnet

#[cfg(feature = "demo")]
use std::{
    error::Error,
    io::{self, Write},
    time::{Duration, SystemTime},
};

#[cfg(feature = "demo")]
use borsh::{BorshDeserialize, BorshSerialize};

#[cfg(feature = "demo")]
use solana_rpc_client::rpc_client::RpcClient;

#[cfg(feature = "demo")]
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    pubkey::Pubkey,
    signature::Signer,
    signer::keypair::{read_keypair_file, Keypair},
    transaction::Transaction,
};

#[cfg(feature = "demo")]
use atomicswap::{Method, Storage, TransferStatus};

#[cfg(not(feature = "demo"))]
fn main() {
    println!("Please run this demo app with demo feature");
}

#[cfg(feature = "demo")]
fn main() -> Result<(), Box<dyn Error>> {
    let alice = ask_key_file("Enter Alice key file:")?;
    let bob = ask_key_file("Enter Bob key file:")?;

    let platform_key = Pubkey::new_from_array([
        248, 168, 61, 18, 213, 218, 160, 220, 199, 48, 254, 164, 209, 214, 235, 60, 128, 101, 144,
        242, 95, 58, 210, 60, 85, 146, 228, 120, 192, 220, 18, 161,
    ]);

    let client = RpcClient::new("https://api.testnet.solana.com");

    let alice_balance = client.get_balance(&alice.pubkey());
    let bob_balance = client.get_balance(&bob.pubkey());
    println!("Alice balance: {alice_balance:?}");
    println!("Bob balance: {bob_balance:?}");

    let program = Pubkey::new_from_array(
        TryInto::<[u8; 32]>::try_into(
            bs58::decode("AGFbaMhMvQvmaiWxGGg1EfPtV7zbyuj6rYddUvuX2LbX")
                .into_vec()
                .unwrap(),
        )
        .unwrap(),
    );

    println!("==> Alice create contract and call fund");
    let contract_1 = create_contract_account(&alice, &program, "alice -> bob 1000 NEAR", &client)?;

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let twenty_mins_lock = (now + Duration::new(1200, 0)).as_secs();

    let method = Method::Fund(
        1_000_000_000_000_000,
        [
            165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43,
            58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
        ],
        twenty_mins_lock,
    );

    let instruction = Instruction::new_with_borsh(
        contract_1,
        &method,
        vec![
            AccountMeta::new(program, false),
            AccountMeta::new(alice.pubkey(), true),
            AccountMeta::new(bob.pubkey(), false),
            AccountMeta::new(platform_key, false),
        ],
    );
    let message = Message::new(&[instruction], Some(&alice.pubkey()));
    let transaction = Transaction::new(&[&alice], message, client.get_latest_blockhash()?);

    let result = client.send_and_confirm_transaction(&transaction)?;
    println!("\tresult: {result:?}");

    println!("==> Bob check status for Alice's transfer");
    let contract_1_account = client.get_account(&contract_1)?;
    assert_eq!(
        Storage::try_from_slice(&contract_1_account.data)
            .unwrap()
            .status,
        TransferStatus::Pending
    );
    // NOTE: other details also should check here.

    println!("==> Bob create contract and call fund");
    let contract_2 = create_contract_account(&bob, &program, "bob -> alice 2000 NEAR", &client)?;
    let ten_mins_lock = (now + Duration::new(600, 0)).as_secs();
    let method = Method::Fund(
        2_000_000_000_000_000,
        [
            165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43,
            58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
        ],
        ten_mins_lock,
    );

    let instruction = Instruction::new_with_borsh(
        contract_2,
        &method,
        vec![
            AccountMeta::new(program, false),
            AccountMeta::new(bob.pubkey(), true),
            AccountMeta::new(alice.pubkey(), false),
            AccountMeta::new(platform_key, false),
        ],
    );
    let message = Message::new(&[instruction], Some(&bob.pubkey()));
    let transaction = Transaction::new(&[&bob], message, client.get_latest_blockhash()?);

    let result = client.send_and_confirm_transaction(&transaction)?;
    println!("\tresult: {result:?}");

    println!("==> Alice check status for Bob's transfer");
    let contract_2_account = client.get_account(&contract_2)?;
    assert_eq!(
        Storage::try_from_slice(&contract_2_account.data)
            .unwrap()
            .status,
        TransferStatus::Pending
    );
    // NOTE: other details also should check here.

    println!("==> Alice confirms Bob's transfer");
    let method = Method::Confirm(
        2_000_000_000_000_000,
        [
            165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43,
            58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
        ],
        ten_mins_lock,
        *b"ssssssssssssssssssssssssssssssss",
    );

    let instruction = Instruction::new_with_borsh(
        contract_2,
        &method,
        vec![
            AccountMeta::new(program, false),
            AccountMeta::new(bob.pubkey(), false),
            AccountMeta::new(alice.pubkey(), true),
        ],
    );
    let message = Message::new(&[instruction], Some(&alice.pubkey()));
    let transaction = Transaction::new(&[&alice], message, client.get_latest_blockhash()?);

    let result = client.send_and_confirm_transaction(&transaction)?;
    println!("\tresult: {result:?}");

    println!("==> Bob check status for his transfer");
    let contract_2_account = client.get_account(&contract_2)?;
    let storage = Storage::try_from_slice(&contract_2_account.data).unwrap();
    assert_eq!(storage.status, TransferStatus::Confirmed);
    let secret_key = storage.secret_key;

    println!("==> Bob confirms Alice's transfer");
    let method = Method::Confirm(
        1_000_000_000_000_000,
        [
            165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166, 43,
            58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
        ],
        twenty_mins_lock,
        secret_key,
    );

    let instruction = Instruction::new_with_borsh(
        contract_1,
        &method,
        vec![
            AccountMeta::new(program, false),
            AccountMeta::new(alice.pubkey(), false),
            AccountMeta::new(bob.pubkey(), true),
        ],
    );
    let message = Message::new(&[instruction], Some(&bob.pubkey()));
    let transaction = Transaction::new(&[&bob], message, client.get_latest_blockhash()?);

    let result = client.send_and_confirm_transaction(&transaction)?;
    println!("\tresult: {result:?}");

    Ok(())
}

#[cfg(feature = "demo")]
fn ask_key_file(query: &str) -> Result<Keypair, Box<dyn Error>> {
    print!("{}", query);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    read_keypair_file(input.trim())
}

#[cfg(feature = "demo")]
fn gen_contract_public_key(
    owner: &Pubkey,
    program: &Pubkey,
    seed: &str,
) -> Result<Pubkey, Box<dyn Error>> {
    Ok(Pubkey::create_with_seed(owner, seed, program)?)
}

#[cfg(feature = "demo")]
fn create_contract_account(
    owner: &Keypair,
    program: &Pubkey,
    seed: &str,
    client: &RpcClient,
) -> Result<Pubkey, Box<dyn Error>> {
    let contract_pubkey = gen_contract_public_key(&owner.pubkey(), program, seed)?;

    if let Err(_) = client.get_account(&contract_pubkey) {
        let data_size = Storage::default().try_to_vec().unwrap().len();
        let lamport_requirement = client.get_minimum_balance_for_rent_exemption(data_size)?;

        let instruction = solana_sdk::system_instruction::create_account_with_seed(
            &owner.pubkey(),
            &contract_pubkey,
            &owner.pubkey(),
            seed,
            lamport_requirement,
            data_size as u64,
            program,
        );
        let message = Message::new(&[instruction], Some(&owner.pubkey()));
        let transaction = Transaction::new(&[owner], message, client.get_latest_blockhash()?);

        client.send_and_confirm_transaction(&transaction)?;
    } else {
        println!("contract exist, use the existing one");
    }
    Ok(contract_pubkey)
}
