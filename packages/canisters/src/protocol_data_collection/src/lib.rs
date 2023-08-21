use candid::Principal;
use ic_cdk::print;
use ic_cdk_macros::{init, post_upgrade, query, update};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};

mod logstore;
mod remittance;

const TIMER_INTERVAL_SEC: u64 = 60;
thread_local! {
    static SUBSCRIBERS: RefCell<lib::dc::SubscriberStore> = RefCell::default();
}

// ----------------------------------- init and upgrade hooks
#[init]
async fn init() {
    lib::owner::init_owner();
    ic_cdk_timers::set_timer_interval(std::time::Duration::from_secs(TIMER_INTERVAL_SEC), || {
        ic_cdk::spawn(logstore::query_logstore())
    });
}

// upon upgrade of contracts, state is  lost
// so we need to reinitialize important variables here
#[post_upgrade]
async fn upgrade() {
    init().await;
}
// ----------------------------------- init and upgrade hooks

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
