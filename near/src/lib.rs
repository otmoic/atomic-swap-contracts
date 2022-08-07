#![allow(clippy::too_many_arguments)]
use std::collections::HashMap;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, log, near_bindgen, require, AccountId, Balance, Promise};
use sha3::{Digest, Sha3_256};

pub type TransferId = [u8; 32];
pub type HashLock = [u8; 32];
pub type SecretKey = [u8; 32];

/// The constant fee to platform for each transfer
const FEE: Balance = 1;
/// The address of the platform to receive the fee
const PLATFORM: &'static str = "platform.near";

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
enum TransferStatus {
    Pending,
    Confirmed,
    Refunded,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
pub enum Event {
    Fund(
        (
            TransferId,
            AccountId, // sender
            AccountId, // receiver
            Balance,
            HashLock,
            u64, // UNIX timestamp
        ),
    ),
    Confirmed((TransferId, SecretKey)),
    Refunded(TransferId),
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    transfers: HashMap<TransferId, TransferStatus>,
    events: Vec<Event>,
}

#[allow(clippy::derivable_impls)]
impl Default for Contract {
    fn default() -> Self {
        Self {
            transfers: HashMap::new(),
            events: vec![],
        }
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn ping_pong(&mut self, ping: String, sender: AccountId) -> String {
        format!("{sender:?} {ping:} -> pong")
    }

    /// sets up a new transfer with hash time lock.
    #[payable]
    pub fn fund(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) {
        if near_sdk::env::attached_deposit() < amount + FEE {
            log!(
                "attached deposit should more than {} and fee {}",
                amount,
                FEE
            );
            env::abort();
        }

        log!("transfer from {} to {}", sender, receiver);

        let transfer_id = keccak256(&sender, &receiver, amount, hashlock, timelock);

        self.transfers
            .insert(transfer_id.clone(), TransferStatus::Pending);

        self.events.push(Event::Fund((
            transfer_id,
            sender,
            receiver,
            amount,
            hashlock,
            timelock,
        )));
    }

    /// confirm a transfer.
    pub fn confirm(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
        secret_key: [u8; 32],
    ) {
        log!("confirm with {}", sender);
        let pending_transfer_id = keccak256(&sender, &receiver, amount, hashlock, timelock);

        if let Some(transfer_status) = self.transfers.get_mut(&pending_transfer_id) {
            require!(
                *transfer_status == TransferStatus::Pending,
                "not pending transfer"
            );
            require!(try_lock(secret_key, hashlock), "incorrect secret_key");

            *transfer_status = TransferStatus::Confirmed;

            let _ = Promise::new(env::current_account_id())
                .transfer(amount)
                .and(Promise::new(PLATFORM.parse().unwrap()).transfer(FEE));

            self.events
                .push(Event::Confirmed((pending_transfer_id, secret_key)));
        } else {
            require!(false, "missing a pending transfer");
            panic!("it should return before go here");
        }
    }

    /// refund when over timelock
    pub fn refund(
        &mut self,
        sender: AccountId,
        receiver: AccountId,
        amount: Balance,
        hashlock: [u8; 32],
        timelock: u64,
    ) {
        log!("refund to {}", sender);
        let pending_transfer_id = keccak256(&sender, &receiver, amount, hashlock, timelock);
        if let Some(transfer_status) = self.transfers.get_mut(&pending_transfer_id) {
            require!(
                *transfer_status == TransferStatus::Pending,
                "not pending transfer"
            );
            require!(
                timelock <= env::block_timestamp(),
                "timelock not yet passed"
            );

            *transfer_status = TransferStatus::Refunded;

            let _ = Promise::new(env::current_account_id()).transfer(amount);
            self.events.push(Event::Refunded(pending_transfer_id));
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
    timelock: u64,
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(sender.as_bytes());
    hasher.update(bridge.as_bytes());
    hasher.update(amount.to_be_bytes());
    hasher.update(hashlock);
    hasher.update(timelock.to_be_bytes());
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

fn try_lock(secret_key: SecretKey, hashlock: HashLock) -> bool {
    let mut hasher = Sha3_256::new();
    hasher.update(secret_key);
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out == hashlock
}

pub fn gen_lock(secret_key: SecretKey) -> HashLock {
    let mut hasher = Sha3_256::new();
    hasher.update(secret_key);
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};
    use std::time::{Duration, SystemTime};

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
        let _fund_contract =
            contract.fund(sender, receiver, 1, [0; 32], five_seconds_later.as_secs());
    }

    #[test]
    fn lock_mechanism() {
        let key = b"ssssssssssssssssssssssssssssssss";
        let lock = gen_lock(*key);
        assert_eq!(
            [
                165, 152, 132, 76, 216, 153, 182, 114, 45, 89, 20, 251, 170, 95, 204, 77, 214, 166,
                43, 58, 171, 243, 206, 181, 109, 46, 63, 177, 197, 13, 234, 154
            ],
            lock
        );
        assert!(try_lock(*key, lock));
    }
}
