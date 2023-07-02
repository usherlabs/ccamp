use candid::CandidType;
// define all major types and their implementation here
use lib;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub balance: u64,
}

pub type Store = HashMap<(String, lib::Chain, String), Account>;
