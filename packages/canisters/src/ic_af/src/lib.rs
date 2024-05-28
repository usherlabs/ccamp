use candid::Principal;
use ic_cdk_macros::*;

#[query]
fn hello() -> String {
    format!("this is hello")
}