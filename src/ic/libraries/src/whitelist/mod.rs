use std::{cell::RefCell, collections::HashMap};

use candid::Principal;

thread_local! {
    static WHITE_LIST: RefCell<HashMap<Principal, bool>> = RefCell::default();
}

pub fn add_to_whitelist(principal: Principal) {
    WHITE_LIST.with(|rc| rc.borrow_mut().insert(principal, true));
}

pub fn remove_from_whitelist(principal: Principal) {
    WHITE_LIST.with(|rc| rc.borrow_mut().remove(&principal));
}

pub fn is_whitelisted(principal: Principal) -> bool {
    WHITE_LIST.with(|rc| rc.borrow().clone().contains_key(&principal))
}
