// define all major types and their implementation here
#![warn(dead_code)]
use crate::{owner, utils};
use candid::CandidType;
use eth_encode_packed::{
    ethabi::{ethereum_types::U256, Address},
    SolidityDataType,
};
use lib;
use rand::rngs::StdRng;
use serde_derive::Deserialize;
use std::{cell::RefCell, collections::HashMap};

pub const MAX_CYCLE: u64 = 25_000_000_000;
thread_local! {
    static RNG: RefCell<Option<StdRng>> = RefCell::new(None);
}
#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub balance: u64,
}
impl Default for Account {
    fn default() -> Self {
        return Self { balance: 0 };
    }
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct WitheldAccount {
    pub balance: u64,
    pub signature: String,
    pub nonce: u64,
}
impl Default for WitheldAccount {
    fn default() -> Self {
        return Self {
            balance: 0,
            signature: String::from(""),
            nonce: 0,
        };
    }
}

#[derive(CandidType, Deserialize, Debug)]
pub struct RemittanceReply {
    pub hash: String,
    pub signature: String,
    pub nonce: u64,
    pub amount: u64,
}

pub type AvailableBalanceStore = HashMap<(String, lib::Chain, String), Account>;
pub type WitheldBalanceStore = HashMap<(String, lib::Chain, String, u64), WitheldAccount>;
pub type WitheldAmountsStore = HashMap<(String, lib::Chain, String), Vec<u64>>;

// this is equivalent to a function which produces abi.encodePacked(nonce, amount, address)
pub fn produce_remittance_hash(
    nonce: u64,
    amount: u64,
    address: &str,
    chain_id: &str,
) -> (Vec<u8>, String) {
    // convert the address to bytes format
    let address: [u8; 20] = utils::string_to_vec_u8(address).try_into().unwrap();
    // pack the encoded bytes
    let input = vec![
        SolidityDataType::Number(U256::from(nonce)),
        SolidityDataType::Number(U256::from(amount)),
        SolidityDataType::Address(Address::from(address)),
        SolidityDataType::String(chain_id),
    ];
    let (_bytes, hash) = eth_encode_packed::abi::encode_packed(&input);

    (_bytes, hash)
}

pub fn get_remitted_balance(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
    amount: u64,
) -> WitheldAccount {
    // validate the address and the chain
    if recipient_address.len() != 42 {
        panic!("INVALID_ADDRESS")
    };
    let chain = lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");
    // validate the address and the chain

    let witheld_amount = crate::WITHELD_REMITTANCE.with(|witheld| {
        let existing_key = (ticker, chain, recipient_address.clone(), amount);
        witheld
            .borrow()
            .get(&existing_key)
            .cloned()
            .unwrap_or_default()
    });

    witheld_amount
}

// it essentially uses the mapping (ticker, chain, recipientaddress) => {DataModel}
// so if an entry exists for a particular combination of (ticker, chain, recipientaddress)
// then the price is updated, otherwise the entry is created
pub fn update_balance(new_remittance: lib::DataModel) {
    owner::only_publisher();
    crate::REMITTANCE.with(|remittance| {
        let mut remittance_store = remittance.borrow_mut();

        let hash_key = (
            new_remittance.ticker.clone(),
            new_remittance.chain.clone(),
            new_remittance.recipient_address.clone(),
        );

        if let Some(existing_data) = remittance_store.get_mut(&hash_key) {
            existing_data.balance =
                (existing_data.balance as i64 + new_remittance.amount as i64) as u64;
        } else {
            remittance_store.insert(
                hash_key,
                Account {
                    balance: new_remittance.amount as u64,
                },
            );
        }
    });
}
