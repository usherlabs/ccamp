use candid::Principal;
use std::collections::BTreeMap;

pub type SubscriberStore = BTreeMap<Principal, crate::Subscriber>;
