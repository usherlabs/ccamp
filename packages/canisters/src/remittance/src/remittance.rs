// define all major types and their implementation here

// TODO VALIDATE incoming remittance requests
// ----- Make sure everyone has valid balances for deductions i.e negative adjustments
// ----- Make sure the net, adjustment is zero i.e make sure no balance is created nor destroyed,
// ----- Only moved from one place to the other
#![allow(dead_code)]
use crate::utils;
use candid::{CandidType, Principal};
use easy_hasher::easy_hasher;
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
pub struct WithheldAccount {
    pub balance: u64,
    pub signature: String,
    pub nonce: u64,
}
impl Default for WithheldAccount {
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

pub type AvailableBalanceStore =
    HashMap<(lib::Wallet, lib::Chain, lib::Wallet, Principal), Account>;
pub type WithheldBalanceStore =
    HashMap<(lib::Wallet, lib::Chain, lib::Wallet, Principal, u64), WithheldAccount>;
pub type WithheldAmountsStore =
    HashMap<(lib::Wallet, lib::Chain, lib::Wallet, Principal), Vec<u64>>;

// this is equivalent to a function which produces abi.encodePacked(nonce, amount, address)
pub fn hash_remittance_parameters(
    nonce: u64,
    amount: u64,
    address: &str,
    chain_id: &str,
    dc_canister_id: &str,
    token_address: &str,
) -> Vec<u8> {
    // convert the address to bytes format
    let address: [u8; 20] = utils::string_to_vec_u8(address).try_into().unwrap();
    let token_address: [u8; 20] = utils::string_to_vec_u8(token_address).try_into().unwrap();

    // pack the encoded bytes
    let input = vec![
        SolidityDataType::Number(U256::from(nonce)),
        SolidityDataType::Number(U256::from(amount)),
        SolidityDataType::Address(Address::from(address)),
        SolidityDataType::String(chain_id),
        SolidityDataType::String(dc_canister_id),
        SolidityDataType::Address(Address::from(token_address)),
    ];
    let (_bytes, __) = eth_encode_packed::abi::encode_packed(&input);

    easy_hasher::raw_keccak256(_bytes.clone()).to_vec()
}

// given some details, which are the parameters of the function
// we want to get the balance signature generated when a remit request created by this account
// it would return a balance of 0 and no signature if a user has not made a remit request for the specified "amount"
pub fn get_remitted_balance(
    token: lib::Wallet,
    chain: lib::Chain,
    account: lib::Wallet,
    dc_canister: Principal,
    amount: u64,
) -> WithheldAccount {
    let withheld_amount = crate::WITHHELD_REMITTANCE.with(|withheld| {
        let existing_key = (token, chain, account.clone(), dc_canister, amount);
        withheld
            .borrow()
            .get(&existing_key)
            .cloned()
            .unwrap_or_default()
    });

    withheld_amount
}

// get the total unspent available-to-use balance for the user
pub fn get_available_balance(
    token: lib::Wallet,
    chain: lib::Chain,
    account: lib::Wallet,
    dc_canister: Principal,
) -> Account {
    let available_amount = crate::REMITTANCE.with(|remittance| {
        let existing_key = (token, chain, account, dc_canister);
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
pub fn update_balance(new_remittance: lib::DataModel, dc_canister: Principal) {
    crate::REMITTANCE.with(|remittance| {
        let mut remittance_store = remittance.borrow_mut();

        let hash_key = (
            new_remittance.token.clone(),
            new_remittance.chain.clone(),
            new_remittance.account.clone(),
            dc_canister.clone(),
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

pub fn confirm_withdrawal(
    token: String,
    chain: String,
    account: String,
    amount_withdrawn: u64,
    dc_canister: Principal,
) -> bool {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let hash_key = (
        token.clone(),
        chain.clone(),
        account.clone(),
        dc_canister.clone(),
    );

    // go through the witheld amounts and remove this amount from it
    crate::WITHHELD_AMOUNTS.with(|witheld_amounts| {
        let mut mut_witheld_amounts = witheld_amounts.borrow_mut();
        let unwithdrawn_amounts = mut_witheld_amounts
            .get(&hash_key)
            .expect("AMOUNT_NOT_WITHELD")
            .into_iter()
            .filter(|&amount_to_withdraw| *amount_to_withdraw != amount_withdrawn)
            .cloned()
            .collect();
        mut_witheld_amounts.insert(hash_key, unwithdrawn_amounts);
    });

    // go through the witheld balance store and remove this amount from it
    crate::WITHHELD_REMITTANCE.with(|withheld_remittance| {
        withheld_remittance.borrow_mut().remove(&(
            token.clone(),
            chain.clone(),
            account.clone(),
            dc_canister.clone(),
            amount_withdrawn,
        ));
    });

    return true;
}

pub fn cancel_withdrawal(
    token: String,
    chain: String,
    account: String,
    amount_canceled: u64,
    dc_canister: Principal,
) -> bool {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let hash_key = (
        token.clone(),
        chain.clone(),
        account.clone(),
        dc_canister.clone(),
    );

    // go through the witheld amounts and remove this amount from it
    crate::WITHHELD_AMOUNTS.with(|witheld_amounts| {
        let mut mut_witheld_amounts = witheld_amounts.borrow_mut();
        let unwithdrawn_amounts = mut_witheld_amounts
            .get(&hash_key)
            .expect("AMOUNT_NOT_WITHELD")
            .into_iter()
            .filter(|&amount_to_withdraw| *amount_to_withdraw != amount_canceled)
            .cloned()
            .collect();
        mut_witheld_amounts.insert(hash_key.clone(), unwithdrawn_amounts);
    });

    // go through the witheld balance store and remove this amount from it
    crate::WITHHELD_REMITTANCE.with(|withheld_remittance| {
        withheld_remittance.borrow_mut().remove(&(
            token.clone(),
            chain.clone(),
            account.clone(),
            dc_canister.clone(),
            amount_canceled,
        ));
    });

    // add the withheld total back to the available balance
    crate::REMITTANCE.with(|remittance| {
        if let Some(existing_data) = remittance.borrow_mut().get_mut(&hash_key) {
            existing_data.balance = existing_data.balance + amount_canceled;
        }
    });

    return true;
}


// use the right validator depending on if the caller is a pdc or not
pub fn validate_remittance_data(
    is_pdc: bool,
    new_remittances: &Vec<lib::DataModel>,
    dc_canister: Principal,
) -> Result<(), String> {
    match is_pdc {
        true => validate_pdc_remittance_data(new_remittances, dc_canister),
        false => validate_dc_remittance_data(new_remittances, dc_canister),
    }
}

pub fn validate_pdc_remittance_data(
    new_remittances: &Vec<lib::DataModel>,
    dc_canister: Principal,
) -> Result<(), String> {
    // validate that all adjust operations lead to a sum of zero
    let adjust_operations: Vec<lib::DataModel> = new_remittances
        .into_iter()
        .filter(|&single_remittance| single_remittance.action == lib::Action::Adjust)
        .cloned()
        .collect();
    // apply the same validation of dc canisters to the adjust operations of a pdc canister
    if let Err(err_message) = validate_dc_remittance_data(&adjust_operations, dc_canister) {
        return Err(err_message);
    };

    // validate that all operations that are not "adjust" operations are positive amounts
    // other than adjusts we currently have no use for negative amounts operations
    // this can be later changed
    let non_adjust_operations_gt_0: Vec<&lib::DataModel> = new_remittances
        .into_iter()
        .filter(|single_remittance| {
            single_remittance.action != lib::Action::Adjust && single_remittance.amount < 0
        })
        .collect();
    if non_adjust_operations_gt_0.len() > 0 {
        return Err("NON_ADJUST_AMOUNT_MUST_BE_GT_0".to_string());
    };

    Ok(())
}

// validate data for an ordinary dc canister
pub fn validate_dc_remittance_data(
    new_remittances: &Vec<lib::DataModel>,
    dc_canister: Principal,
) -> Result<(), String> {
    // validate that all operations are adjust and the resultant of amounts is zero
    let amount_delta = new_remittances
        .iter()
        .fold(0, |acc, account| acc + account.amount);

    if amount_delta != 0 {
        return Err("SUM_ADJUST_AMOUNTS != 0".to_string());
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
        .filter(|&item| item.amount < 0)
        .for_each(|item| {
            let existing_balance = get_available_balance(
                item.token.clone(),
                item.chain.clone(),
                item.account.clone(),
                dc_canister.clone(),
            );
            if existing_balance.balance < item.amount.abs() as u64 {
                sufficient_balance_error = Err("INSUFFICIENT_USER_BALANCE".to_string());
            };
        });

    sufficient_balance_error
}
