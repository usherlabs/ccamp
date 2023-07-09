use candid::{CandidType, Principal};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Display};

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Hash, Eq)]
pub struct DataModel {
    pub ticker: String,
    pub recipient_address: String,
    pub amount: i64,
    pub chain: Chain,
}

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Hash, Eq)]
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

impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ethereum1 => write!(f, "ethereum:1"),
            Self::Polygon137 => write!(f, "polygon:137"),
            Self::Icp => write!(f, "icp"),
        }
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Subscriber {
    pub topic: String,
}

pub type SubscriberStore = BTreeMap<Principal, Subscriber>;
