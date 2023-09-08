use candid::Principal;
use ic_cdk::storage;
use ic_cdk_macros::*;
use std::cell::RefCell;

mod remittance;

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
async fn manual_publish(json_data: String) {
    // create a dummy remittance object we can publish until we implement data collection
    // which would then generate the data instead of hardcoding it
    let _ = remittance::publish_json(json_data).await;
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
