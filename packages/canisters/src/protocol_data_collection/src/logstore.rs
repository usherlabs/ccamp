use candid::Principal;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod,
};
use ic_cdk::api::time;
use serde::Deserialize;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::remittance;

thread_local! {
    // TODO back up this variable as well
    pub static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(1693383017218);
}

#[derive(Deserialize, Debug)]
pub struct Event {
    pub event_name: String,
    pub canister_id: String,
    pub account: String,
    pub amount: u64,
    pub chain: String,
    pub token: String,
}

impl Into<lib::DataModel> for Event {
    fn into(self) -> lib::DataModel {
        lib::DataModel {
            token: self.token.try_into().unwrap(),
            chain: self.chain.try_into().unwrap(),
            amount: self.amount as i64,
            account: self.account.try_into().unwrap(),
            action: self.event_name.try_into().unwrap(),
        }
    }
}

// dummy function to confirm the timer is continously running
pub fn get_last_timestamp() -> u64 {
    LAST_TIMESTAMP.with(|counter| counter.load(Ordering::Relaxed))
}

// the function to query the logstore
pub async fn query_logstore() {
    // confirm at least one remittance canister is subscribed to this pdc
    let subscribers_length = crate::SUBSCRIBERS.with(|subscribers| subscribers.borrow().len());
    if subscribers_length == 0 {
        panic!("NO_REMITTANCE_SUBSCRIBED")
    }
    let last_timestamp = get_last_timestamp();
    // update his value to the current timestamp

    let url = format!(
        "https://rich-suns-tan.loca.lt/query?from={}",
        last_timestamp
    );

    let request_headers = vec![
        // HttpHeader {
        //     name: "Host".to_string(),
        //     value: format!("{host}:443"),
        // },
        // HttpHeader {
        //     name: "User-Agent".to_string(),
        //     value: "exchange_rate_canister".to_string(),
        // },
    ];

    //note "CanisterHttpRequestArgument" and "HttpMethod" are declared in line 4
    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,               //optional for request
        max_response_bytes: None, //optional for request
        transform: None,          //optional for request
        headers: request_headers,
    };

    match http_request(request).await {
        //4. DECODE AND RETURN THE RESPONSE

        //See:https://docs.rs/ic-cdk/latest/ic_cdk/api/management_canister/http_request/struct.HttpResponse.html
        Ok((response,)) => {
            //if successful, `HttpResponse` has this structure:
            // pub struct HttpResponse {
            //     pub status: Nat,
            //     pub headers: Vec<HttpHeader>,
            //     pub body: Vec<u8>,
            // }

            //We need to decode that Vec<u8> that is the body into readable text.
            //To do this, we:
            //  1. Call `String::from_utf8()` on response.body
            let str_body = String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.");

            let json_event: Value =
                serde_json::from_str(&str_body[..]).expect("Failed to deserialize JSON");
            // Make sure the top-level JSON is an array
            if let Value::Array(events) = json_event {
                for event in events {
                    // parse the json object gotten back into an "'Event' struct"
                    let json_event: Event = serde_json::from_value(event).unwrap();
                    // parse the canister_id which is a string into a principal
                    let dc_canister: Principal = (&json_event.canister_id[..]).try_into().unwrap();
                    // convert each "event" object into a data model and send it to the remittance canister
                    let parsed_event: lib::DataModel = json_event.into();
                    // send this info over to the remittance canister in order to modify the balances
                    remittance::broadcast_to_subscribers(&vec![parsed_event], dc_canister).await;
                }
                // update the value of the query timestamp
                LAST_TIMESTAMP.with(|counter| counter.store(time() / 1000000, Ordering::SeqCst));
            }
        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            panic!("{message}");
        }
    };
}
