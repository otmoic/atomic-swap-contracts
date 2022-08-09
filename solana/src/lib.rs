use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, msg, pubkey::Pubkey,
};
use std::collections::HashMap;

pub type TransferId = [u8; 32];
pub type HashLock = [u8; 32];
pub type SecretKey = [u8; 32];

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
    _instruction_data: &[u8],
) -> ProgramResult {
    msg!("Atomic swap entrypoint");
    Ok(())
}
