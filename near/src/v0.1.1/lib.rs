use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use near_sdk::{ext_contract, Gas, PromiseResult, env, log, require, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue, StorageUsage, CryptoHash, BorshStorageKey};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_sdk::json_types::{ValidAccountId, U128};

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256, Keccak256};
use num_bigint::BigUint;
use num256::uint256::Uint256;
use num_traits::cast::ToPrimitive;

mod event;
use event::{NearEvent, TransferOutData, TransferInData, TransferConfirmedData, TransferRefundedData};

pub const DEPOSIT_ONE_YOCTO: Balance = 1;
pub const NO_DEPOSIT: Balance = 0;
pub const FT_TRANSFER_GAS: Gas = Gas(10_000_000_000_000);
pub const FT_HARVEST_CALLBACK_GAS: Gas = Gas(10_000_000_000_000);

fn keccak256(
    sender: &AccountId,
    bridge: &AccountId,
    token: &TokenId,
    amount: &[u8; 32],
    hashlock: &[u8; 32],
    timelock: &u64,
) -> [u8; 32] {
    let mut hasher = Sha3_256::new();
    hasher.update(sender.as_bytes());
    hasher.update(bridge.as_bytes());
    hasher.update(token.as_bytes());
    hasher.update(amount);
    hasher.update(hashlock);
    hasher.update(timelock.to_be_bytes());
    let result = hasher.finalize();
    let out: [u8; 32] = result.try_into().unwrap();
    out
}

#[derive(Serialize, Deserialize)]
pub struct ApiDataTransferOut {
    sender: AccountId,
    receiver: AccountId,
    token: TokenId,
    amount: [u8; 32],
    hashlock: [u8; 32],
    timelock: u64,
    dst_chain_id: u64,
    dst_address: [u8; 32],
    bid_id: u64,
    token_dst: [u8; 32],
    amount_dst: [u8; 32]
}

#[derive(Serialize, Deserialize)]
pub struct ApiDataTransferIn {
    sender: AccountId,
    receiver: AccountId,
    token: TokenId,
    token_amount: [u8; 32],
    hashlock: [u8; 32],
    timelock: u64,
    src_chain_id: u64,
    src_transfer_id: [u8; 32]
}

#[derive(Serialize, Deserialize)]
pub struct ApiDataTransferConfirm {
    sender: AccountId,
    receiver: AccountId,
    token: TokenId,
    token_amount: [u8; 32],
    hashlock: [u8; 32],
    timelock: u64,   
    preimage: [u8; 32]
}

#[derive(Serialize, Deserialize)]
pub struct ApiDataTransferRefund {
    sender: AccountId,
    receiver: AccountId,
    token: TokenId,
    token_amount: [u8; 32],
    hashlock: [u8; 32],
    timelock: u64
}

#[derive(Serialize, Deserialize)]
pub struct ApiData {
    api_type: String,
    data_transfer_out: Option<ApiDataTransferOut>,
    data_transfer_in: Option<ApiDataTransferIn>
}

#[ext_contract(ext_ft_contract)]
pub trait FungibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait TransferCallbackContract {
    fn on_confirm_transfer(&mut self, data: ApiDataTransferConfirm, amount: U128) -> U128;
    fn on_refund_transfer(&mut self, data: ApiDataTransferRefund, amount: U128) -> U128;
}

#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Owner {
    owner_id: AccountId
}

impl Owner {
    pub fn new(
        _owner_id: AccountId
    ) -> Self {
        let this = Self {
            owner_id: _owner_id
        };

        this
    }

    pub fn set(
        &mut self,
        _owner_id: AccountId
    ) {
        self.owner_id = _owner_id
    }

    pub fn is_owner(
        &mut self,
        account: AccountId
    ) -> bool {
        let matched = account.as_str() == self.owner_id.as_str();
        matched
    }
}

pub type TransferId = [u8; 32];

#[derive(BorshDeserialize, BorshSerialize, PartialEq, Debug)]
pub enum TransferStatus {
    Pending,
    Confirmed,
    Refunded,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct ObridgeContract {
    owner: Owner,
    transfers: HashMap<TransferId, TransferStatus>,
    basis_points_rate: U128,
    toll_address: AccountId,
    maximum_fee: HashMap<TokenId, U128>
}

#[near_bindgen]
impl ObridgeContract {

    #[init]
    pub fn new(
        owner_id: AccountId
    ) -> Self {

        let this = Self {
            owner: Owner::new(owner_id.clone()),
            transfers: HashMap::new(),
            basis_points_rate: U128::from(0),
            toll_address: owner_id.clone(),
            maximum_fee: HashMap::new()
        };

        this
    }

