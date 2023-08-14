use candid::Principal;

pub const REMITTANCE_EVENT: &str = "REMITTANCE";

// we would use this method to publish data to the subscriber
// which would be the remittance model
// so when we have some new data, we would publish it to the remittance model
pub async fn broadcast_to_subscribers(events: &Vec<lib::DataModel>, dc_canister: Principal) {
    crate::SUBSCRIBERS.with(|subscribers| {
        for (k, v) in subscribers.borrow().iter() {
            if v.topic == REMITTANCE_EVENT {
                let _call_result: Result<(), _> =
                    ic_cdk::notify(*k, "update_remittance", (&events, dc_canister));
            }
        }
    });
}
