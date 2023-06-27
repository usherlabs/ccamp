use ic_cdk_macros::*;
use candid::Principal;
use std::cell::{Cell, RefCell};

mod remittance;

const REMITTANCE_EVENT: &str = "REMITTANCE";
thread_local! {
    static REMITTANCE: RefCell<remittance::Store> = RefCell::default();
    static COUNTER: Cell<u64> = Cell::new(0);
}

// @dev test function
#[query]
fn greet(name: String) -> String {
    format!("Hello data_collection canister, {}!", name)
}

// W.I.P this would be called to fe
#[query]
fn request() -> String {
    format!("Signature_response")
}

// we call this method, with the id of the data_collection canister
// this then subscribes the remittance canister to "REMITTANCE" events from the data cannister
#[update]
async fn setup_subscribe(publisher_id: Principal) {
    let subscriber = remittance::Subscriber {
        topic: REMITTANCE_EVENT.to_string(),
    };
    let _call_result: Result<(), _> = ic_cdk::call(publisher_id, "subscribe", (subscriber,)).await;
}

// this is an external function which is going to be called by  the data collection canister
// when there is a new data
// it essentially uses the mapping (ticker, chain, recipientaddress) => {DataModel}
// so if an entry exists for a particular combination of (ticker, chain, recipientaddress)
// then the price is updated, otherwise the entry is created
#[update]
fn update_remittance(new_remittance: remittance::DataModel) {
    REMITTANCE.with(|remittance| {
        let mut remittance_store = remittance.borrow_mut();

        let hash_key = (
            new_remittance.ticker.clone(),
            new_remittance.chain.clone(),
            new_remittance.recipient_address.clone(),
        );

        if let Some(existing_data) = remittance_store.get_mut(&hash_key) {
            existing_data.amount += new_remittance.amount;
        } else {
            remittance_store.insert(
                hash_key,
                remittance::DataModel {
                    ticker: new_remittance.ticker.clone(),
                    chain_id: new_remittance.chain_id.clone(),
                    chain_name: new_remittance.chain_name.clone(),
                    recipient_address: new_remittance.recipient_address.clone(),
                    amount: new_remittance.amount,
                    chain: new_remittance.chain,
                },
            );
        }
    });
}

// this function is just used to test and confirm if the data is actually included in the hashmap(object/dictionary)
// and if it can be queried, it would eventually be taken out when we get to testing
#[query]
fn get_remittance(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
) -> remittance::DataModel {
    let chain = remittance::Chain::from_chain_details(&chain_name, &chain_id).expect("Invalid chain");

    REMITTANCE.with(|remittance| {
        let existing_key = (ticker, chain, recipient_address);
        remittance
            .borrow()
            .get(&existing_key)
            .expect("Remittance not found ")
            .clone()
    })
}
