use core::panic;
use solabi::{abi::EventDescriptor, Log, Topics};
use lib::{utils::string_to_vec_u8, Event};
use crate::logstore::{
    constants::{
        FUNDS_CANCELED_DECLARATION, FUNDS_DEPOSITED_DECLARATION, FUNDS_WITHDRAWN_DECLARATION,
    },
    utils::{extract_event_name, fmt_event_data},
};

pub mod constants;
pub mod traits;
pub mod types;
pub mod utils;

pub fn derive_event_model(topics: &Vec<String>, data: &String) -> lib::Event {
    let event_selector = &topics[0][..];

    // match an event selector to a corresponding parser
    let event_descriptor = match event_selector {
        "0x7b2c468feb026788630cfbdc9c64aa29bde0318b58c8ab900614cc96f9305955" => {
            EventDescriptor::parse_declaration(FUNDS_DEPOSITED_DECLARATION)
        }
        "0x8407e01c63958861c3963bdc9dd3ea91ccde4c64aeef37c136ae8549cf828808" => {
            EventDescriptor::parse_declaration(FUNDS_WITHDRAWN_DECLARATION)
        }
        "0x0f26c836f96a618c08949606c5dea169c2ae9ade5c3b42b00e7aacfc4be0612a" => {
            EventDescriptor::parse_declaration(FUNDS_CANCELED_DECLARATION)
        }
        _ => panic!("INVALID_EVENT_TYPE"),
    }
    .unwrap();

    let event_encoder = solabi::value::EventEncoder::new(&event_descriptor).unwrap();
    let log = Log {
        topics: Topics::from([
            string_to_vec_u8(&topics[0][..]).try_into().unwrap(),
            string_to_vec_u8(&topics[1][..]).try_into().unwrap(),
        ]),
        data: string_to_vec_u8(&data).try_into().unwrap(),
    };

    let parsed_logs = event_encoder.decode(&log).unwrap();
    let canister_id = fmt_event_data(&parsed_logs[0]);
    let account_address = fmt_event_data(&parsed_logs[1]);
    let amount = fmt_event_data(&parsed_logs[2]);
    let chain = fmt_event_data(&parsed_logs[3]);
    let token_address = fmt_event_data(&parsed_logs[4]);
    let event_name = extract_event_name(&event_descriptor.canonical().to_string()[..]).to_string();

    Event {
        event_name: event_name,
        canister_id: canister_id,
        account: account_address,
        amount: amount.parse::<i64>().unwrap(),
        chain: chain,
        token: token_address,
    }
}
