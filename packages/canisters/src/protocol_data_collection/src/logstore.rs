use ic_cdk_macros::{init, post_upgrade, query, update};
use std::sync::atomic::{AtomicU64, Ordering};

const TIMER_INTERVAL_SEC: u64 = 60;

thread_local! {
    pub static COUNTER: AtomicU64 = AtomicU64::new(0);
}


pub fn query_logstore() {
    // dummy action, will be replaced with http call to logstore network
    COUNTER.with(|counter| counter.fetch_add(1, Ordering::Relaxed));
}
