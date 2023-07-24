use std::cell::RefCell;

use candid::Principal;
use ic_cdk::caller;

thread_local! {
    static OWNER: RefCell<Option<Principal>> = RefCell::default();
}

// ------- Access control
pub fn only_publisher() {
    let caller_principal_id = caller();
    if !crate::DC_CANISTERS.with(|publisher| publisher.borrow().contains(&caller_principal_id)) {
        panic!("NOT_ALLOWED");
    }
}

pub fn only_owner() {
    let caller_principal_id = caller();
    if !OWNER.with(|owner| owner.borrow().expect("NO_OWNER") == caller_principal_id) {
        panic!("NOT_ALLOWED");
    }
}

pub fn init_owner() {
    let caller_principal_id = caller();
    OWNER.with(|token| {
        token.replace(Some(caller_principal_id));
    });
}

pub fn get_owner() -> String {
    OWNER.with(|owner| owner.borrow().clone().expect("NO_OWNER").to_string())
}
// ------- Access control