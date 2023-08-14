use candid::Principal;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpMethod,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::remittance;

thread_local! {
    pub static COUNTER: AtomicU64 = AtomicU64::new(0);
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
pub fn query_logstore() {
    // dummy action, will be replaced with http call to logstore network
    COUNTER.with(|counter| counter.fetch_add(1, Ordering::Relaxed));
}

// the function to query the logstore
pub async fn query_logstore_wip() {
    // TOD change this URL to the query url and its query parameters
    let url = "https://my-json-server.typicode.com/typicode/demo/posts";
    // let url = format!(
    //     "https://{}/products/ICP-USD/candles?start={}&end={}&granularity={}",
    //     host,
    //     start_timestamp.to_string(),
    //     start_timestamp.to_string(),
    //     seconds_of_time.to_string()
    // );

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
            // let str_body = String::from_utf8(response.body)
            //     .expect("Transformed response is not UTF-8 encoded.");

            // ========================================= For now we will mock a sample response that we expect from the api ================================ //
            // {
            //     "chain":"ethereum:5",
            //     "event_name": "WithdrawCanceled",
            //     "canister_id": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
            //     "account": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
            //     "token": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
            //     "amount": 400000
            //   },
            //   {
            //     "chain":"ethereum:5",
            //     "event_name": "FundsWithdrawn",
            //     "canister_id": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
            //     "account": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
            //     "token": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
            //     "amount": 100000,
            //     "recipient": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840"
            //   },
            // ,
            //       {
            //         "chain":"ethereum:5",
            //         "event_name": "FundsDeposited",
            //         "canister_id": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
            //         "account": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
            //         "token": "0x99Cb2B2f007d6Aa21a7d864687110Cdc0573591a",
            //         "amount": 500000
            //       },
            //       {
            //         "chain":"ethereum:5",
            //         "event_name": "FundsDeposited",
            //         "canister_id": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
            //         "account": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
            //         "token": "0x99Cb2B2f007d6Aa21a7d864687110Cdc0573591a",
            //         "amount": 100
            //       }
            // mock response
            let str_body = r#"
            [
                {
                    "chain":"ethereum:5",
                    "event_name": "FundsDeposited",
                    "canister_id": "bkyz2-fmaaa-aaaaa-qaaaq-cai",
                    "account": "0x9C81E8F60a9B8743678F1b6Ae893Cc72c6Bc6840",
                    "token": "0x99Cb2B2f007d6Aa21a7d864687110Cdc0573591a",
                    "amount": 5000000
                }
              ]
            "#;
            // ========================================= For now we will mock a sample response that we expect from the api ================================ //
            let json_event: Value =
            serde_json::from_str(str_body).expect("Failed to deserialize JSON");
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
            }

        }
        Err((r, m)) => {
            let message =
                format!("The http_request resulted into error. RejectionCode: {r:?}, Error: {m}");
            panic!("{message}");
        }
    }
}
