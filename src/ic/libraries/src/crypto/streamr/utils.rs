use solabi::{abi::EventDescriptor, Address};

use crate::crypto::vec_u8_to_string;

/// given a a representation of an event log, convert the type from solabi type to a native type
pub fn fmt_event_data(param: &solabi::Value) -> String {
    match param {
        solabi::Value::String(s) => s.to_string(),
        solabi::Value::Address(Address(addr)) => {
            format!("0x{}", vec_u8_to_string(&addr.clone().into()))
        }
        solabi::Value::Uint(x) => {
            format!("{}", x.get())
        }
        _ => String::new(), // Add more cases for other SolType variants as needed
    }
}

/// convert FundsDeposited(string,address,uint256,string,address) to FundsDeposited
pub fn extract_event_name(input: &str) -> &str {
    input
        .split('(')
        .next()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .unwrap()
}

pub fn get_event_selector(event_declaration: &str) -> String {
    let event = EventDescriptor::parse_declaration(&event_declaration[..]).unwrap();

    format!(
        "0x{}",
        vec_u8_to_string(&event.selector().unwrap().to_vec())
    )
}
