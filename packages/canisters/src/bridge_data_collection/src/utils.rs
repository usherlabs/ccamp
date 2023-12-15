use candid::Principal;
use ic_cdk::{api, id};
use lib::{
    constants::ZERO_ADDRESS,
    dc::{get_remittance_canister, Account},
    DataModel,
};

use crate::constants::CHAIN_IDENTIFIER;

// get the user balance for this canister which handles the matic chain
// for the matic native token
pub async fn get_user_canister_balance(account: String) -> u128 {
    let rc = get_remittance_canister();
    let (balance,): (Account,) = api::call::call(
        rc.canister_principal,
        "get_available_balance",
        (
            ZERO_ADDRESS.to_string(),
            CHAIN_IDENTIFIER.to_string(),
            account.to_string(),
            id(),
        ),
    )
    .await
    .unwrap();

    balance.balance as u128
}

// generate a payload that moves funds from the user's account to the canister's assigned account
pub async fn generate_mint_payload(account: String, amount: u128) -> Vec<DataModel> {
    let payload = &vec![
        DataModel {
            token: String::from(ZERO_ADDRESS).try_into().unwrap(),
            chain: String::from(CHAIN_IDENTIFIER).try_into().unwrap(),
            amount: -(amount as i64),
            account: account.try_into().unwrap(),
            action: lib::Action::Adjust,
        },
        DataModel {
            token: String::from(ZERO_ADDRESS).try_into().unwrap(),
            chain: String::from(CHAIN_IDENTIFIER).try_into().unwrap(),
            amount: amount as i64,
            account: String::from(ZERO_ADDRESS).try_into().unwrap(),
            action: lib::Action::Adjust,
        },
    ];

    payload.clone()
}

// generate a payload that moves funds from the canister's account to th euser's account
pub async fn generate_burn_payload(account: String, amount: u128) -> Vec<DataModel> {
    let payload = &vec![
        DataModel {
            token: String::from(ZERO_ADDRESS).try_into().unwrap(),
            chain: String::from(CHAIN_IDENTIFIER).try_into().unwrap(),
            amount: -(amount as i64),
            account: String::from(ZERO_ADDRESS).try_into().unwrap(),
            action: lib::Action::Adjust,
        },
        DataModel {
            token: String::from(ZERO_ADDRESS).try_into().unwrap(),
            chain: String::from(CHAIN_IDENTIFIER).try_into().unwrap(),
            amount: amount as i64,
            account: account.try_into().unwrap(),
            action: lib::Action::Adjust,
        },
    ];

    payload.clone()
}

pub async fn mint_tokens_to_caller(amount: u128, caller: &Principal) -> u128 {
    let token_canister = get_token_principal();
    let (minted_tokens,): (u128,) = api::call::call(token_canister, "mint", (caller, amount))
        .await
        .unwrap();

    minted_tokens
}

pub async fn burn_tokens_from_caller(amount: u128, caller: &Principal) -> Result<u128, String> {
    let token_canister = get_token_principal();
    let (burned_tokens,): (Result<u128, String>,) =
        api::call::call(token_canister, "burn", (caller, amount))
            .await
            .unwrap();

    burned_tokens
}

pub fn get_token_principal() -> Principal {
    crate::TOKEN_PRINCIPAL.with(|t| t.borrow().unwrap())
}

pub fn set_token_principal(token_principal: Principal) {
    crate::TOKEN_PRINCIPAL.with(|t| *t.borrow_mut() = Some(token_principal))
}
