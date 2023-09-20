use ic_cdk::api::call::RejectionCode;
use lib::Event;
use serde_json::Value;

pub async fn publish_json(json_data: String) -> Result<(), String> {
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
    let update_succesfull = if let Value::Array(events) = json_event {
        let mut parsed_events: Vec<lib::DataModel> = Vec::new();

        for event in events {
            // parse the json object gotten back into an "'Event' struct"
            let json_event: Event = serde_json::from_value(event).unwrap();
            // convert each "event" object into a data model and send it to the remittance canister
            let parsed_event: lib::DataModel = json_event.into();
            // send this info over to the remittance canister in order to modify the balances
            parsed_events.push(parsed_event);
        }
        // TODO: use the response from the broadcast to return a response of ok or otherwise
        let _ = broadcast_to_subscribers(&parsed_events).await;
        Ok(())
    } else {
        Err("ERROR_PARSING_EVENT_INTO_DATAMODEL".to_string())
    };

    update_succesfull
}

// we would use this method to publish data to the subscriber
// which would be the remittance model
// so when we have some new data, we would publish it to the remittance model
pub async fn broadcast_to_subscribers(events: &Vec<lib::DataModel>) -> Result<(), RejectionCode> {
    lib::dc::update_remittance_canister(events)
}
