use std::collections::HashMap;

use candid::Principal;
use ic_cdk::storage;
use ic_cdk_macros::*;
use verity_dp_ic::{
    crypto::ecdsa::PublicKeyReply,
    owner,
    remittance::{
        self, config::Environment, random, types::{Account, DataModel, RemittanceReciept, RemittanceReply}, utils::{self, only_whitelisted_dc_canister}
    },
};

#[init]
fn init(env_opt: Option<Environment>) {
    verity_dp_ic::remittance::init(env_opt)
}

#[query]
fn name() -> String {
    verity_dp_ic::remittance::name()
}

#[query]
fn owner() -> String {
    verity_dp_ic::remittance::owner()
}

#[update]
async fn subscribe_to_dc(canister_id: Principal) {
    owner::only_owner();
    verity_dp_ic::remittance::subscribe_to_dc(canister_id).await
}

#[update]
async fn subscribe_to_pdc(pdc_canister_id: Principal) {
    owner::only_owner();
    verity_dp_ic::remittance::subscribe_to_pdc(pdc_canister_id).await
}

// TODO: Investigate if we need to add this to the candid file for this canister
#[update]
fn update_remittance(
    new_remittances: Vec<DataModel>,
    dc_canister: Principal,
) -> Result<(), String> {
    utils::only_whitelisted_dc_canister();
    verity_dp_ic::remittance::update_remittance(new_remittances, dc_canister)
}

#[update]
async fn remit(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
    amount: u64,
    proof: String,
) -> Result<RemittanceReply, String> {
    verity_dp_ic::remittance::remit(token, chain, account, dc_canister, amount, proof)
        .await
        .map_err(|e| e.to_string())
}

#[query]
fn get_available_balance(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
) -> Result<Account, String> {
    verity_dp_ic::remittance::get_available_balance(token, chain, account, dc_canister)
        .map_err(|e| e.to_string())
}

#[query]
fn get_canister_balance(
    token: String,
    chain: String,
    dc_canister: Principal,
) -> Result<Account, String> {
    verity_dp_ic::remittance::get_canister_balance(token, chain, dc_canister)
        .map_err(|e| e.to_string())
}

#[query]
fn get_withheld_balance(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
) -> Result<Account, String> {
    verity_dp_ic::remittance::get_withheld_balance(token, chain, account, dc_canister)
        .map_err(|e| e.to_string())
}

#[query]
fn get_reciept(dc_canister: Principal, nonce: u64) -> Result<RemittanceReciept, String> {
    verity_dp_ic::remittance::get_reciept(dc_canister, nonce).map_err(|e| e.to_string())
}

#[update]
async fn public_key() -> Result<PublicKeyReply, String> {
    verity_dp_ic::remittance::public_key()
        .await
        .map_err(|e| e.to_string())
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    // clone all important variables
    let cloned_available_balance_store = remittance::state::REMITTANCE.with(|store| store.borrow().clone());
    let cloned_witheld_balance_store = remittance::state::WITHHELD_REMITTANCE.with(|store| store.borrow().clone());
    let cloned_witheld_amounts = remittance::state::WITHHELD_AMOUNTS.with(|store| store.borrow().clone());
    let cloned_is_pdc_canister = remittance::state::IS_PDC_CANISTER.with(|store| store.borrow().clone());
    let dc_canisters = remittance::state::DC_CANISTERS.with(|store| store.borrow().clone());
    let remittance_reciepts_store = remittance::state::REMITTANCE_RECIEPTS.with(|store| store.borrow().clone());
    let config_store = remittance::state::CONFIG.with(|store| store.borrow().clone());
    let canister_balance_store = remittance::state::CANISTER_BALANCE.with(|store| store.borrow().clone());
    
    // save cloned memory
    storage::stable_save((
        cloned_available_balance_store,
        cloned_witheld_balance_store,
        cloned_witheld_amounts,
        cloned_is_pdc_canister,
        dc_canisters,
        remittance_reciepts_store,
        config_store,
        canister_balance_store
    ))
    .unwrap()
}

#[post_upgrade]
async fn post_upgrade() {
    owner::init_owner();
    random::init_ic_rand();

    // load the variables from memory
    let (
        cloned_available_balance_store,
        cloned_witheld_balance_store,
        cloned_witheld_amounts_store,
        cloned_is_pdc_canister,
        cloned_dc_canisters,
        cloned_remittance_reciepts,
        cloned_config,
        cloned_canister_balance
    ): (
        remittance::types::AvailableBalanceStore,
        remittance::types::WithheldBalanceStore,
        remittance::types::WithheldAmountsStore,
        HashMap<Principal, bool>,
        Vec<Principal>,
        remittance::types::RemittanceRecieptsStore,
        remittance::config::Config,
        remittance::types::CanisterBalanceStore
    ) = storage::stable_restore().unwrap();

    //  restore by reassigning to vairiables
    remittance::state::REMITTANCE.with(|r| *r.borrow_mut() = cloned_available_balance_store);
    remittance::state::WITHHELD_REMITTANCE.with(|wr| *wr.borrow_mut() = cloned_witheld_balance_store);
    remittance::state::WITHHELD_AMOUNTS.with(|wa| *wa.borrow_mut() = cloned_witheld_amounts_store);
    remittance::state::IS_PDC_CANISTER.with(|ipc| *ipc.borrow_mut() = cloned_is_pdc_canister);
    remittance::state::DC_CANISTERS.with(|dc| *dc.borrow_mut() = cloned_dc_canisters);
    remittance::state::REMITTANCE_RECIEPTS.with(|rr| *rr.borrow_mut() = cloned_remittance_reciepts);
    remittance::state::CONFIG.with(|c| *c.borrow_mut() = cloned_config);
    remittance::state::CANISTER_BALANCE.with(|c| *c.borrow_mut() = cloned_canister_balance);
}
// --------------------------- upgrade hooks ------------------------- //
