use candid::Principal;

use ic_cdk::storage;
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use std::{cell::RefCell, collections::BTreeMap};

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
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(TIMER_INTERVAL_SEC), || {
        ic_cdk::spawn(logstore::query_logstore())
    });
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

#[query]
fn is_subscribed(principal: Principal) -> bool {
    SUBSCRIBERS.with(|subscribers| subscribers.borrow().contains_key(&principal))
}


// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store: BTreeMap<Principal, lib::Subscriber> =
        SUBSCRIBERS.with(|store| store.borrow().clone());
    storage::stable_save((cloned_store,)).unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    init().await;

    let (old_store,): (lib::SubscriberStore,) = storage::stable_restore().unwrap();
    SUBSCRIBERS.with(|store| *store.borrow_mut() = old_store);
}
// --------------------------- upgrade hooks ------------------------- //
