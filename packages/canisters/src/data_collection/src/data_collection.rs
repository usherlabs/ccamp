use candid::Principal;
use lib;
use std::collections::BTreeMap;

pub type SubscriberStore = BTreeMap<Principal, lib::Subscriber>;
