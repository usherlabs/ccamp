use candid::Principal;
use ic_cdk::api;
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
        token: "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d"
            .to_string()
            .try_into()
            .unwrap(),
        chain: lib::Chain::Ethereum1,
        amount: 1000000,
        account: "0x57c1D4dbFBc9F8cB77709493cc43eaA3CD505432"
            .to_string()
            .try_into()
            .unwrap(),
        action: lib::Action::Deposit,
    };

    let sample_deposit_two = lib::DataModel {
        token: "0x8ac76a51cc950d9822d68b83fe1ad97b32cd580d"
            .to_string()
            .try_into()
            .unwrap(),
        chain: lib::Chain::Ethereum1,
        amount: 500000,
        account: "0x5B38Da6a701c568545dCfcB03FcB875f56beddC4"
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
