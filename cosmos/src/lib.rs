#![allow(clippy::derive_partial_eq_without_eq)]
use cosmwasm_std::{
    entry_point, to_binary, BankMsg, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdError,
    StdResult,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};

use cosmwasm_std::{Addr, Coin, Storage};
use cosmwasm_storage::{
    bucket, bucket_read, singleton, singleton_read, Bucket, ReadonlyBucket, ReadonlySingleton,
    Singleton,
};

use thiserror::Error;
use utils::{try_lock, HashLock, SecretKey};

pub static CONFIG_KEY: &[u8] = b"config";
pub static TRANSFER_KEY: &[u8] = b"transfers";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub platform: String,
    pub fee: Coin,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, JsonSchema)]
pub enum TransferStatus {
    Pending,
    Confirmed,
    Refunded,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TransferRecord {
    pub sender: Addr,
    pub receiver: Addr,
    pub coin: Coin,
    pub hashlock: HashLock,
    pub timelock: u64,
    pub secret_key: SecretKey,
    pub status: TransferStatus,
}

pub fn config(storage: &mut dyn Storage) -> Singleton<Config> {
    singleton(storage, CONFIG_KEY)
}

pub fn config_read(storage: &dyn Storage) -> ReadonlySingleton<Config> {
    singleton_read(storage, CONFIG_KEY)
}

pub fn transfers(storage: &mut dyn Storage) -> Bucket<TransferRecord> {
    bucket(storage, TRANSFER_KEY)
}

pub fn transfers_read(storage: &dyn Storage) -> ReadonlyBucket<TransferRecord> {
    bucket_read(storage, TRANSFER_KEY)
}

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Insufficient funds sent")]
    InsufficientFundsSend,

    #[error("Transfer does not exist")]
    TransferNotExists,

    #[error("The secret not correct")]
    IncorrectSecret,

    #[error("Transfer still locked")]
    TransferLocked,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    platform: String,
    fee: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Fund(TransferMsg),
    Confirm((TransferMsg, SecretKey)),
    Refund(TransferMsg),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TransferMsg {
    sender: String,
    receiver: String,
    coin: Coin,
    hashlock: HashLock,
    timelock: u64,
}

#[cfg(test)]
mod tests;

fn assert_sent_sufficient_coin(
    sent: &[Coin],
    required_coin: &Coin,
    fee_coin: &Coin,
) -> Result<(), ContractError> {
    let required_amount = required_coin.amount.u128();
    let fee_amount = fee_coin.amount.u128();
    let sent_sufficient_funds = sent.iter().any(|coin| {
        coin.denom == required_coin.denom && coin.amount.u128() >= required_amount + fee_amount
    });

    if sent_sufficient_funds {
        Ok(())
    } else {
        Err(ContractError::InsufficientFundsSend {})
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let config_state = Config {
        platform: msg.platform,
        fee: msg.fee,
    };

    config(deps.storage).save(&config_state)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Fund(msg) => {
            let TransferMsg {
                sender,
                receiver,
                coin,
                hashlock,
                timelock,
            } = msg;
            let config_state = config(deps.storage).load()?;
            assert_sent_sufficient_coin(&info.funds, &coin, &config_state.fee)?;
            let transfer_id = keccak256(&sender, &receiver, &coin, &hashlock, timelock);
            let record = TransferRecord {
                sender: deps.api.addr_validate(&sender)?,
                receiver: deps.api.addr_validate(&receiver)?,
                coin,
                hashlock,
                timelock,
                secret_key: [0; 32],
                status: TransferStatus::Pending,
            };
            transfers(deps.storage).save(&transfer_id, &record)?;
            Ok(Response::default())
        }
        ExecuteMsg::Confirm((msg, secret)) => {
            let TransferMsg {
                sender,
                receiver,
                coin,
                hashlock,
                timelock,
            } = msg;
            let transfer_id = keccak256(&sender, &receiver, &coin, &hashlock, timelock);
            transfers(deps.storage).update(&transfer_id, |t| {
                if let Some(mut transfer) = t {
                    if try_lock(secret, hashlock) && transfer.status == TransferStatus::Pending {
                        transfer.secret_key = secret;
                        transfer.status = TransferStatus::Confirmed;
                        Ok(transfer)
                    } else {
                        Err(ContractError::IncorrectSecret {})
                    }
                } else {
                    Err(ContractError::TransferNotExists {})
                }
            })?;
            let config_state = config(deps.storage).load()?;
            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: receiver,
                    amount: vec![coin],
                })
                .add_message(BankMsg::Send {
                    to_address: config_state.platform,
                    amount: vec![config_state.fee],
                }))
        }
        ExecuteMsg::Refund(msg) => {
            let TransferMsg {
                sender,
                receiver,
                coin,
                hashlock,
                timelock,
            } = msg;
            let transfer_id = keccak256(&sender, &receiver, &coin, &hashlock, timelock);
            transfers(deps.storage).update(&transfer_id, |t| {
                if let Some(mut transfer) = t {
                    if env.block.time.seconds() > timelock {
                        transfer.status = TransferStatus::Refunded;
                        Ok(transfer)
                    } else {
                        Err(ContractError::TransferLocked {})
                    }
                } else {
                    Err(ContractError::TransferNotExists {})
                }
            })?;
            let config_state = config(deps.storage).load()?;
            Ok(Response::new()
                .add_message(BankMsg::Send {
                    to_address: sender,
                    amount: vec![coin],
                })
                .add_message(BankMsg::Send {
                    to_address: config_state.platform.to_string(),
                    amount: vec![config_state.fee],
                }))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: TransferMsg) -> StdResult<Binary> {
    let TransferMsg {
        sender,
        receiver,
        coin,
        hashlock,
        timelock,
    } = msg;
    let transfer_id = keccak256(&sender, &receiver, &coin, &hashlock, timelock);
    let transfer = transfers_read(deps.storage).load(&transfer_id)?;
    to_binary(&transfer)
}

fn keccak256(
    sender: &String,
    bridge: &String,
    coin: &Coin,
    hashlock: &[u8; 32],
    timelock: u64,
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(sender.as_bytes());
    hasher.update(bridge.as_bytes());
    hasher.update(coin.amount.to_be_bytes());
    hasher.update(coin.denom.as_bytes());
    hasher.update(hashlock);
    hasher.update(timelock.to_be_bytes());
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}
