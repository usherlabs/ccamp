use candid::Principal;
use ic_cdk::caller;
use std::{cell::RefCell, collections::HashMap};


thread_local! {
    pub static WHITELISTED_PUBLISHERS: RefCell<HashMap<Principal, bool>> = RefCell::default();
}

pub fn add_publisher(principal: Principal) {
    WHITELISTED_PUBLISHERS.with(|rc| rc.borrow_mut().insert(principal, true));
}

pub fn remove_publisher(principal: Principal) {
    WHITELISTED_PUBLISHERS.with(|rc| rc.borrow_mut().remove(&principal));
}

pub fn only_whitelisted_publishers(){
    let caller_principal_id = caller();
    let whitelisted = WHITELISTED_PUBLISHERS.with(|rc| rc.borrow().clone());

    if !whitelisted.contains_key(&caller_principal_id) {
        panic!("PRINCPAL NOT WHITELISTED")
    }
}