use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    clock::Clock,
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_instruction::transfer,
    sysvar::Sysvar,
};

use utils::{try_lock, HashLock, SecretKey};

#[derive(BorshDeserialize, BorshSerialize)]
pub enum Method {
    Fund(Pubkey, Pubkey, u64, HashLock, u64),
    Confirm(Pubkey, Pubkey, u64, HashLock, u64, SecretKey),
    Refund(Pubkey, Pubkey, u64, HashLock, u64),
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub enum TransferStatus {
    Pending((Pubkey, Pubkey, u64, u64, HashLock)),
    Confirmed((Pubkey, Pubkey, u64, u64, SecretKey)),
    Refunded,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Storage {
    pub transfers: HashMap<(Pubkey, Pubkey, u64, HashLock, u64), TransferStatus>,
}

pub enum Error {
    SecretNotMatch = 1,
    TransferNotFund = 2,
    TransferNotPending = 3,
    LockByTime = 4,
}

entrypoint!(atomic_swap);

fn fund(
    program_id: &Pubkey,
    account: &AccountInfo,
    sender: &Pubkey,
    receiver: &Pubkey,
    amount: u64,
    hashlock: HashLock,
    timelock: u64,
) -> ProgramResult {
    msg!(
        "transfer from {} to {}, account({:?})",
        sender,
        receiver,
        account
    );
    if account.owner == program_id {
        invoke(&transfer(sender, program_id, amount), &[account.clone()])?;
        let mut storage = Storage::try_from_slice(&account.data.borrow())?;
        storage.transfers.insert(
            (*sender, *receiver, amount, hashlock, timelock),
            TransferStatus::Pending((*sender, *receiver, amount, timelock, hashlock)),
        );
        Ok(())
    } else {
        Err(ProgramError::IncorrectProgramId)
    }
}

fn confirm(
    program_id: &Pubkey,
    account: &AccountInfo,
    sender: &Pubkey,
    receiver: &Pubkey,
    amount: u64,
    hashlock: HashLock,
    timelock: u64,
    secret_key: SecretKey,
) -> ProgramResult {
    msg!("confirm with {}", sender);
    if account.owner == program_id {
        if !try_lock(secret_key, hashlock) {
            Err(ProgramError::Custom(Error::SecretNotMatch as u32))
        } else {
            let mut storage = Storage::try_from_slice(&account.data.borrow())?;
            invoke(&transfer(program_id, receiver, amount), &[account.clone()])?;
            if let Some(transfer_status) = storage
                .transfers
                .get_mut(&(*sender, *receiver, amount, hashlock, timelock))
            {
                match transfer_status {
                    TransferStatus::Pending(_) => {
                        *transfer_status = TransferStatus::Confirmed((
                            *sender, *receiver, amount, timelock, secret_key,
                        ));
                        Ok(())
                    }
                    _ => Err(ProgramError::Custom(Error::TransferNotPending as u32)),
                }
            } else {
                Err(ProgramError::Custom(Error::TransferNotFund as u32))
            }
        }
    } else {
        Err(ProgramError::IncorrectProgramId)
    }
}

fn refund(
    program_id: &Pubkey,
    account: &AccountInfo,
    sender: &Pubkey,
    receiver: &Pubkey,
    amount: u64,
    hashlock: HashLock,
    timelock: u64,
) -> ProgramResult {
    msg!("refund to {}", sender);
    if account.owner == program_id {
        let now_ts = Clock::get().unwrap().unix_timestamp;
        if (now_ts as u64) < timelock {
            Err(ProgramError::Custom(Error::LockByTime as u32))
        } else {
            let mut storage = Storage::try_from_slice(&account.data.borrow())?;
            invoke(&transfer(program_id, receiver, amount), &[account.clone()])?;
            if let Some(transfer_status) = storage
                .transfers
                .get_mut(&(*sender, *receiver, amount, hashlock, timelock))
            {
                match transfer_status {
                    TransferStatus::Pending(_) => {
                        *transfer_status = TransferStatus::Refunded;
                        Ok(())
                    }
                    _ => Err(ProgramError::Custom(Error::TransferNotPending as u32)),
                }
            } else {
                Err(ProgramError::Custom(Error::TransferNotFund as u32))
            }
        }
    } else {
        Err(ProgramError::IncorrectProgramId)
    }
}

pub fn atomic_swap(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Ok(method) = Method::try_from_slice(instruction_data) {
        let accounts_iter = &mut accounts.iter();
        let account = next_account_info(accounts_iter)?;
        match method {
            Method::Fund(sender, receiver, amount, hashlock, timelock) => fund(
                program_id, account, &sender, &receiver, amount, hashlock, timelock,
            )?,
            Method::Confirm(sender, receiver, amount, hashlock, timelock, secret_key) => confirm(
                program_id, account, &sender, &receiver, amount, hashlock, timelock, secret_key,
            )?,
            Method::Refund(sender, receiver, amount, hashlock, timelock) => refund(
                program_id, account, &sender, &receiver, amount, hashlock, timelock,
            )?,
        }
    } else {
        msg!("Unsupported method");
    }

    Ok(())
}
