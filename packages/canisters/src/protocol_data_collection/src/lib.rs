use candid::{Principal, CandidType};

use ic_cdk::storage;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use ic_cdk_timers::{TimerId, clear_timer};
use lib::RemittanceSubscriber;
use serde_derive::Deserialize;
use std::{cell::RefCell, sync::atomic::Ordering};

mod logstore;
mod remittance;

const TIMER_INTERVAL_SEC: u64 = 60;
// ? Currently running a timer, but in the future will be executed via the public method.
// ? What might be necessary, is a public method to schedule the PDC Canister to start based a wait time relative to block confirmations.

thread_local! {
    static REMITTANCE_CANISTER: RefCell<Option<lib::RemittanceSubscriber>> = RefCell::default();
    pub static TIMER_ID: RefCell<Option<TimerId>> = RefCell::default();
}

// ----------------------------------- init hook ------------------------------------------ //
#[init]
async fn init() {
    lib::owner::init_owner();
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


#[query]
fn last_queried_timestamp() -> u64 {
    logstore::get_last_timestamp()
}

#[query]
fn get_query_url() -> String {
    logstore::get_query_url()
}

#[query]
fn get_query_token() -> String {
    lib::owner::only_owner();

    logstore::get_query_token()
}

#[update]
pub fn set_last_timestamp(last_timestamp: u64) {
    lib::owner::only_owner();

    logstore::set_last_timestamp(last_timestamp);
}

#[update]
pub fn set_query_url(query_url: String) {
    lib::owner::only_owner();

    logstore::set_query_url(query_url);
}

#[update]
pub fn set_query_token(query_token: String) {
    lib::owner::only_owner();

    logstore::set_query_token(query_token);
}

#[update]
pub async fn update_data() {
    // validators
    logstore::is_initialised();
    // validators

    logstore::query_logstore().await;
}

#[update]
pub fn initialise_logstore(last_timestamp: u64, query_url: String, query_token: String) {
    lib::owner::only_owner();
    logstore::initialise_logstore(last_timestamp, query_url, query_token);
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

// start job to poll
// we have to manually call this function to start the polling process
#[update]
pub async fn poll_logstore() {
    // confirm at least one remittance canister is subscribed to this pdc
    let whitelisted_remittance_canister = get_remittance_canister();

    if !whitelisted_remittance_canister.subscribed {
        panic!("REMITTANCE_CANISTER_NOT_INITIALIZED")
    }

    // confirm logstore is initialised
    logstore::is_initialised();

    let timer_id = ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(TIMER_INTERVAL_SEC), || {
        ic_cdk::spawn(logstore::query_logstore())
    });
    TIMER_ID.with(|tid| *tid.borrow_mut() = Some(timer_id))
}

#[update]
pub async fn halt_logstore_poll(){
    let timer_id = TIMER_ID.with(|tid|tid.borrow().expect("TIMER_NOT_SET"));
    clear_timer(timer_id);
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

#[query]
fn is_subscribed(canister_principal: Principal) -> bool {
    let whitelisted_remittance_canister = get_remittance_canister();

    return whitelisted_remittance_canister.canister_principal == canister_principal && whitelisted_remittance_canister.subscribed;
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store = REMITTANCE_CANISTER.with(|rc| rc.borrow().clone());
    let cloned_timestamp = logstore::LAST_TIMESTAMP.with(|ts| ts.load(Ordering::Relaxed));
    let cloned_query_url = logstore::get_query_url();
    let cloned_query_token = logstore::get_query_token();
    storage::stable_save((
        cloned_store,
        cloned_timestamp,
        cloned_query_url,
        cloned_query_token,
    ))
    .unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    let (old_store, old_timestamp, old_query_url, old_query_token): (
        Option<lib::RemittanceSubscriber>,
        u64,
        String,
        String,
    ) = storage::stable_restore().unwrap();

    REMITTANCE_CANISTER.with(|store| *store.borrow_mut() = old_store);
    logstore::LAST_TIMESTAMP.with(|ts| ts.store(old_timestamp, Ordering::SeqCst));
    logstore::QUERY_TOKEN.with(|token| *token.borrow_mut() = Some(old_query_token));
    logstore::QUERY_URL.with(|url| *url.borrow_mut() = Some(old_query_url));

    init().await;
}
// --------------------------- upgrade hooks ------------------------- //
