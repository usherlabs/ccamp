use candid::Principal;

use ic_cdk::storage;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use std::{cell::RefCell, sync::atomic::Ordering};

mod logstore;
mod remittance;

const TIMER_INTERVAL_SEC: u64 = 60;
thread_local! {
    static SUBSCRIBERS: RefCell<lib::dc::SubscriberStore> = RefCell::default();
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
    format!("data_collection canister")
}

// get deployer of contract
#[query]
fn owner() -> String {
    lib::owner::get_owner()
}

// fect dummy var to confirm timer is working
#[query]
fn last_queried_timestamp() -> u64 {
    logstore::get_last_timestamp()
}

#[update]
pub async fn update_data() {
    lib::owner::only_owner();
    logstore::query_logstore().await;
}

// start job to poll
// we have to manually call this function to start the polling process
#[update]
pub async fn poll_logstore() {
    // confirm at least one remittance canister is subscribed to this pdc
    let subscribers_length = SUBSCRIBERS.with(|subscribers| subscribers.borrow().len());
    if subscribers_length == 0 {
        panic!("NO_REMITTANCE_SUBSCRIBED")
    }
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(TIMER_INTERVAL_SEC), || {
        ic_cdk::spawn(logstore::query_logstore())
    });
}

// this function is going to be called by the remittance canister
// so it can recieve "publish" events from this canister
#[update]
fn subscribe(subscriber: lib::Subscriber) {
    let subscriber_principal_id = ic_cdk::caller();
    SUBSCRIBERS.with(|subscribers| {
        subscribers
            .borrow_mut()
            .insert(subscriber_principal_id, subscriber);
    });
}

#[update]
async fn manual_publish(json_data: String) {
    lib::owner::only_owner();

    let _ = remittance::publish_json(json_data).await;
}

#[query]
fn is_subscribed(principal: Principal) -> bool {
    SUBSCRIBERS.with(|subscribers| subscribers.borrow().contains_key(&principal))
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store = SUBSCRIBERS.with(|store| store.borrow().clone());
    let cloned_timestamp = logstore::LAST_TIMESTAMP.with(|ts| ts.load(Ordering::Relaxed));
    storage::stable_save((cloned_store, cloned_timestamp)).unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    let (old_store, old_timestamp): (lib::SubscriberStore, u64) =
        storage::stable_restore().unwrap();

    SUBSCRIBERS.with(|store| *store.borrow_mut() = old_store);
    logstore::LAST_TIMESTAMP.with(|counter| counter.store(old_timestamp, Ordering::SeqCst));

    init().await;
}
// --------------------------- upgrade hooks ------------------------- //
