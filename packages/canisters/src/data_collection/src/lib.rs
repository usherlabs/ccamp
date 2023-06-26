// this canister is responsible for collecting data
// and then publishing it to all the remittance canisters subscribed to it
// it essentially Loads -> transforms -> Publishes data
use candid::{CandidType, Principal};
use ic_cdk_macros::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;

type SubscriberStore = BTreeMap<Principal, Subscriber>;
thread_local! {
    static SUBSCRIBERS: RefCell<SubscriberStore> = RefCell::default();
}
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Counter {
    topic: String,
    value: u64,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Subscriber {
    topic: String,
}

#[query]
fn greet(name: String) -> String {
    format!("Hello data_collection canister, {}!", name)
}

#[update]
fn subscribe(subscriber: Subscriber) {
    let subscriber_principal_id = ic_cdk::caller();
    SUBSCRIBERS.with(|subscribers| {
        subscribers
            .borrow_mut()
            .insert(subscriber_principal_id, subscriber)
    });
}

#[update]
async fn publish(counter: Counter) {
    SUBSCRIBERS.with(|subscribers| {
        for (k, v) in subscribers.borrow().iter() {
            if v.topic == counter.topic {
                let _call_result: Result<(), _> = ic_cdk::notify(*k, "update_count", (&counter,));
            }
        }
    });
}
