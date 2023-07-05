// define all major types and their implementation here
#![warn(dead_code)]
use crate::utils;
use candid::CandidType;
use eth_encode_packed::{
    ethabi::{ethereum_types::U256, Address},
    SolidityDataType,
};
use lib;
use rand::{Rng, rngs::StdRng};
use serde_derive::Deserialize;
use std::{collections::HashMap, cell::RefCell};

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub balance: u64,
}

thread_local! {
    static RNG: RefCell<Option<StdRng>> = RefCell::new(None);
}

pub type Store = HashMap<(String, lib::Chain, String), Account>;

// generate a nonce which ranges between 0 and 2^(64 - 1)
pub fn generate_nonce() -> u64 {
    let mut rng = rand::thread_rng();
    rng.gen_range(0..=u64::MAX)
}


// this is equivalent to a function which produces abi.encodePacked(nonce, amount, address)
pub fn produce_remittance_hash(nonce: u64, amount: u64, address: &str) -> (Vec<u8>, String) {
    // convert the address to bytes format
    let address: [u8; 20] = utils::string_to_vec_u8(address).try_into().unwrap();
    // pack the encoded bytes
    let input = vec![
        SolidityDataType::Number(U256::from(nonce)),
        SolidityDataType::Number(U256::from(amount)),
        SolidityDataType::Address(Address::from(address)),
    ];
    let (_bytes, hash) = eth_encode_packed::abi::encode_packed(&input);

    (_bytes, hash)
}
