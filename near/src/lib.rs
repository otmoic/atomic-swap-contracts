#![allow(clippy::too_many_arguments)]

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{log, near_bindgen, AccountId, Balance};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct Contract {}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {}
    }

    pub fn transfer_out(
        &self,
        sender: AccountId,
        _bridge: AccountId,
        _token: AccountId,
        _amount: Balance,
        _hashlock: [u8; 32],
        _timelock: u64,
        _dst_chain_id: u64,
        _dst_address: AccountId,
        _bid_id: u64,
    ) {
        log!("transfer out from {}", sender);
    }
    pub fn transfer_in(
        &self,
        sender: AccountId,
        _dst_address: AccountId,
        _token: AccountId,
        _amount: Balance,
        _hashlock: [u8; 32],
        _timelock: u64,
        _src_chain_id: u64,
        _src_transfer_id: [u8; 32],
    ) {
        log!("transfer in from {}", sender);
    }
    pub fn confirm(
        &self,
        sender: AccountId,
        _receiver: AccountId,
        _token: AccountId,
        _amount: Balance,
        _hashlock: [u8; 32],
        _timelock: u64,
        _preimage: [u8; 32],
    ) {
        log!("confirm with {}", sender);
    }
    pub fn refund(
        &self,
        sender: AccountId,
        _receiver: AccountId,
        _token: AccountId,
        _amount: Balance,
        _hashlock: [u8; 32],
        _timelock: u64,
    ) {
        log!("refund to {}", sender);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_contract() {
        Contract::new();
    }
}
