#![allow(dead_code)]

use candid::{CandidType, Principal};
use serde::Deserialize;
use std::{collections::BTreeMap, fmt::Display};

pub mod constants;
pub mod utils;
pub mod dc;
pub mod owner;

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Hash, Eq)]
pub struct Wallet {
    pub address: Vec<u8>,
}
impl TryFrom<String> for Wallet {
    type Error = String;
    fn try_from(address: String) -> Result<Self, Self::Error> {
        let starts_from: usize;
        if address.starts_with("0x") {
            starts_from = 2;
        } else {
            starts_from = 0;
        }

        let result = (starts_from..address.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&address[i..i + 2], 16).unwrap())
            .collect::<Vec<u8>>();

        if result.len() != 20 {
            Err(String::from("INVALID_ADDRESSS_LENGTH"))
        } else {
            Ok(Self { address: result })
        }
    }
}
impl Display for Wallet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string_address = self
            .address
            .iter()
            .map(|r| format!("{:02x}", r))
            .collect::<Vec<String>>()
            .join("")
            .to_string();

        write!(f, "0x{}", string_address)
    }
}

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Hash, Eq)]
pub enum Action {
    Adjust,
    Deposit,
    Withdraw,
    CancelWithdraw,
}

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Hash, Eq)]
pub struct DataModel {
    pub token: Wallet,
    pub chain: Chain,
    pub amount: i64,
    pub account: Wallet,
    pub action: Action,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Subscriber {
    pub topic: String,
}

#[derive(Clone, Debug, Deserialize, CandidType, PartialEq, Hash, Eq)]
pub enum Chain {
    Ethereum1,
    Ethereum5,
    Polygon137,
    Icp,
}
impl TryFrom<String> for Chain {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let details: Vec<&str> = value.split(':').collect();

        let chain_name = details[0];
        let chain_id = details[1];

        let lowercase_chain_name = &chain_name.to_lowercase()[..];
        match (lowercase_chain_name, chain_id) {
            ("ethereum", "1") => Ok(Chain::Ethereum1),
            ("ethereum", "5") => Ok(Chain::Ethereum5),
            ("polygon", "137") => Ok(Chain::Polygon137),
            ("icp", _) => Ok(Chain::Icp),
            _ => Err(String::from("INVALID CHAIN")),
        }
    }
}
impl Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ethereum1 => write!(f, "ethereum:1"),
            Self::Ethereum5 => write!(f, "ethereum:5"),
            Self::Polygon137 => write!(f, "polygon:137"),
            Self::Icp => write!(f, "icp"),
        }
    }
}
impl Chain {
    fn get_chain_details(&self) -> (String, String) {
        let chain_string = self.to_string();
        let chain_details: Vec<&str> = chain_string.split(":").collect();

        (
            String::from(chain_details[0]),
            String::from(*chain_details.get(1).unwrap_or(&"")),
        )
    }
}

pub type SubscriberStore = BTreeMap<Principal, Subscriber>;
