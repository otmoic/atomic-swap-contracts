#![feature(derive_default_enum)]
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

/// The constant fee to platform for each transfer
const FEE: u64 = 1;
/// The address of the platform to receive the fee
const PLATFORM: Pubkey = Pubkey::new_from_array([255; 32]);

#[derive(BorshDeserialize, BorshSerialize)]
pub enum Method {
    Fund(u64, HashLock, u64),
    Confirm(u64, HashLock, u64, SecretKey),
    Refund(u64, HashLock, u64),
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug, Default)]
pub enum TransferStatus {
    #[default]
    Initializd,
    Pending,
    Confirmed,
    Refunded,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Default)]
pub struct Storage {
    pub sender: Pubkey,
    pub receiver: Pubkey,
    pub amount: u64,
    pub hashlock: HashLock,
    pub timelock: u64,
    pub secret_key: SecretKey,
    pub status: TransferStatus,
}

pub enum Error {
    SecretNotMatch = 1,
    TransferExisting = 2,
    TransferNotMatch = 3,
    LockByTime = 4,
    PlatfromIncorrect = 5,
}

entrypoint!(atomic_swap);

fn fund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    hashlock: HashLock,
    timelock: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let contract = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let receiver = next_account_info(accounts_iter)?;
    let platform = next_account_info(accounts_iter)?;
    msg!("transfer from {} to {}", sender.key, receiver.key);
    if *platform.key != PLATFORM {
        return Err(ProgramError::Custom(Error::PlatfromIncorrect as u32));
    }
    if contract.owner == program_id {
        invoke(
            &transfer(&sender.key, &platform.key, amount + FEE),
            &[sender.clone(), platform.clone()],
        )?;
        invoke(
            &transfer(&sender.key, &contract.key, amount),
            &[sender.clone(), contract.clone()],
        )?;
        let mut storage = Storage::try_from_slice(&contract.data.borrow())?;
        if storage.status == TransferStatus::Initializd {
            storage.sender = *sender.key;
            storage.receiver = *receiver.key;
            storage.amount = amount;
            storage.hashlock = hashlock;
            storage.timelock = timelock;
            storage.status = TransferStatus::Pending;
            storage.serialize(&mut &mut contract.data.borrow_mut()[..])?;
            Ok(())
        } else {
            Err(ProgramError::Custom(Error::TransferExisting as u32))
        }
    } else {
        Err(ProgramError::IncorrectProgramId)
    }
}

fn confirm(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    hashlock: HashLock,
    timelock: u64,
    secret_key: SecretKey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let contract = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let receiver = next_account_info(accounts_iter)?;
    msg!("confirm with {}", sender.key);
    if contract.owner == program_id {
        if !try_lock(secret_key, hashlock) {
            Err(ProgramError::Custom(Error::SecretNotMatch as u32))
        } else {
            let mut storage = Storage::try_from_slice(&contract.data.borrow())?;
            if storage.status == TransferStatus::Pending
                && storage.sender == *sender.key
                && storage.receiver == *receiver.key
                && storage.amount == amount
                && storage.hashlock == hashlock
                && storage.timelock == timelock
            {
                invoke(
                    &transfer(&contract.key, &receiver.key, amount),
                    &[contract.clone(), receiver.clone()],
                )?;
                storage.secret_key = secret_key;
                storage.status = TransferStatus::Confirmed;
                storage.serialize(&mut &mut contract.data.borrow_mut()[..])?;
                Ok(())
            } else {
                Err(ProgramError::Custom(Error::TransferNotMatch as u32))
            }
        }
    } else {
        Err(ProgramError::IncorrectProgramId)
    }
}

