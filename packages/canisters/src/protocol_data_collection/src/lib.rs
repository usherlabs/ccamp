use candid::Principal;
use ic_cdk_macros::{init, post_upgrade, query, update};
use std::cell::RefCell;
use std::sync::atomic::{AtomicU64, Ordering};


mod logstore;
mod remittance;

const TIMER_INTERVAL_SEC: u64 = 60;
thread_local! {
    static SUBSCRIBERS: RefCell<lib::dc::SubscriberStore> = RefCell::default();
    static COUNTER: AtomicU64 = AtomicU64::new(0);
}

// ----------------------------------- init and upgrade hooks
#[init]
fn init() {
    lib::owner::init_owner();
    ic_cdk_timers::set_timer_interval(
        std::time::Duration::from_secs(TIMER_INTERVAL_SEC),
        logstore::query_logstore,
    );
}

// upon upgrade of contracts, state is  lost
// so we need to reinitialize important variables here
#[post_upgrade]
fn upgrade() {
    init();
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
fn counter() -> u64 {
    logstore::COUNTER.with(|counter| counter.load(Ordering::Relaxed))
}

#[update]
pub async fn update_data() {
    lib::owner::only_owner();
    // dummy action, will be replaced with http call to logstore network
    // for now json data is mocked in order to deposit to the network from a json response from an http endpoint
    logstore::query_logstore_wip().await;
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