    pub fn set_basis_points_rate(
        &mut self,
        rate: U128
    ) {
        let sender_id = env::predecessor_account_id();
        if self.owner.is_owner(sender_id) {
            self.basis_points_rate = rate;
            log!("basis_points_rate updated to {}", self.basis_points_rate.0);
        }
    }

    pub fn set_toll_address(
        &mut self,
        toll_address: AccountId
    ) {
        let sender_id = env::predecessor_account_id();
        if self.owner.is_owner(sender_id) {
            self.toll_address = toll_address.clone();
            log!("toll_address updated to {}", toll_address);
        }
    }

    pub fn set_maximum_fee(
        &mut self,
        token: TokenId,
        fee: U128
    ) {
        let sender_id = env::predecessor_account_id();
        if self.owner.is_owner(sender_id) {
            self.maximum_fee.insert(token, fee);
        }
    }

    fn calc_fee(
        &mut self,
        token: &TokenId,
        value: Uint256
    ) -> Uint256 {
        let mut fee: Uint256 = Uint256(BigUint::from(0 as u128));
        let rate = Uint256(BigUint::from(self.basis_points_rate.0 as u128));
        let basis_points = Uint256(BigUint::from(10000 as u128));
        let rate_fee: Uint256 = value * rate / basis_points;


        if let Some(max_fee) = self.maximum_fee.get_mut(token) {
            let max_fee_value = Uint256(BigUint::from(max_fee.0 as u128));
            if rate_fee > max_fee_value {
                fee = max_fee_value;
            } else {
                fee = rate_fee;
            }
        } else {
            fee = rate_fee;
        }


        fee
    }

    fn transfer_out(
        &mut self,
        data: ApiDataTransferOut
    ) {
        let amount_value = Uint256::from_bytes_be(&data.amount);
        let dst_address_value = Uint256::from_bytes_be(&data.dst_address);
        let token_dst_value = Uint256::from_bytes_be(&data.token_dst);
        let amount_dst_value = Uint256::from_bytes_be(&data.amount_dst);

        let transfer_id = keccak256(&data.sender, &data.receiver, &data.token, &data.amount, &data.hashlock, &data.timelock);

        self.transfers.insert(transfer_id, TransferStatus::Pending);

        NearEvent::transfer_out(TransferOutData::new(
            &transfer_id,
            &data.sender,
            &data.receiver,
            &data.token,
            &amount_value,
            &data.hashlock,
            &data.timelock,
            &data.dst_chain_id,
            &dst_address_value,
            &data.bid_id,
            &token_dst_value,
            &amount_dst_value
        ))
        .emit();
    }

    fn transfer_in(
        &mut self,
        data: ApiDataTransferIn
    ) {
        let amount_value = Uint256::from_bytes_be(&data.token_amount);

        let transfer_id = keccak256(&data.sender, &data.receiver, &data.token, &data.token_amount, &data.hashlock, &data.timelock);

        self.transfers.insert(transfer_id, TransferStatus::Pending);

        NearEvent::transfer_in(TransferInData::new(
            &transfer_id,
            &data.sender,
            &data.receiver,
            &data.token,
            &amount_value,
            &data.hashlock,
            &data.timelock,
            &data.src_chain_id,
            &data.src_transfer_id,
        ))
        .emit();
    }

    pub fn transfer_confirm(
        &mut self,
        data: ApiDataTransferConfirm
    ) {
        let transfer_id = keccak256(&data.sender, &data.receiver, &data.token, &data.token_amount, &data.hashlock, &data.timelock);

        require!(*self.transfers.get(&transfer_id).unwrap() == TransferStatus::Pending, "not pending transfer");
        
        let mut preimage_vec = vec![];
        preimage_vec.extend_from_slice(&data.preimage);

        let mut hasher = Keccak256::new();
        hasher.update(preimage_vec.clone());
        let result = hasher.finalize();
        let out: [u8; 32] = result.try_into().unwrap();
        
        log!("hashlock: {}", hex::encode(data.hashlock));
        log!("hashlock math: {}", hex::encode(out));
        log!("preimage_vec: {}", hex::encode(preimage_vec));

        require!(hex::encode(data.hashlock) == hex::encode(out), "incorrect preimage");

        //send token
        let amount = Uint256::from_bytes_be(&data.token_amount);
        let fee = self.calc_fee(&data.token, amount.clone());

        let send_amount = U128::from((amount - fee.clone()).0.to_u128().unwrap());
        let send_fee = U128::from(fee.0.to_u128().unwrap());

        let token_account = AccountId::try_from(data.token.clone()).unwrap();
        ext_ft_contract::ext(token_account.clone())
            .with_attached_deposit(DEPOSIT_ONE_YOCTO)
            .ft_transfer(
                data.receiver.clone(),
                U128(send_amount.0 - send_fee.0),
                Some("Confirm bridge".to_string()),
            )
        .then(
            Self::ext(env::current_account_id())
            .with_static_gas(FT_HARVEST_CALLBACK_GAS)
            .on_confirm_transfer(
                data,
                U128(send_amount.0 - send_fee.0)
            )
        );

        if send_fee.0 > 0 {
            ext_ft_contract::ext(token_account.clone())
            .with_attached_deposit(DEPOSIT_ONE_YOCTO)
            .ft_transfer(
                self.toll_address.clone(),
                send_fee,
                Some("Confirm bridge".to_string()),
            );
        }
        
    }

