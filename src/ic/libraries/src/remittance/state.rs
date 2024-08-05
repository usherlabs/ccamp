use candid::Principal;
use std::{cell::RefCell, collections::HashMap};

use super::{config, types};

thread_local! {
    pub static REMITTANCE: RefCell<types::AvailableBalanceStore> = RefCell::default();
    pub static WITHHELD_REMITTANCE: RefCell<types::WithheldBalanceStore> = RefCell::default();
    pub static WITHHELD_AMOUNTS: RefCell<types::WithheldAmountsStore> = RefCell::default();
    pub static IS_PDC_CANISTER: RefCell<HashMap<Principal, bool>> = RefCell::default();
    pub static DC_CANISTERS: RefCell<Vec<Principal>> = RefCell::default();
    pub static REMITTANCE_RECIEPTS: RefCell<types::RemittanceRecieptsStore> = RefCell::default();
    pub static CANISTER_BALANCE: RefCell<types::CanisterBalanceStore> = RefCell::default();
    pub static CONFIG: RefCell<config::Config> = RefCell::default();
}