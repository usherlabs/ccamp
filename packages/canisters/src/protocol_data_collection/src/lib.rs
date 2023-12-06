use candid::Principal;
use ic_cdk::{caller, storage};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use lib::{
    remittance::{Config, Environment},
    RemittanceSubscriber,
};
use std::{cell::RefCell, collections::HashMap};

mod logstore;
mod remittance;

thread_local! {
    static REMITTANCE_CANISTER: RefCell<Option<lib::RemittanceSubscriber>> = RefCell::default();
    static CONFIG: RefCell<Config> = RefCell::default();
    static WHITELISTED_PUBLISHERS: RefCell<HashMap<Principal, bool>> = RefCell::default();
}

// ----------------------------------- init hook ------------------------------------------ //
#[init]
async fn init(env_opt: Option<Environment>) {
    lib::owner::init_owner();

    // save the environment this is running in
    if let Some(env) = env_opt {
        CONFIG.with(|s| {
            let mut state = s.borrow_mut();
            *state = Config::from(env);
        })
    }
}
// ----------------------------------- init hook ------------------------------------------ //

// @dev testing command
#[query]
fn name() -> String {
    format!("protocol_data_collection canister")
}

// get deployer of contract
#[query]
fn owner() -> String {
    lib::owner::get_owner()
}

#[update]
pub async fn set_remittance_canister(remittance_principal: Principal) {
    lib::owner::only_owner();
    REMITTANCE_CANISTER.with(|rc| {
        let _ = rc.borrow_mut().insert(lib::RemittanceSubscriber {
            canister_principal: remittance_principal,
            subscribed: false,
        });
    })
}

#[query]
pub fn get_remittance_canister() -> RemittanceSubscriber {
    // confirm at least one remittance canister is subscribed to this pdc
    crate::REMITTANCE_CANISTER
        .with(|rc| rc.borrow().clone())
        .expect("REMITTANCE_CANISTER_NOT_INITIALIZED")
}

#[update]
pub fn add_publisher(principal: Principal) {
    WHITELISTED_PUBLISHERS.with(|rc| rc.borrow_mut().insert(principal, true));
}

#[update]
pub fn remove_publisher(principal: Principal) {
    WHITELISTED_PUBLISHERS.with(|rc| rc.borrow_mut().remove(&principal));
}

// this function is going to be called by the remittance canister
// so it can recieve "publish" events from this canister
#[update]
fn subscribe() {
    // verify if this remittance canister has been whitelisted
    // set the subscribed value to true if its the same, otherwise panic
    let subscriber_principal_id = ic_cdk::caller();
    let whitelisted_remittance_canister = REMITTANCE_CANISTER
        .with(|rc| rc.borrow().clone())
        .expect("REMITTANCE_CANISTER_NOT_INITIALIZED");

    if whitelisted_remittance_canister.canister_principal != subscriber_principal_id {
        panic!("REMITTANCE_CANISTER_NOT_WHITELISTED")
    };

    REMITTANCE_CANISTER.with(|rc| {
        let _ = rc.borrow_mut().insert(lib::RemittanceSubscriber {
            canister_principal: subscriber_principal_id,
            subscribed: true,
        });
    });
}

#[update]
async fn manual_publish(json_data: String) {
    lib::owner::only_owner();

    let _ = remittance::publish_json(json_data).await;
}

#[update]
async fn process_event(json_data: String) {
    let caller_principal_id = caller();
    let whitelisted = WHITELISTED_PUBLISHERS.with(|rc| rc.borrow().clone());

    if !whitelisted.contains_key(&caller_principal_id) {
        panic!("PRINCPAL NOT WHITELISTED")
    }
    let _ = remittance::publish_event(json_data).await;
}

#[query]
fn is_subscribed(canister_principal: Principal) -> bool {
    let whitelisted_remittance_canister = get_remittance_canister();

    return whitelisted_remittance_canister.canister_principal == canister_principal
        && whitelisted_remittance_canister.subscribed;
}

#[query]
fn get_caller() -> Principal {
    caller()
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store = REMITTANCE_CANISTER.with(|rc| rc.borrow().clone());
    let config_store = CONFIG.with(|store| store.borrow().clone());
    let whitelisted_store = WHITELISTED_PUBLISHERS.with(|store| store.borrow().clone());

    storage::stable_save((cloned_store, config_store, whitelisted_store)).unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    let (old_store, cloned_config, whitelisted_store): (
        Option<lib::RemittanceSubscriber>,
        Config,
        HashMap<Principal, bool>,
    ) = storage::stable_restore().unwrap();

    REMITTANCE_CANISTER.with(|store| *store.borrow_mut() = old_store);
    CONFIG.with(|c| *c.borrow_mut() = cloned_config);
    WHITELISTED_PUBLISHERS.with(|c| *c.borrow_mut() = whitelisted_store);

    lib::owner::init_owner();
}
// --------------------------- upgrade hooks ------------------------- //
