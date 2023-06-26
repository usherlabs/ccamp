// this canister is responsible for collecting data
// and then publishing it to all the remittance canisters subscribed to it
// it essentially Loads -> transforms -> Publishes data
use candid::{CandidType, Principal};
use ic_cdk_macros::*;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::BTreeMap;

const REMITTANCE_EVENT: &str = "REMITTANCE";
// define the structure of the remittance data model
#[derive(Clone, Debug, CandidType, Deserialize)]
struct DataModel<'a> {
    ticker: &'a str,
    chain_id: u64,
    recipient_address: &'a str,
    amount: u64
}
// define the structure of the remittance data model
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Counter {
    value: u64,
}

#[derive(Clone, Debug, CandidType, Deserialize)]
struct Subscriber {
    topic: String,
}

type SubscriberStore = BTreeMap<Principal, Subscriber>;

thread_local! {
    static SUBSCRIBERS: RefCell<SubscriberStore> = RefCell::default();
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
async fn publish() {
    // create a dummy remittance object we can publish until we implement data collection
    // which would then generate daa
    let data_model = DataModel {
        ticker: "USDC",
        chain_id: 1,
        recipient_address: "0x1234567890123456789012345678901234567891",
        amount: 1000000
    };
    SUBSCRIBERS.with(|subscribers| {
        for (k, v) in subscribers.borrow().iter() {
            if v.topic == REMITTANCE_EVENT {
                let _call_result: Result<(), _> = ic_cdk::notify(*k, "update_remittance", (&data_model,));
            }
        }
    });
}