fn refund(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
    hashlock: HashLock,
    timelock: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let contract = next_account_info(accounts_iter)?;
    let sender = next_account_info(accounts_iter)?;
    let receiver = next_account_info(accounts_iter)?;
    msg!("refund with {}", sender.key);
    if contract.owner == program_id {
        let now_ts = Clock::get().unwrap().unix_timestamp;
        if (now_ts as u64) < timelock {
            Err(ProgramError::Custom(Error::LockByTime as u32))
        } else {
            let mut storage = Storage::try_from_slice(&contract.data.borrow())?;
            if storage.status == TransferStatus::Pending
                && storage.sender == *sender.key
                && storage.receiver == *receiver.key
                && storage.amount == amount
                && storage.hashlock == hashlock
                && storage.timelock == timelock
            {
                invoke(
                    &transfer(&contract.key, &sender.key, amount),
                    &[contract.clone(), sender.clone()],
                )?;
                storage.status = TransferStatus::Refunded;
                storage.serialize(&mut &mut contract.data.borrow_mut()[..])?;
                Ok(())
            } else {
                Err(ProgramError::Custom(Error::TransferNotMatch as u32))
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
        match method {
            Method::Fund(amount, hashlock, timelock) => {
                fund(program_id, accounts, amount, hashlock, timelock)?
            }
            Method::Confirm(amount, hashlock, timelock, secret_key) => {
                confirm(program_id, accounts, amount, hashlock, timelock, secret_key)?
            }
            Method::Refund(amount, hashlock, timelock) => {
                refund(program_id, accounts, amount, hashlock, timelock)?
            }
        }
    } else {
        msg!("Unsupported method");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use solana_program::clock::Epoch;
    use std::time::{Duration, SystemTime};

    #[test]
    fn sanity_round_trip() {
        let program_id = Pubkey::default();
        let sender_key = Pubkey::default();
        let mut lamports = 0;
        let mut data = Storage::default().try_to_vec().unwrap();
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &sender_key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );
        let receiver_key = Pubkey::new_from_array([1; 32]);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let five_seconds_later = now + Duration::new(5, 0);

        let method = Method::Fund(
            sender_key,
            receiver_key,
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            five_seconds_later.as_secs(),
        );
        let instruction_data: Vec<u8> = method.try_to_vec().unwrap();

        let accounts = vec![account];

        let mut storage = Storage::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(storage.status, TransferStatus::Initializd);

        atomic_swap(&program_id, &accounts, &instruction_data).unwrap();

        storage = Storage::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(storage.status, TransferStatus::Pending);

        let method = Method::Confirm(
            sender_key,
            receiver_key,
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            five_seconds_later.as_secs(),
            *b"ssssssssssssssssssssssssssssssss",
        );
        let instruction_data: Vec<u8> = method.try_to_vec().unwrap();

        atomic_swap(&program_id, &accounts, &instruction_data).unwrap();

        storage = Storage::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(storage.status, TransferStatus::Confirmed);
    }

    #[test]
    fn incorrect_secret() {
        let program_id = Pubkey::default();
        let sender_key = Pubkey::default();
        let mut lamports = 0;
        let mut data = Storage::default().try_to_vec().unwrap();
        let owner = Pubkey::default();
        let account = AccountInfo::new(
            &sender_key,
            false,
            true,
            &mut lamports,
            &mut data,
            &owner,
            false,
            Epoch::default(),
        );
        let receiver_key = Pubkey::new_from_array([1; 32]);
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let five_seconds_later = now + Duration::new(5, 0);

        let method = Method::Fund(
            sender_key,
            receiver_key,
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            five_seconds_later.as_secs(),
        );
        let instruction_data: Vec<u8> = method.try_to_vec().unwrap();

        let accounts = vec![account];

        let mut storage = Storage::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(storage.status, TransferStatus::Initializd);

        atomic_swap(&program_id, &accounts, &instruction_data).unwrap();

        storage = Storage::try_from_slice(&accounts[0].data.borrow()).unwrap();
        assert_eq!(storage.status, TransferStatus::Pending);

        let method = Method::Confirm(
            sender_key,
            receiver_key,
            100,
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154,
            ],
            five_seconds_later.as_secs(),
            *b"nnnnnnnnnnnnnnnnnnnnnnnnnnnnnnnn",
        );
        let instruction_data: Vec<u8> = method.try_to_vec().unwrap();

        assert_eq!(
            atomic_swap(&program_id, &accounts, &instruction_data),
            Err(ProgramError::Custom(Error::SecretNotMatch as u32))
        );
    }
}
