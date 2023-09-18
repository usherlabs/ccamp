use core::panic;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod,
};
use ic_cdk::api::time;
use serde_json::Value;
use std::cell::RefCell;

use std::sync::atomic::{AtomicU64, Ordering};

use crate::remittance;

thread_local! {
    // set this variable to the timestamp to which we want to start querying the logstore via timestamp from
    pub static LAST_TIMESTAMP: AtomicU64 = AtomicU64::new(1694614219969);
    pub static QUERY_URL: RefCell<Option<String>> = RefCell::default();
    pub static QUERY_TOKEN: RefCell<Option<String>> = RefCell::default();
}

pub fn get_last_timestamp() -> u64 {
    LAST_TIMESTAMP.with(|counter| counter.load(Ordering::Relaxed))
}

pub fn get_query_url() -> String {
    QUERY_URL.with(|url| url.borrow().clone().expect("QUERY_URL_NOT_SET"))
}

pub fn get_query_token() -> String {
    QUERY_TOKEN.with(|token| token.borrow().clone().expect("QUERY_TOKENL_NOT_SET"))
}

pub fn set_last_timestamp(last_timestamp: u64) {
    LAST_TIMESTAMP.with(|ts| ts.store(last_timestamp, Ordering::SeqCst))
}

pub fn set_query_url(query_url: String) {
    QUERY_URL.with(|url| *url.borrow_mut() = Some(query_url))
}

pub fn set_query_token(query_token: String) {
    QUERY_TOKEN.with(|token| *token.borrow_mut() = Some(query_token))
}

pub fn initialise_logstore(last_timestamp: u64, query_url: String, query_token: String) {
    set_last_timestamp(last_timestamp);
    set_query_url(query_url);
    set_query_token(query_token);
}

pub fn is_initialised() {
    get_query_token();
    get_query_url();
    get_last_timestamp();
}

// parse the http response gotten from logstore into json string
fn parse_logstore_response_to_event(event_string: String) -> String {
    let json_event: Value =
        serde_json::from_str(&event_string[..]).expect("Failed to deserialize JSON");

    let response: Vec<Value> =
        if let Value::Array(events) = json_event.get("messages").expect("INVALID_MESSAGE") {
            events
                .iter()
                .map(|event| event.get("content").expect("INVALID").clone())
                .collect()
        } else {
            panic!("INVALID_DATA_FORMAT");
        };
    Value::from_iter(response).to_string()
}

// the function to query the logstore
pub async fn query_logstore() {
    let query_url = get_query_url(); //"https://broker-us-1.logstore.usher.so/streams/0x9c81e8f60a9b8743678f1b6ae893cc72c6bc6840%2fccamp%2falphanet";
    let query_token = get_query_token(); //"MHg5YzgxZThmNjBhOWI4NzQzNjc4ZjFiNmFlODkzY2M3MmM2YmM2ODQwOjB4MmI2NmRiNTgwOTcwYTkyYzcyMzEwMzFiMGFlNzY2MjU4Y2RkNTEzODkwOTMyNDUyM2QzNDY2OGVmMDJlZDNiMDQ5ZjFhMTgzMmRiNTI0NTQwMTVhODczMzYwMDY5YTJkZWIyODNhNjM0ZTYwZDE5NDg4ZTcxNDI0NTc5OTBlM2MxYw";

    let last_timestamp = get_last_timestamp();
    let url = format!(
        "{query_url}/data/partitions/0/from?fromTimestamp={}&format=object",
        last_timestamp
    );

    let request_headers = vec![HttpHeader {
        name: "Authorization".to_string(),
        value: format!("Basic {query_token}"),
    }];

    //note "CanisterHttpRequestArgument" and "HttpMethod" are declared in line 4
    let request = CanisterHttpRequestArgument {
        url: url.to_string(),
        method: HttpMethod::GET,
        body: None,               //optional for request
        max_response_bytes: None, //optional for request
        transform: None,          //optional for request
        headers: request_headers,
    };

    let http_response: Result<String, String> =
        match http_request(request).await {
            Ok((response,)) => Ok(String::from_utf8(response.body)
                .expect("Transformed response is not UTF-8 encoded.")),

            Err((r, m)) => Err(format!(
                "The http_request resulted into error. RejectionCode: {r:?}, Error: {m}"
            )),
        };

    match http_response {
        Ok(json_string) => {
            let str_body = parse_logstore_response_to_event(json_string);
            let _ = remittance::publish_json(str_body).await;
            LAST_TIMESTAMP.with(|counter| counter.store(time() / 1000000, Ordering::SeqCst));
        }

        Err(eror_string) => {
            panic!("{eror_string}");
        }
    }
}
