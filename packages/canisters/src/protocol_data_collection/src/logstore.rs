use ic_cdk_macros::{init, post_upgrade, query};
use std::sync::atomic::{AtomicU64, Ordering};

const TIMER_INTERVAL_SEC: u64 = 60;

thread_local! {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
}

// ----------------------------------- init and upgrade hooks
#[init]
fn init() {
    ic_cdk_timers::set_timer_interval(
        std::time::Duration::from_secs(TIMER_INTERVAL_SEC),
        query_logstore,
    );
}

// upon upgrade of contracts, state is  lost
// so we need to reinitialize important variables here
#[post_upgrade]
fn upgrade() {
    init();
}
// ----------------------------------- init and upgrade hooks

pub fn query_logstore() {
    // dummy action, will be replaced with http call to logstore network
    COUNTER.with(|counter| counter.fetch_add(1, Ordering::Relaxed));
}

// fect dummy var to confirm timer is working
#[query]
fn counter() -> u64 {
    COUNTER.with(|counter| counter.load(Ordering::Relaxed))
}
