// define all major types and their implementation here
use candid::CandidType;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Eq, Hash)]
pub enum Chain {
    Ethereum1,
    Polygon137,
    Icp,
}

impl Chain {
    pub fn from_chain_details(chain_name: &str, chain_id: &str) -> Option<Chain> {
        let lowercase_chain_name = &chain_name.to_lowercase()[..];
        match (lowercase_chain_name, chain_id) {
            ("ethereum", "1") => Some(Chain::Ethereum1),
            ("polygon", "137") => Some(Chain::Polygon137),
            ("icp", _) => Some(Chain::Icp),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct DataModel {
    pub ticker: String,
    pub chain_id: String,
    pub chain_name: String,
    pub recipient_address: String,
    pub amount: u64,
    pub chain: Chain,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Subscriber {
    pub topic: String,
}

pub type Store = HashMap<(String, Chain, String), DataModel>;
