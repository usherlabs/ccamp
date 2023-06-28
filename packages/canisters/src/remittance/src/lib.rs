use candid::Principal;
use ic_cdk::caller;
use ic_cdk_macros::*;
use std::cell::RefCell;
mod remittance;
use lib;

const REMITTANCE_EVENT: &str = "REMITTANCE";
thread_local! {
    static REMITTANCE: RefCell<remittance::Store> = RefCell::default();
    static OWNER: RefCell<Option<Principal>> = RefCell::default();
    static PUBLISHERS: RefCell<Vec<Principal>> = RefCell::default();
}

fn only_publisher() {
    let caller_principal_id = caller();
    if !PUBLISHERS.with(|publisher| publisher.borrow().contains(&caller_principal_id)) {
        panic!("NOT_ALLOWED");
    }
}

fn only_owner() {
    let caller_principal_id = caller();
    if !OWNER.with(|owner| owner.borrow().expect("NO_OWNER") == caller_principal_id) {
        panic!("NOT_ALLOWED");
    }
}

#[init]
fn init() {
    // TODO: upon upgrade of canister, sometimes initialzed variables are lost, reinitialise on upgrade or find way to preserve state
    let caller_principal_id = caller();
    OWNER.with(|token| {
        token.replace(Some(caller_principal_id));
    })
}

// get deployer of contract
#[query]
fn owner() -> String {
    OWNER.with(|owner| owner.borrow().clone().expect("NO_OWNER").to_string())
}

// @dev test function
#[query]
fn greet(name: String) -> String {
    format!("Hello data_collection canister, {}!", name)
}

// W.I.P this would be called to fe
#[query]
fn request() -> String {
    // make sure this function can only be called by a registered user
    format!("Signature_response")
}

// we call this method, with the id of the data_collection canister
// this then subscribes the remittance canister to "REMITTANCE" events from the data cannister
#[update]
async fn setup_subscribe(publisher_id: Principal) {
    only_owner();
    let subscriber = lib::Subscriber {
        topic: REMITTANCE_EVENT.to_string(),
    };
    let _call_result: Result<(), _> = ic_cdk::call(publisher_id, "subscribe", (subscriber,)).await;
    // update the list of all the publishers subscribed to while avoiding duplicates
    PUBLISHERS.with(|publisher| {
        let mut borrowed_publisher = publisher.borrow_mut();
        if !borrowed_publisher.contains(&publisher_id) {
            borrowed_publisher.push(publisher_id)
        }
    });
}

// test to get the last publisher
// test to get the number of publishers
// test to check if the passed in principal is present in the vector
// #[query]
// fn includes_publisher(publisher_id: Principal) -> bool {
//     PUBLISHERS.with(|publisher| publisher.borrow().contains(&publisher_id))
// }

// this is an external function which is going to be called by  the data collection canister
// when there is a new data
// it essentially uses the mapping (ticker, chain, recipientaddress) => {DataModel}
// so if an entry exists for a particular combination of (ticker, chain, recipientaddress)
// then the price is updated, otherwise the entry is created
#[update]
fn update_remittance(new_remittance: lib::DataModel) {
    only_publisher();
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
                lib::DataModel {
                    ticker: new_remittance.ticker.clone(),
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
) -> lib::DataModel {
    let chain = lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");

    REMITTANCE.with(|remittance| {
        let existing_key = (ticker, chain, recipient_address);
        remittance
            .borrow()
            .get(&existing_key)
            .expect("REMITTANCE_NOT_FOUND ")
            .clone()
    })
}
