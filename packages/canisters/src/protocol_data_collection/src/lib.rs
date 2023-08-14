use candid::Principal;
use ic_cdk_macros::*;
use std::cell::RefCell;
use ic_cdk_macros::{init, post_upgrade, query, update};
use std::sync::atomic::{AtomicU64, Ordering};

const TIMER_INTERVAL_SEC: u64 = 60;

mod logstore;


const REMITTANCE_EVENT: &str = "REMITTANCE";
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

#[update]
pub fn update_data() {
    lib::owner::only_owner();
    // dummy action, will be replaced with http call to logstore network
    logstore::query_logstore()
}

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

// we would use this method to publish data to the subscriber
// which would be the remittance model
// so when we have some new data, we would publish it to the remittance model
#[update]
async fn publish() {
    // create a dummy remittance object we can publish until we implement data collection
    // which would then generate the data instead of hardcoding it
    let sample_deposit_one = lib::DataModel {
        token: "0x99Cb2B2f007d6Aa21a7d864687110Cdc0573591a"
            .to_string()
            .try_into()
            .unwrap(),
        chain: lib::Chain::Ethereum5,
        amount: 1000000,
        account: "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840"
            .to_string()
            .try_into()
            .unwrap(),
        action: lib::Action::Deposit,
    };

    let sample_deposit_two = lib::DataModel {
        token: "0x99Cb2B2f007d6Aa21a7d864687110Cdc0573591a"
            .to_string()
            .try_into()
            .unwrap(),
        chain: lib::Chain::Ethereum5,
        amount: 500000,
        account: "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840"
            .to_string()
            .try_into()
            .unwrap(),
        action: lib::Action::Deposit,
    };

    let bulk_update = vec![
        sample_deposit_one,
        sample_deposit_two,
    ];
    // TODO make this the dc_canister responsible for the incoming data
    let dc_canister: Principal = "bkyz2-fmaaa-aaaaa-qaaaq-cai".try_into().unwrap();

    SUBSCRIBERS.with(|subscribers| {
        for (k, v) in subscribers.borrow().iter() {
            if v.topic == REMITTANCE_EVENT {
                let _call_result: Result<(), _> =
                    ic_cdk::notify(*k, "update_remittance", (&bulk_update, dc_canister));
            }
        }
    });
}
