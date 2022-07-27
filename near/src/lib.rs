#![allow(clippy::too_many_arguments)]
use std::{collections::HashMap, time::Duration};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, require, AccountId, Balance, Promise};
use sha3::{Digest, Sha3_256};

pub type TransferId = [u8; 32];
pub type HashLock = [u8; 32];

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
enum TransferStatus {
    Null,
    Pending,
    Confirmed,
    Refunded,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum Event {
    LogNewTransferOut(
        (
            TransferId,
            AccountId, // sender
            AccountId, // receiver
            AccountId, // token
            Balance,
            HashLock,
            u64,       // UNIX timestamp
            u64,       // dst chain Id
            AccountId, // dst address
            u64,
        ),
    ),
    LogNewTransferIn(
        (
            TransferId,
            AccountId, // sender
            AccountId, // receiver
            AccountId, // toekn
            Balance,
            HashLock,
            u64,        // UNIX timestamp
            u64,        // dst chain Id
            TransferId, // outbound transfer id at src chain
        ),
    ),
    LogTransferConfirmed((TransferId, [u8; 32])),
    LogTransferRefunded(TransferId),
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    transfers: HashMap<TransferId, TransferStatus>,
}

#[allow(clippy::derivable_impls)]
impl Default for Contract {
    fn default() -> Self {
        Self {
            transfers: HashMap::new(),
        }
    }
}

#[near_bindgen]
impl Contract {
    /// transfer sets up a new outbound transfer with hash time lock.
    pub fn transfer_out(
        &mut self,
        sender: AccountId,
        bridge: AccountId,
        token: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
        dst_chain_id: u64,
        dst_address: AccountId,
        bid_id: u64,
    ) -> Event {
        log!("transfer out from {}", sender);
        require!(env::current_account_id() == sender, "require sender");
        let transfer_id = keccak256(&sender, &bridge, &token, amount, hashlock, timelock);
        Event::LogNewTransferOut((
            transfer_id,
            sender,
            bridge,
            token,
            amount,
            hashlock,
            timelock.as_secs(),
            dst_chain_id,
            dst_address,
            bid_id,
        ))
    }

    /// transfer sets up a new inbound transfer with hash time lock.
    pub fn transfer_in(
        &self,
        sender: AccountId,
        dst_address: AccountId,
        token: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
        src_chain_id: u64,
        src_transfer_id: TransferId,
    ) -> Event {
        log!("transfer in from {}", sender);
        require!(env::current_account_id() == sender, "require sender");
        let transfer_id = keccak256(&sender, &dst_address, &token, amount, hashlock, timelock);
        Event::LogNewTransferIn((
            transfer_id,
            sender,
            dst_address,
            token,
            amount,
            hashlock,
            timelock.as_secs(),
            src_chain_id,
            src_transfer_id,
        ))
    }

    /// confirm a transfer.
    pub fn confirm(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        token: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
        preimage: [u8; 32],
    ) -> Event {
        log!("confirm with {}", sender);
        let pending_transfer_id = keccak256_pending_transfer(
            &sender, &receiver, hashlock, timelock, &token, amount, b"NEAR",
        );

        if let Some(transfer_status) = self.transfers.get_mut(&pending_transfer_id) {
            require!(
                *transfer_status == TransferStatus::Pending,
                "not pending transfer"
            );
            require!(
                hashlock == keccak256_preimage(preimage),
                "incorrect preimage"
            );

            *transfer_status = TransferStatus::Confirmed;

            let _ = Promise::new(receiver).transfer(amount);
            Event::LogTransferConfirmed((pending_transfer_id, preimage))
        } else {
            require!(false, "missing a pending transfer");
            panic!("it should return before go here");
        }
    }

    pub fn refund(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        token: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
    ) -> Event {
        log!("refund to {}", sender);
        let pending_transfer_id = keccak256_pending_transfer(
            &sender, &receiver, hashlock, timelock, &token, amount, b"NEAR",
        );
        if let Some(transfer_status) = self.transfers.get_mut(&pending_transfer_id) {
            require!(
                *transfer_status == TransferStatus::Pending,
                "not pending transfer"
            );
            require!(
                timelock.as_secs() <= env::block_timestamp(),
                "timelock not yet passed"
            );

            *transfer_status = TransferStatus::Refunded;

            let _ = Promise::new(sender).transfer(amount);
            Event::LogTransferRefunded(pending_transfer_id)
        } else {
            require!(false, "missing a pending transfer");
            panic!("it should return before go here");
        }
    }
}

fn keccak256(
    sender: &AccountId,
    bridge: &AccountId,
    token: &AccountId,
    amount: Balance,
    hashlock: [u8; 32],
    timelock: Duration,
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(sender.as_bytes());
    hasher.update(bridge.as_bytes());
    hasher.update(token.as_bytes());
    hasher.update(amount.to_be_bytes());
    hasher.update(hashlock);
    hasher.update(timelock.as_secs().to_be_bytes());
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

fn keccak256_pending_transfer(
    sender: &AccountId,
    receiver: &AccountId,
    hashlock: [u8; 32],
    timelock: Duration,
    token: &AccountId,
    amount: Balance,
    chain_id: &[u8; 4],
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(sender.as_bytes());
    hasher.update(receiver.as_bytes());
    hasher.update(hashlock);
    hasher.update(timelock.as_secs().to_be_bytes());
    hasher.update(token.as_bytes());
    hasher.update(amount.to_be_bytes());
    hasher.update(chain_id);
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

fn keccak256_preimage(preimage: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(preimage);
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_contract() {
        Contract::default();
    }
}
