// this canister basically first subscribes to a data collection canister
// then once the data collection cannister has some data
// it publishes it over the pub sub layer
// this canister then
use candid::{CandidType, Principal};
use ic_cdk_macros::*;
use serde::Deserialize;
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
};

// define a data structure to store all the data recieved for remittance
const REMITTANCE_EVENT: &str = "REMITTANCE";
type RemittanceStore = HashMap<(String, u64, String), DataModel>;
thread_local! {
    static REMITTANCE: RefCell<RemittanceStore> = RefCell::default();
    static COUNTER: Cell<u64> = Cell::new(0);
}

// define the structure of the remittance data model
#[derive(Clone, Debug, CandidType, Deserialize)]
struct DataModel {
    ticker: String,
    chain_id: u64,
    recipient_address: String,
    amount: u64,
}
#[derive(Clone, Debug, CandidType, Deserialize)]
struct Counter {
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

// this is the function that would be called by the user to request a signature
#[query]
fn request() -> String {
    format!("Signature_response")
}

// this function is called by the subscriber(remittance canister)
// it takes in the id of publisher(data remittance canister)
// and then calls the "subscribe" function on the remittance canister
// which makes sure that when there sia new data it would be captured by the "update_remittance" function
#[update]
async fn setup_subscribe(publisher_id: Principal) {
    let subscriber = Subscriber {
        topic: REMITTANCE_EVENT.to_string(),
    };
    let _call_result: Result<(), _> = ic_cdk::call(publisher_id, "subscribe", (subscriber,)).await;
}

// this function is called with a new remittance object when there is a new data from the data collection canister
#[update]
fn update_remittance(new_remittance: DataModel) {
    REMITTANCE.with(|remittance| {
        // when there is a new remittance item
        // check if the key exists
        // if it does, increase the keys
        // if it doesnt then we create a new instance
        let mut remittance_store = remittance.borrow_mut();

        let hash_key = (
            new_remittance.ticker.clone(),
            new_remittance.chain_id,
            new_remittance.recipient_address.clone(),
        );

        if let Some(existing_data) = remittance_store.get_mut(&hash_key) {
            // Key exists, update the amount that can be remitted by adding the new amount
            existing_data.amount += new_remittance.amount;
        } else {
            // Key doesn't exist, create a new entry with the remittance data
            remittance_store.insert(
                hash_key,
                DataModel {
                    ticker: new_remittance.ticker.clone(),
                    chain_id: new_remittance.chain_id,
                    recipient_address: new_remittance.recipient_address.clone(),
                    amount: new_remittance.amount,
                },
            );
        }
    });
}

// this function is just used to test and confirm if the data is actually included in the hashmap(object/dictionary)
// and if it can be queried, it would eventually be taken out when we get to testing
#[query]
fn get_remittance(ticker: String, chain_id: u64, recipient_address: String) -> DataModel {
    // Modify an existing value in the hashmap
    REMITTANCE.with(|remittance| {
        let existing_key = (ticker, chain_id, recipient_address);
        remittance
            .borrow()
            .get(&existing_key)
            .expect("NO_REMITTANCE")
            .clone()
    })
}
