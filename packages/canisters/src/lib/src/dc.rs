use candid::Principal;
use ic_cdk::api::{self, call::RejectionCode};
use std::{cell::RefCell, collections::BTreeMap};
pub type SubscriberStore = BTreeMap<Principal, crate::Subscriber>;

thread_local! {
    pub static REMITTANCE_CANISTER: RefCell<Option<crate::RemittanceSubscriber>> = RefCell::default();
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
pub fn update_remittance_canister(events: &Vec<crate::DataModel>) -> Result<(), RejectionCode> {
    let dc_canister = api::id();
    let whitelisted_remittance_canister = get_remittance_canister();

    if !whitelisted_remittance_canister.subscribed {
        panic!("REMITTANCE_CANISTER_NOT_INITIALIZED")
    }

    let remittance_response: Result<(), RejectionCode> = ic_cdk::notify(
        whitelisted_remittance_canister.canister_principal,
        "update_remittance",
        (&events, dc_canister),
    );

    remittance_response
}
