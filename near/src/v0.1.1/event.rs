extern crate hex;

use num256::uint256::Uint256;
use near_sdk::AccountId;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Serialize, Debug)]
#[serde(tag = "standard")]
#[serde(rename_all = "snake_case")]
pub enum NearEvent<'a> {
    #[serde(borrow)]
    Obridge(ObridgeEvent<'a>),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObridgeEvent<'a> {
    pub version: &'static str,
    #[serde(flatten)]
    #[serde(borrow)]
    pub event_kind: ObridgeEventKind<'a>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
pub enum ObridgeEventKind<'a> {
    #[serde(borrow)]
    TransferOut(TransferOutData<'a>),
    #[serde(borrow)]
    TransferIn(TransferInData<'a>),
    #[serde(borrow)]
    TransferConfirmed(TransferConfirmedData<'a>),
    #[serde(borrow)]
    TransferRefunded(TransferRefundedData<'a>),
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferOutData<'a> {
   
    pub transfer_id: String,
    #[serde(borrow)]
    pub sender: &'a str,
    #[serde(borrow)]
    pub receiver: &'a str,
    #[serde(borrow)]
    pub token: &'a str,

    pub amount: String,
    
    pub hashlock: String,
    
    pub timelock: String,
    
    pub dst_chain_id: String,
    
    pub dst_address: String,
    
    pub bid_id: String,
    
    pub token_dst: String,
    
    pub amount_dst: String,
}

impl<'a> TransferOutData<'a> {
    pub fn new(
        transfer_id: &'a [u8; 32],
        sender: &'a AccountId,
        receiver: &'a AccountId,
        token: &'a str,
        amount: &'a Uint256,
        hashlock: &'a [u8; 32],
        timelock: &'a u64,
        dst_chain_id: &'a u64,
        dst_address: &'a Uint256,
        bid_id: &'a u64,
        token_dst: &'a Uint256,
        amount_dst: &'a Uint256
    ) -> TransferOutData<'a> {
        Self {
            transfer_id: hex::encode(transfer_id),
            sender: sender.as_str(),
            receiver: receiver.as_str(),
            token,
            amount: amount.to_str_radix(10),
            hashlock: hex::encode(hashlock),
            timelock: timelock.to_string(),
            dst_chain_id: dst_chain_id.to_string(),
            dst_address: dst_address.to_str_radix(10),
            bid_id: bid_id.to_string(),
            token_dst: token_dst.to_str_radix(10),
            amount_dst: amount_dst.to_str_radix(10)
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferInData<'a> {

    pub transfer_id: String,
    #[serde(borrow)]
    pub sender: &'a str,
    #[serde(borrow)]
    pub receiver: &'a str,
    #[serde(borrow)]
    pub token: &'a str,
    
    pub token_amount: String,

    pub hashlock: String,
    
    pub timelock: String,

    pub src_chain_id: String,
    
    pub src_transfer_id: String
}

impl<'a> TransferInData<'a> {
    pub fn new(
        transfer_id: &'a [u8; 32],
        sender: &'a AccountId,
        receiver: &'a AccountId,
        token: &'a str,
        token_amount: &'a Uint256,
        hashlock: &'a [u8; 32],
        timelock: &'a u64,
        src_chain_id: &'a u64,
        src_transfer_id:  &'a [u8; 32]
    ) -> TransferInData<'a> {
        Self {
            transfer_id: hex::encode(transfer_id),
            sender: sender.as_str(),
            receiver: receiver.as_str(),
            token,
            token_amount: token_amount.to_str_radix(10),
            hashlock: hex::encode(hashlock),
            timelock: timelock.to_string(),
            src_chain_id: src_chain_id.to_string(),
            src_transfer_id: hex::encode(src_transfer_id),
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferConfirmedData<'a> {

    pub transfer_id: String,

    pub preimage: String,

    #[serde(borrow)]
    pub msg: &'a str,
}

impl<'a> TransferConfirmedData<'a> {
    pub fn new(
        transfer_id: &'a [u8; 32],
        preimage: &'a [u8; 32],
        msg: &'a str,
    ) -> TransferConfirmedData<'a> {
        Self {
            transfer_id: hex::encode(transfer_id),
            preimage: hex::encode(preimage),
            msg
        }
    }
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Debug)]
pub struct TransferRefundedData<'a> {
    pub transfer_id: String,

    #[serde(borrow)]
    pub msg: &'a str,
}

impl<'a> TransferRefundedData<'a> {
    pub fn new(
        transfer_id: &'a [u8; 32],
        msg: &'a str,
    ) -> TransferRefundedData<'a> {
        Self {
            transfer_id: hex::encode(transfer_id),
            msg
        }
    }
}

impl<'a> NearEvent<'a> {

    pub fn new_obridge(version: &'static str, event_kind: ObridgeEventKind<'a>) -> Self {
        NearEvent::Obridge(ObridgeEvent {
            version,
            event_kind,
        })
    }

    pub fn new_obridge_v1(event_kind: ObridgeEventKind<'a>) -> Self {
        NearEvent::new_obridge("1.0.0", event_kind)
    }

    #[must_use = "don't forget to .emit() the event"]
    pub fn transfer_out(data: TransferOutData<'a>) -> Self {
        NearEvent::new_obridge_v1(ObridgeEventKind::TransferOut(data))
    }

    #[must_use = "don't forget to .emit() the event"]
    pub fn transfer_in(data: TransferInData<'a>) -> Self {
        NearEvent::new_obridge_v1(ObridgeEventKind::TransferIn(data))
    }

    #[must_use = "don't forget to .emit() the event"]
    pub fn transfer_confirmed(data: TransferConfirmedData<'a>) -> Self {
        NearEvent::new_obridge_v1(ObridgeEventKind::TransferConfirmed(data))
    }

    #[must_use = "don't forget to .emit() the event"]
    pub fn transfer_refunded(data: TransferRefundedData<'a>) -> Self {
        NearEvent::new_obridge_v1(ObridgeEventKind::TransferRefunded(data))
    }

    pub(crate) fn to_json_string(&self) -> String {
        near_sdk::serde_json::to_string(self).unwrap()
    }

    pub fn to_json_event_string(&self) -> String {
        format!("EVENT_JSON:{}", self.to_json_string())
    }

    /// Logs the event to the host. This is required to ensure that the event is triggered
    /// and to consume the event.
    pub fn emit(self) {
        near_sdk::env::log_str(&self.to_json_event_string());
    }
}
