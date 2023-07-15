// define all major types and their implementation here

// TODO VALIDATE incoming remittance requests
// ----- Make sure everyone has valid balances for deductions i.e negative adjustments
// ----- Make sure the net, adjustment is zero i.e make sure no balance is created nor destroyed,
// ----- Only moved from one place to the other
#![allow(dead_code)]
use crate::{owner, utils};
use candid::{CandidType, Error};
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

pub type AvailableBalanceStore = HashMap<(lib::Wallet, lib::Chain, lib::Wallet), Account>;
pub type WitheldBalanceStore = HashMap<(lib::Wallet, lib::Chain, lib::Wallet, u64), WitheldAccount>;
pub type WitheldAmountsStore = HashMap<(lib::Wallet, lib::Chain, lib::Wallet), Vec<u64>>;

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

// given some details, which are the parameters of the function
// we want to get the balance signature generated when a remit request created by this account
// it would return a balance of 0 and no signature if a user has not made a remit request for the specified "amount"
pub fn get_remitted_balance(
    token: lib::Wallet,
    chain: lib::Chain,
    account: lib::Wallet,
    amount: u64,
) -> WitheldAccount {
    let witheld_amount = crate::WITHELD_REMITTANCE.with(|witheld| {
        let existing_key = (token, chain, account.clone(), amount);
        witheld
            .borrow()
            .get(&existing_key)
            .cloned()
            .unwrap_or_default()
    });

    witheld_amount
}

// get the total unspent available-to-use balance for the user
pub fn get_available_balance(
    token: lib::Wallet,
    chain: lib::Chain,
    account: lib::Wallet,
) -> Account {
    let available_amount = crate::REMITTANCE.with(|remittance| {
        let existing_key = (token, chain, account);
        remittance
            .borrow()
            .get(&existing_key)
            .cloned()
            .unwrap_or_default()
    });

    available_amount
}

// it essentially uses the mapping (ticker, chain, recipientaddress) => {DataModel}
// so if an entry exists for a particular combination of (ticker, chain, recipientaddress)
// then the price is updated, otherwise the entry is created
pub fn update_balance(new_remittance: lib::DataModel) {
    owner::only_publisher();
    crate::REMITTANCE.with(|remittance| {
        let mut remittance_store = remittance.borrow_mut();

        let hash_key = (
            new_remittance.token.clone(),
            new_remittance.chain.clone(),
            new_remittance.account.clone(),
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

// use the right validator depending on if the caller is a pdc or not
pub fn validate_remittance_data(
    is_pdc: bool,
    new_remittances: &Vec<lib::DataModel>,
) -> Result<(), String> {
    match is_pdc {
        true => Ok(()),
        false => validate_dc_remitance_data(new_remittances),
    }
}

// validate data for an ordinary dc canister
pub fn validate_dc_remitance_data(new_remittances: &Vec<lib::DataModel>) -> Result<(), String> {
    // validate that all operations are adjust and the resultant of amounts is zero
    let amount_delta = new_remittances
        .iter()
        .fold(0, |acc, account| acc + account.amount);

    if amount_delta != 0 {
        return Err("SUM_AMOUNT != 0".to_string());
    }

    // validate it is only adjust action provided
    let is_action_valid = new_remittances
        .iter()
        .all(|item| item.action == lib::Action::Adjust);

    if !is_action_valid {
        return Err("INVALID_ACTION_FOUND".to_string());
    }

    // check for all the negative deductions and confirm that the owners have at least that much balance
    let mut sufficient_balance_error: Result<(), String> = Ok(());
    new_remittances
        .iter()
        .filter(|item| item.amount < 0)
        .for_each(|item| {
            let existing_balance =
                get_available_balance(item.token.clone(), item.chain.clone(), item.account.clone());
            if existing_balance.balance < item.amount.abs() as u64 {
                sufficient_balance_error = Err("INSUFFICIENT_USER_BALANCE".to_string());
            };
        });

    
    sufficient_balance_error
}
