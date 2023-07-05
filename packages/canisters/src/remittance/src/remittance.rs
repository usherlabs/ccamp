// define all major types and their implementation here
#![warn(dead_code)]
use candid::CandidType;
use lib;
use rand::Rng;
use serde_derive::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub balance: u64,
}

pub type Store = HashMap<(String, lib::Chain, String), Account>;

// generate a nonce which ranges between 0 and 2^(64 - 1)
pub fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..=u64::MAX)
}
