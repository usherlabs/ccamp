use candid::Principal;
use ic_cdk::storage;
use ic_cdk_macros::*;
use lib::{dc::publish_json_to_remittance, RemittanceSubscriber};

// @dev testing command
#[query]
fn name() -> String {
    format!("data_collection canister")
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

// we would use this method to publish data to the subscriber
// which would be the remittance model
// so when we have some new data, we would publish it to the remittance model
#[update]
async fn manual_publish(json_data: String) -> Result<(), String> {
    // create a dummy remittance object we can publish until we implement data collection
    // which would then generate the data instead of hardcoding it
    let response = publish_json_to_remittance(json_data).await;

    response
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store = lib::dc::REMITTANCE_CANISTER.with(|rc| rc.borrow().clone());
    storage::stable_save((cloned_store,)).unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    init().await;

    let (old_store,): (Option<lib::RemittanceSubscriber>,) = storage::stable_restore().unwrap();
    lib::dc::REMITTANCE_CANISTER.with(|store| *store.borrow_mut() = old_store);
}
// --------------------------- upgrade hooks ------------------------- //
