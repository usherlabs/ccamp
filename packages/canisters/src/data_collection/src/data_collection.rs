use candid::{CandidType, Principal};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, CandidType)]
pub struct DataModel {
    pub ticker: String,
    pub chain_id: String,
    pub chain_name: String,
    pub recipient_address: String,
    pub amount: u64,
    pub chain: Chain,
}

#[derive(Clone, Debug, Deserialize, CandidType)]
pub enum Chain {
    Ethereum1,
    Polygon137,
    Icp,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Subscriber {
    pub topic: String,
}

pub type SubscriberStore = BTreeMap<Principal, Subscriber>;
