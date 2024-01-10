use std::cell::RefCell;

use candid::Principal;
use ic_cdk::storage;
use ic_cdk_macros::*;
use lib::{
    constants::ZERO_ADDRESS, ethereum::recover_address_from_eth_signature, RemittanceSubscriber,
};

use utils::{
    burn_tokens_from_caller, generate_burn_payload, generate_mint_payload,
    get_user_canister_balance, mint_tokens_to_caller,
};

pub mod constants;
pub mod utils;

// set the address of the token
thread_local! {
    static TOKEN_PRINCIPAL: RefCell<Option<Principal>> = RefCell::default();
}

#[update]
pub async fn mint(account: String, signature: String, amount: u128) {
    // validate the signature, which is a signature of the amount to be minted
    let recovered = recover_address_from_eth_signature(signature, amount.to_string()).unwrap();
    if recovered.to_lowercase() != account.to_lowercase() {
        panic!(
            "SIGNATURE_VERIFICATION_FAILED:recovered key: {}; public key:{}",
            recovered, account
        );
    }
    // validate the signature, which is a signature of the amount to be minted

    let dc_canister: Principal = ic_cdk::id();
    let caller = ic_cdk::caller();

    let user_balance = get_user_canister_balance(account.clone()).await;
    if user_balance < amount {
        panic!("INSUFFICIENT_FUNDS")
    }

    // deduct from the 'amount' user's balance and add it to the canister's balance
    let remittance_payload = generate_mint_payload(account.clone(), amount).await;
    lib::dc::update_remittance_canister(&remittance_payload, &dc_canister)
        .await
        .unwrap();

    // mint them some ccmatic tokens
    mint_tokens_to_caller(amount, &caller).await;
}

#[update]
pub async fn burn(account: String, signature: String, amount: u128) {
    // validate the signature, which is a signature of the amount to be minted
    let recovered = recover_address_from_eth_signature(signature, amount.to_string()).unwrap();
    if recovered.to_lowercase() != account.to_lowercase() {
        panic!(
            "SIGNATURE_VERIFICATION_FAILED:recovered key: {}; public key:{}",
            recovered, account
        );
    }
    // validate the signature, which is a signature of the amount to be minted

    let dc_canister: Principal = ic_cdk::id();
    let caller = ic_cdk::caller();

    // first try to burn
    burn_tokens_from_caller(amount, &caller).await.unwrap();

    // add the 'amount' to the user's balance and add it to the canister's balance
    let remittance_payload = generate_burn_payload(account.clone(), amount).await;
    lib::dc::update_remittance_canister(&remittance_payload, &dc_canister)
        .await
        .unwrap();

    // mint them some ccmatic tokens
    mint_tokens_to_caller(amount, &caller).await;
}

// @dev testing command
#[query]
fn name() -> String {
    format!("bridge_data_collection canister")
}

#[init]
async fn init() {
    lib::owner::init_owner();
}

#[query]
fn owner() -> String {
    lib::owner::get_owner()
}

#[update]
pub async fn set_remittance_canister(remittance_principal: Principal) {
    lib::owner::only_owner();

    lib::dc::set_remittance_canister(remittance_principal);
}

#[query]
pub fn get_remittance_canister() -> RemittanceSubscriber {
    // confirm at least one remittance canister is subscribed to this pdc
    lib::dc::get_remittance_canister()
}

// this function is going to be called by the remittance canister
// so it can recieve "publish" events from this canister
#[update]
fn subscribe() {
    // verify if this remittance canister has been whitelisted
    // set the subscribed value to true if its the same, otherwise panic
    lib::dc::subscribe();
}

#[query]
fn is_subscribed(canister_principal: Principal) -> bool {
    lib::dc::is_subscribed(canister_principal)
}

#[update]
fn set_token_principal(token_canister_principal: Principal) {
    utils::set_token_principal(token_canister_principal)
}

#[query]
fn get_token_principal() -> Principal {
    utils::get_token_principal()
}

#[update]
async fn get_user_balance(account_address: String) -> u128 {
    get_user_canister_balance(account_address).await
}

#[update]
async fn get_canister_balance() -> u128 {
    get_user_canister_balance(ZERO_ADDRESS.to_string()).await
}

// we would use this method to publish data to the subscriber
// which would be the remittance model
// so when we have some new data, we would publish it to the remittance model
#[update]
async fn manual_publish(json_data: String) {
    // create a dummy remittance object we can publish until we implement data collection
    // which would then generate the data instead of hardcoding it
    let _ = lib::dc::publish_json_to_remittance(json_data);
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store = lib::dc::REMITTANCE_CANISTER.with(|rc| rc.borrow().clone());
    let cloned_token_principal = TOKEN_PRINCIPAL.with(|rc| rc.borrow().clone());
    storage::stable_save((cloned_store, cloned_token_principal)).unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    init().await;

    let (old_store, cloned_token_principal): (
        Option<lib::RemittanceSubscriber>,
        Option<Principal>,
    ) = storage::stable_restore().unwrap();
    lib::dc::REMITTANCE_CANISTER.with(|store| *store.borrow_mut() = old_store);
    TOKEN_PRINCIPAL.with(|store| *store.borrow_mut() = cloned_token_principal);
}
// --------------------------- upgrade hooks ------------------------- //
