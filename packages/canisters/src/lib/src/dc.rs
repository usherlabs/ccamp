use candid::{CandidType, Principal};
use ic_cdk::{api::{self}, id};
use serde_derive::Deserialize;
use serde_json::Value;
use std::{cell::RefCell, collections::BTreeMap};

use crate::Event;
pub type SubscriberStore = BTreeMap<Principal, crate::Subscriber>;

thread_local! {
    pub static REMITTANCE_CANISTER: RefCell<Option<crate::RemittanceSubscriber>> = RefCell::default();
}

#[derive(Clone, Debug, CandidType, Deserialize)]
pub struct Account {
    pub balance: u64,
}

//admin function to directly set a value for the remittance canister
pub fn set_remittance_canister(remittance_principal: Principal) {
    REMITTANCE_CANISTER.with(|rc| {
        let _ = rc.borrow_mut().insert(crate::RemittanceSubscriber {
            canister_principal: remittance_principal,
            subscribed: false,
        });
    })
}

// get the remittance canister
pub fn get_remittance_canister() -> crate::RemittanceSubscriber {
    // confirm at least one remittance canister is subscribed to this pdc
    REMITTANCE_CANISTER
        .with(|rc| rc.borrow().clone())
        .expect("REMITTANCE_CANISTER_NOT_INITIALIZED")
}

// this function is going to be called by the remittance canister which wants to be able to recieve data from this canister
pub fn subscribe() {
    // verify if this remittance canister has been whitelisted
    // set the subscribed value to true if its the same, otherwise panic
    let subscriber_principal_id = ic_cdk::caller();
    let whitelisted_remittance_canister = REMITTANCE_CANISTER
        .with(|rc| rc.borrow().clone())
        .expect("REMITTANCE_CANISTER_NOT_INITIALIZED");

    if whitelisted_remittance_canister.canister_principal != subscriber_principal_id {
        panic!("REMITTANCE_CANISTER_NOT_WHITELISTED")
    };

    REMITTANCE_CANISTER.with(|rc| {
        let _ = rc.borrow_mut().insert(crate::RemittanceSubscriber {
            canister_principal: subscriber_principal_id,
            subscribed: true,
        });
    });
}

// pass in a remittance canister principal to confirm if it has been subscribed to this data canister
pub fn is_subscribed(canister_principal: Principal) -> bool {
    let whitelisted_remittance_canister = REMITTANCE_CANISTER
        .with(|rc| rc.borrow().clone())
        .expect("REMITTANCE_CANISTER_NOT_INITIALIZED");

    return whitelisted_remittance_canister.canister_principal == canister_principal
        && return whitelisted_remittance_canister.canister_principal == canister_principal
            && whitelisted_remittance_canister.subscribed;
}

// we would use this method to publish data to the subscriber
// which would be the remittance model
// so when we have some new data, we would publish it to the remittance model
pub async fn update_remittance_canister(
    events: &Vec<crate::DataModel>,
    dc_canister: &Principal,
) -> Result<(), String> {
    let whitelisted_remittance_canister = get_remittance_canister();

    if !whitelisted_remittance_canister.subscribed {
        panic!("REMITTANCE_CANISTER_NOT_INITIALIZED")
    }

    let (remittance_response,): (Result<(), String>,) = api::call::call(
        whitelisted_remittance_canister.canister_principal,
        "update_remittance",
        (&events, dc_canister),
    )
    .await
    .unwrap();

    remittance_response
}

pub async fn publish_json_to_remittance(json_data: String) -> Result<(), String> {
    // the string provided should be an array of events
    // the same format the ccamp uses to fetch events from logstore
    // probably for some reason were missed by the poller or for some other reason we need a manual provision of events
    // schema
    // NOTE: a DC canister can only perform `BalanceAdjusted` adjust operations
    // [{
    //     "event_name": "BalanceAdjusted",
    //     "canister_id": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
    //     "account": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
    //     "amount": 100000,
    //     "chain": "ethereum:5",
    //     "token": "0xB24a30A3971e4d9bf771BDc81435c25EA69A445c"
    // }]

    // Parse the string of data into serde_json::Value.
    let json_event: Value =
        serde_json::from_str(&json_data[..]).expect("JSON_DESERIALIZATION_FAILED");
    // Make sure the top-level JSON is an array
    let update_succesful = if let Value::Array(events) = json_event {
        let mut parsed_events: Vec<crate::DataModel> = Vec::new();

        for event in events {
            // parse the json object gotten back into an "'Event' struct"
            let json_event: Event = serde_json::from_value(event).unwrap();
            // convert each "event" object into a data model and send it to the remittance canister
            let parsed_event: crate::DataModel = json_event.into();
            // send this info over to the remittance canister in order to modify the balances
            parsed_events.push(parsed_event);
        }

        let dc_canister = id();
        let response = update_remittance_canister(&parsed_events, &dc_canister).await;

        response
    } else {
        Err("ERROR_PARSING_EVENT_INTO_DATAMODEL".to_string())
    };

    update_succesful
}
