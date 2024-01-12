use std::collections::HashMap;

use candid::{CandidType, Principal};
use serde_derive::Deserialize;

pub type Timestamp = u64;
pub type Subaccount = [u8; 32];
pub type ApprovalType = HashMap<(Principal,Principal), Allowance>;
pub const DEFAULT_SUBACCOUNT: &Subaccount = &[0; 32];

#[derive(CandidType, Clone, Debug, Copy, Deserialize)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<Subaccount>,
}

impl Account {
    #[inline]
    pub fn effective_subaccount(&self) -> &Subaccount {
        self.subaccount.as_ref().unwrap_or(DEFAULT_SUBACCOUNT)
    }
}

impl From<Principal> for Account {
    fn from(account_principal: Principal) -> Self {
        Account {
            owner: account_principal,
            subaccount: None,
        }
    }
}

impl From<String> for Account {
    fn from(principal_text: String) -> Self {
        Account {
            owner: Principal::from_text(principal_text).unwrap(),
            subaccount: None,
        }
    }
}

impl std::hash::Hash for Account {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.owner.hash(state);
        self.effective_subaccount().hash(state);
    }
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct TransferArgs {
    pub from_subaccount: Option<Subaccount>,
    pub to: Account,
    pub amount: u128,
    pub fee: Option<u128>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub enum TransferError {
    BadFee { expected_fee: u128 },
    BadBurn { min_burn_amount: u128 },
    InsufficientFunds { balance: u128 },
    TooOld,
    CreatedInFuture { ledger_time: Timestamp },
    Duplicate { duplicate_of: u128 },
    TemporarilyUnavailable,
    GenericError { error_code: u128, message: String },
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct ApproveArgs {
    pub from_subaccount: Option<Vec<u8>>,
    pub spender: Account,
    pub amount: u64,
    pub expected_allowance: Option<u64>,
    pub expires_at: Option<u64>,
    pub fee: Option<u64>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub enum ApproveError {
    BadFee { expected_fee: u64 },
    InsufficientFunds { balance: u64 },
    AllowanceChanged { current_allowance: u64 },
    Expired { ledger_time: u64 },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: u64 },
    TemporarilyUnavailable,
    GenericError { error_code: u64, message: String },
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct AllowanceArgs {
    pub account: Account,
    pub spender: Account,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct TransferFromArgs {
    pub spender_subaccount: Option<Vec<u8>>,
    pub from: Account,
    pub to: Account,
    pub amount: u128,
    pub fee: Option<u128>,
    pub memo: Option<Vec<u8>>,
    pub created_at_time: Option<u64>,
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub enum TransferFromError {
    BadFee { expected_fee: u128 },
    BadBurn { min_burn_amount: u128 },
    InsufficientFunds { balance: u128 },
    InsufficientAllowance { allowance: u128 },
    TooOld,
    CreatedInFuture { ledger_time: u64 },
    Duplicate { duplicate_of: u128 },
    TemporarilyUnavailable,
    GenericError { error_code: u128, message: String },
}

#[derive(CandidType, Clone, Debug, Deserialize)]
pub struct Allowance {
    pub amount: u128,
    pub expires_at: Option<u64>,
}

impl Default for Allowance {
    fn default() -> Self {
        Self {
            amount: 0,
            expires_at: None,
        }
    }
}
