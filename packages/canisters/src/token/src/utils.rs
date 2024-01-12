use crate::{
    config::{DECIMALS, FEE, MAX_MEMO_LENGTH, TOKEN_NAME, TOKEN_SYMBOL},
    types::{Account, Allowance},
};
use candid::{CandidType, Principal};
use std::collections::HashMap;

pub type MetaDataType = HashMap<String, Variant>;

#[derive(CandidType, Debug)]
pub enum Variant {
    Nat(u128),
    Text(String),
}

#[derive(CandidType, Debug)]
pub struct SupportedStandards {
    url: String,
    name: String,
}

pub fn generate_metadata() -> MetaDataType {
    let mut new_map = HashMap::new();

    new_map.insert(String::from("icrc1:decimals"), Variant::Nat(DECIMALS));
    new_map.insert(
        String::from("icrc1:name"),
        Variant::Text(String::from(TOKEN_NAME)),
    );
    new_map.insert(
        String::from("icrc1:symbol"),
        Variant::Text(String::from(TOKEN_SYMBOL)),
    );
    new_map.insert(String::from("icrc1:fee"), Variant::Nat(FEE));
    new_map.insert(
        String::from("icrc1:max_memo_length"),
        Variant::Nat(MAX_MEMO_LENGTH),
    );

    new_map
}

pub fn generate_supported_standards() -> Vec<SupportedStandards> {
    vec![
        SupportedStandards {
            url: String::from("https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-1"),
            name: String::from("ICRC-1"),
        },
        SupportedStandards {
            url: String::from("https://github.com/dfinity/ICRC-1/tree/main/standards/ICRC-2"),
            name: String::from("ICRC-2"),
        },
    ]
}

pub fn only_admin_canister() {
    let admin_principal = crate::ADMIN_PRINCIPAL.with(|f| f.borrow().clone().unwrap());
    let caller = ic_cdk::caller();
    let owner = Principal::from_text(lib::owner::get_owner()).unwrap();

    if caller != admin_principal && caller != owner {
        panic!("NOT_AUTHORIZED")
    }
}

pub fn internal_burn(account: Account, burn_amount: u128) -> Result<u128, String> {
    let user_balance = crate::icrc1_balance_of(account);
    if user_balance < burn_amount {
        return Err("USER_BALANCE < BURN_AMOUNT".to_string());
    }

    crate::BALANCES.with(|balance| {
        let account_balance = match balance.borrow().get(&account.owner) {
            Some(original_balance) => original_balance.clone(),
            None => 0,
        };

        balance
            .borrow_mut()
            .insert(account.owner, account_balance - burn_amount);
    });
    crate::TOTAL_SUPPLY.with(|ts| *ts.borrow_mut() -= burn_amount);

    Ok(burn_amount)
}

pub fn internal_mint(account: Account, amount: u128) {
    let new_balance = crate::BALANCES.with(|balance| {
        let account_balance = match balance.borrow().get(&account.owner) {
            Some(original_balance) => original_balance.clone(),
            None => 0,
        };

        balance
            .borrow_mut()
            .insert(account.owner, account_balance + amount);
    });

    crate::TOTAL_SUPPLY.with(|ts| *ts.borrow_mut() += amount);

    new_balance
}

pub fn internal_transfer(sender: Principal, recipient: Principal, amount: u128) -> u128 {
    // get the recipient
    let sender_balance = crate::icrc1_balance_of(sender.into());
    // make sure the caller has enough balance to send to another person
    if sender_balance < amount {
        panic!("BALANCE < AMOUNT")
    }

    let new_balance: u128 = sender_balance - amount;

    crate::BALANCES.with(|balance| {
        balance.borrow_mut().insert(sender, new_balance);

        *balance
            .borrow_mut()
            .entry(recipient.to_owned())
            .or_default() += amount;
    });

    new_balance
}

pub fn get_allowance(owner: Principal, spender: Principal) -> Allowance {
    let allowance = crate::APPROVALS.with(|approvals| {
        approvals
            .borrow()
            .get(&(owner, spender))
            .unwrap_or(&Allowance::default())
            .clone()
    });

    allowance
}

pub fn set_allowance(owner: Principal, spender: Principal, amount: u128, expires_at: Option<u64>) {
    crate::APPROVALS.with(|approvals| {
        let mut approvals = approvals.borrow_mut();

        approvals.insert(
            (owner, spender),
            Allowance {
                amount: amount,
                expires_at: expires_at,
            },
        );
    });
}