    pub fn transfer_refund(
        &mut self,
        data: ApiDataTransferRefund
    ) {
        let transfer_id = keccak256(&data.sender, &data.receiver, &data.token, &data.token_amount, &data.hashlock, &data.timelock);

        require!(*self.transfers.get(&transfer_id).unwrap() == TransferStatus::Pending, "not pending transfer");
        require!(data.timelock <= env::block_timestamp(), "timelock not yet passed");

        let send_amount = U128::from(Uint256::from_bytes_be(&data.token_amount).0.to_u128().unwrap());

        // refund token
        ext_ft_contract::ext( AccountId::try_from(data.token.clone()).unwrap() )
            .with_attached_deposit(DEPOSIT_ONE_YOCTO)
            .ft_transfer(
                data.sender.clone(),
                send_amount,
                Some("Refund bridge".to_string()),
            )
        .then(
            Self::ext(env::current_account_id())
            .with_static_gas(FT_HARVEST_CALLBACK_GAS)
            .on_refund_transfer(
                data,
                send_amount
            )
        );
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for ObridgeContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        let api_data: ApiData = serde_json::from_str(&msg).unwrap();
        let mut rollback: PromiseOrValue<U128> = PromiseOrValue::Value(U128::from(0));
        let transfer_amount = Uint256(BigUint::from(amount.0 as u128));


        // log!("amount transfer:{} , amount in TxOutData:{}", self.basis_points_rate.0, Uint256::from_bytes_be(&api_data.data_transfer_out.unwrap().amount).0.to_u128().unwrap());

        if api_data.api_type == "transfer_out" {
            let data = api_data.data_transfer_out.unwrap();
            let need_amount = Uint256::from_bytes_be(&data.amount);
            if need_amount.le(&transfer_amount) {
                self.transfer_out(data);
                let ooa: BigUint = transfer_amount.0 - need_amount.0;
                rollback = PromiseOrValue::Value(U128::from(ooa.to_u128().unwrap()));
            } else {
                rollback = PromiseOrValue::Value(amount);
            }
        }
        else if api_data.api_type == "transfer_in" {
            let data = api_data.data_transfer_in.unwrap();
            let need_amount = Uint256::from_bytes_be(&data.token_amount);
            if need_amount.le(&transfer_amount) {
                self.transfer_in(data);
                let ooa: BigUint = transfer_amount.0 - need_amount.0;
                rollback = PromiseOrValue::Value(U128::from(ooa.to_u128().unwrap()));
            } else {
                rollback = PromiseOrValue::Value(amount);
            }
        }


        rollback
    }
}

#[near_bindgen]
impl TransferCallbackContract for ObridgeContract {
    
    fn on_confirm_transfer(
        &mut self,
        data: ApiDataTransferConfirm,
        amount: U128
    ) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULT");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"ERR_CALLBACK"),
            PromiseResult::Successful(_value) => {
                
                let transfer_id = keccak256(&data.sender, &data.receiver, &data.token, &data.token_amount, &data.hashlock, &data.timelock);
                
                self.transfers.insert(transfer_id, TransferStatus::Confirmed);

                NearEvent::transfer_confirmed(TransferConfirmedData::new(
                    &transfer_id,
                    &data.preimage,
                    ""
                ))
                .emit();

                amount
            }
        }
    }

    fn on_refund_transfer(
        &mut self,
        data: ApiDataTransferRefund,
        amount: U128
    ) -> U128 {
        assert_eq!(env::promise_results_count(), 1, "ERR_TOO_MANY_RESULT");
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"ERR_CALLBACK"),
            PromiseResult::Successful(_value) => {
                
                let transfer_id = keccak256(&data.sender, &data.receiver, &data.token, &data.token_amount, &data.hashlock, &data.timelock);
                
                self.transfers.insert(transfer_id, TransferStatus::Refunded);

                NearEvent::transfer_refunded(TransferRefundedData::new(
                    &transfer_id,
                    ""
                ))
                .emit();

                amount
            }
        }
    }
}