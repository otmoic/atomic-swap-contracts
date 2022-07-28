#![allow(clippy::too_many_arguments)]
use std::{collections::HashMap, time::Duration};

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, require, AccountId, Balance, Promise};
use sha3::{Digest, Sha3_256};

pub type TransferId = [u8; 32];
pub type HashLock = [u8; 32];

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
enum TransferStatus {
    Pending,
    Confirmed,
    Refunded,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub enum Event {
    LogNewTransferOut(
        (
            TransferId,
            AccountId, // sender
            AccountId, // receiver
            Balance,
            HashLock,
            u64, // UNIX timestamp
        ),
    ),
    LogNewTransferIn(
        (
            TransferId,
            AccountId, // sender
            AccountId, // receiver
            Balance,
            HashLock,
            u64, // UNIX timestamp
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
    #[payable]
    pub fn transfer_out(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
    ) -> Event {
        require!(env::signer_account_id() == sender, "require sender");
        log!("transfer out to {}", receiver);

        if near_sdk::env::attached_deposit() < amount {
            log!("attached deposit should more than {}", amount);
            env::abort();
        }

        let transfer_id = keccak256(&sender, &receiver, amount, hashlock, timelock);
        self.transfers
            .insert(transfer_id.clone(), TransferStatus::Pending);

        Event::LogNewTransferOut((
            transfer_id,
            sender,
            receiver,
            amount,
            hashlock,
            timelock.as_secs(),
        ))
    }

    /// transfer sets up a new inbound transfer with hash time lock.
    pub fn transfer_in(
        &mut self,
        sender: AccountId,
        dst_address: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
    ) -> Event {
        log!("transfer in from {}", sender);

        require!(env::signer_account_id() == dst_address, "require sender");
        let transfer_id = keccak256(&sender, &dst_address, amount, hashlock, timelock);
        self.transfers
            .insert(transfer_id.clone(), TransferStatus::Pending);

        Event::LogNewTransferIn((
            transfer_id,
            sender,
            dst_address,
            amount,
            hashlock,
            timelock.as_secs(),
        ))
    }

    /// confirm a transfer.
    pub fn confirm(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
        preimage: [u8; 32],
    ) -> Event {
        log!("confirm with {}", sender);
        let pending_transfer_id = keccak256(&sender, &receiver, amount, hashlock, timelock);

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

            let _ = Promise::new(env::current_account_id()).transfer(amount);
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
        amount: Balance,
        hashlock: [u8; 32],
        timelock: Duration,
    ) -> Event {
        log!("refund to {}", sender);
        let pending_transfer_id = keccak256(&sender, &receiver, amount, hashlock, timelock);
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

            let _ = Promise::new(env::current_account_id()).transfer(amount);
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
    amount: Balance,
    hashlock: [u8; 32],
    timelock: Duration,
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(sender.as_bytes());
    hasher.update(bridge.as_bytes());
    hasher.update(amount.to_be_bytes());
    hasher.update(hashlock);
    hasher.update(timelock.as_secs().to_be_bytes());
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
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use std::time::SystemTime;

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .signer_account_id("caller".parse().unwrap())
            .is_view(is_view)
            .build()
    }

    /// Test transfer out without attach balance
    #[test]
    #[should_panic]
    fn initial_contract() {
        let context = get_context(false);
        testing_env!(context);

        let mut contract = Contract::default();
        let sender: AccountId = "caller".parse().unwrap();
        let receiver: AccountId = "receiver".parse().unwrap();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();
        let five_seconds_later = now + Duration::new(5, 0);
        let _transfer_out = contract.transfer_out(sender, receiver, 1, [0; 32], five_seconds_later);
    }
}
