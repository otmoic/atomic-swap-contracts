use std::collections::HashMap;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};

use utils::{HashLock, SecretKey};

pub type TransferId = [u8; 32];

#[derive(BorshDeserialize, BorshSerialize)]
pub enum Methods {
    Fund(Pubkey, Pubkey, u64, HashLock, u64),
    Confirm(Pubkey, Pubkey, u64, HashLock, u64, SecretKey),
    Refund(Pubkey, Pubkey, u64, HashLock, u64),
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub enum TransferStatus {
    Pending,
    Confirmed,
    Refunded,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Transfer {
    pub transfers: HashMap<TransferId, TransferStatus>,
}

entrypoint!(atomic_swap);

pub fn atomic_swap(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Ok(method) = Methods::try_from_slice(instruction_data) {
        match method {
            _ => (),
        }
    } else {
        msg!("Unsupport method");
    }

    Ok(())
}
