use candid::Principal;
use ic_cdk::{api, storage};
use ic_cdk_macros::*;
use std::cell::RefCell;

const REMITTANCE_EVENT: &str = "REMITTANCE";
thread_local! {
    static SUBSCRIBERS: RefCell<lib::dc::SubscriberStore> = RefCell::default();
}

// @dev testing command
#[query]
fn name() -> String {
    format!("data_collection canister")
}

#[init]
async fn init() {
    lib::owner::init_owner();
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
    let sample_adjust_one = lib::DataModel {
        token: "0xB24a30A3971e4d9bf771BDc81435c25EA69A445c"
            .to_string()
            .try_into()
            .unwrap(),
        chain: lib::Chain::Ethereum5,
        amount: -100,
        account: "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840"
            .to_string()
            .try_into()
            .unwrap(),
        action: lib::Action::Adjust,
    };

    let sampe_adjust_two = lib::DataModel {
        token: "0xB24a30A3971e4d9bf771BDc81435c25EA69A445c"
            .to_string()
            .try_into()
            .unwrap(),
        chain: lib::Chain::Ethereum5,
        amount: 100,
        account: "0x1AE26a1F23E2C70729510cdfeC205507675208F2"
            .to_string()
            .try_into()
            .unwrap(),
        action: lib::Action::Adjust,
    };

    let bulk_update = vec![sample_adjust_one, sampe_adjust_two];

    SUBSCRIBERS.with(|subscribers| {
        for (k, v) in subscribers.borrow().iter() {
            if v.topic == REMITTANCE_EVENT {
                let dc_canister = api::id();
                let _call_result: Result<(), _> =
                    ic_cdk::notify(*k, "update_remittance", (&bulk_update, dc_canister));
            }
        }
    });
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_store: lib::dc::SubscriberStore = SUBSCRIBERS.with(|store| store.borrow().clone());
    storage::stable_save((cloned_store,)).unwrap()
}
#[post_upgrade]
async fn post_upgrade() {
    init().await;

    let (old_store,): (lib::SubscriberStore,) = storage::stable_restore().unwrap();
    SUBSCRIBERS.with(|store| *store.borrow_mut() = old_store);
}
// --------------------------- upgrade hooks ------------------------- //
