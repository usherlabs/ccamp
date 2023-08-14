use std::cell::RefCell;

use candid::Principal;
use ic_cdk::caller;

// ------- Access control
pub fn only_publisher() {
    let caller_principal_id = caller();
    if !crate::DC_CANISTERS.with(|publisher| publisher.borrow().contains(&caller_principal_id)) {
        panic!("NOT_ALLOWED");
    }
}
// ------- Access control