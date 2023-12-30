use candid::Principal;
use ic_cdk::api::time;
use ic_cdk::{caller, storage};

use config::{DECIMALS, FEE, INITIAL_SUPPLY, TOKEN_NAME, TOKEN_SYMBOL};
use ic_cdk_macros::{init, post_upgrade, pre_upgrade, query, update};
use std::{cell::RefCell, collections::HashMap};
use types::{
    Account, Allowance, AllowanceArgs, ApprovalType, ApproveArgs, ApproveError, TransferArgs,
    TransferError, TransferFromArgs, TransferFromError,
};
use utils::{
    generate_metadata, generate_supported_standards, get_allowance, internal_burn, internal_mint,
    internal_transfer, only_admin_canister, set_allowance, MetaDataType, SupportedStandards,
};

mod config;
mod types;
mod utils;

thread_local! {
    static TOTAL_SUPPLY: RefCell<u128> = RefCell::default();
    static BALANCES: RefCell<HashMap<Principal,u128>> = RefCell::default();
    static ADMIN_PRINCIPAL: RefCell<Option<Principal>> = RefCell::default();
    static APPROVALS: RefCell<ApprovalType> = RefCell::default();
}

// ----------------------------------- init hooks
#[init]
fn init() {
    lib::owner::init_owner();
    let account_principal = ic_cdk::caller();

    // mint a certan value to the deployer account and admin canister
    internal_mint(Account::from(account_principal), INITIAL_SUPPLY);
}
// ----------------------------------- init hooks

// get deployer of contract
#[query]
fn owner() -> String {
    lib::owner::get_owner()
}

#[query]
fn icrc1_metadata() -> MetaDataType {
    generate_metadata()
}

#[query]
fn icrc1_name() -> String {
    String::from(TOKEN_NAME)
}

#[query]
fn icrc1_symbol() -> String {
    String::from(TOKEN_SYMBOL)
}

#[query]
fn icrc1_decimals() -> u128 {
    DECIMALS
}

#[query]
fn icrc1_fee() -> u128 {
    FEE
}

#[query]
fn icrc1_total_supply() -> u128 {
    TOTAL_SUPPLY.with(|ts| ts.borrow().clone())
}

#[query]
fn icrc1_minting_account() -> Account {
    let deployer = Principal::from_text(lib::owner::get_owner()).unwrap();
    Account::from(deployer)
}

#[query]
fn get_dc_canister() -> Principal {
    ADMIN_PRINCIPAL.with(|ap| ap.borrow().clone().expect("DC_CANISTER_NOT_SET"))
}

#[query]
fn balance() -> u128 {
    let account_principal = ic_cdk::caller();

    icrc1_balance_of(Account::from(account_principal))
}

#[query]
fn icrc1_supported_standards() -> Vec<SupportedStandards> {
    generate_supported_standards()
}

#[query]
fn total_supply() -> u128 {
    TOTAL_SUPPLY.with(|ts| ts.borrow().clone())
}

#[query]
fn icrc1_balance_of(account: Account) -> u128 {
    let balance = BALANCES.with(|balance| {
        let balance_map = balance.borrow().clone();
        let val = balance_map.get(&account.owner).or(Some(&0)).unwrap();
        val.clone()
    });

    balance
}

#[update]
fn icrc1_transfer(args: TransferArgs) -> Result<u128, TransferError> {
    // get the caller
    let caller = ic_cdk::caller();
    let recipient = args.to.owner;
    if caller == recipient {
        return Err(TransferError::GenericError {
            error_code: u128::default(),
            message: String::from("CALLER IS RECIPIENT"),
        });
    }

    // get the recipient
    let caller_balance = icrc1_balance_of(caller.into());
    // make sure the caller has enough balance to send to another person
    if caller_balance < args.amount {
        return Err(TransferError::GenericError {
            error_code: u128::default(),
            message: String::from("BALANCE < AMOUNT"),
        });
    }

    let _ = internal_transfer(caller, recipient, args.amount);
    Ok(args.amount)
}

#[update]
fn icrc2_transfer_from(transfer_from_args: TransferFromArgs) -> Result<u128, TransferFromError> {
    let spender = ic_cdk::caller();
    let owner = transfer_from_args.from.owner;
    let amount = transfer_from_args.amount;
    let recipient = transfer_from_args.to.owner;

    if owner == recipient {
        panic!("SENDER ==RECIPIENT")
    }

    // confirm allowance
    let user_allowance = get_allowance(owner, spender).amount;
    if user_allowance < transfer_from_args.amount {
        return Err(TransferFromError::InsufficientAllowance {
            allowance: user_allowance,
        });
    }

    // confirm balance of owner
    let owner_balance = icrc1_balance_of(Account::from(owner));
    if owner_balance < amount {
        return Err(TransferFromError::InsufficientFunds {
            balance: owner_balance,
        });
    }

    // move funds
    internal_transfer(owner, recipient, amount);

    // edit allowance
    let new_allowance = user_allowance - amount;
    set_allowance(owner, spender, new_allowance, None);

    // return amount
    Ok(amount)
}

#[update]
fn icrc2_approve(approve_args: ApproveArgs) -> Result<u128, ApproveError> {
    let owner = caller();
    let spender = (&approve_args.spender).clone().owner;

    // make sure the preconditions are met
    if owner == spender {
        panic!("SPENDER == OWNER")
    }
    if let Some(expected_allowance) = approve_args.expected_allowance {
        let current_allowance = icrc2_allowance(AllowanceArgs {
            account: Account::from(owner),
            spender: Account::from(spender),
        })
        .amount;

        if current_allowance != expected_allowance as u128 {
            return Err(ApproveError::AllowanceChanged {
                current_allowance: current_allowance as u64,
            });
        }
    }
    if let Some(expires_at) = approve_args.expires_at {
        let ledger_time = time();
        if expires_at > time() {
            return Err(ApproveError::Expired {
                ledger_time: ledger_time,
            });
        }
    }
    // make sure the preconditions are met
    set_allowance(
        owner,
        spender,
        approve_args.amount as u128,
        approve_args.expires_at,
    );

    Ok(approve_args.amount.into())
}

#[update]
fn icrc2_allowance(allowance_args: AllowanceArgs) -> Allowance {
    let owner = (&allowance_args.account.owner).clone();
    let spender = (&allowance_args.account.owner).clone();

    get_allowance(owner, spender)
}

#[update]
fn mint(account_principal: Principal, amount: u128) -> u128 {
    only_admin_canister();

    internal_mint(Account::from(account_principal), amount);

    amount
}

#[update]
fn burn(account_principal: Principal, amount: u128) -> Result<u128, String> {
    only_admin_canister();
    internal_burn(Account::from(account_principal), amount)
}

#[update]
fn set_dc_canister(dc_principal: Principal) {
    ADMIN_PRINCIPAL.with(|ap| *ap.borrow_mut() = Some(dc_principal))
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    let cloned_balances = BALANCES.with(|rc| rc.borrow().clone());
    let cloned_supply = TOTAL_SUPPLY.with(|rc| rc.borrow().clone());
    let cloned_admin = ADMIN_PRINCIPAL.with(|rc| rc.borrow().clone());
    let cloned_approvals = APPROVALS.with(|rc| rc.borrow().clone());

    storage::stable_save((
        cloned_balances,
        cloned_supply,
        cloned_admin,
        cloned_approvals,
    ))
    .unwrap()
}

#[post_upgrade]
async fn post_upgrade() {
    lib::owner::init_owner();

    let (cloned_balances, cloned_supply, cloned_admin, cloned_approvals): (
        HashMap<Principal, u128>,
        u128,
        Option<Principal>,
        ApprovalType,
    ) = storage::stable_restore().unwrap();

    BALANCES.with(|r| *r.borrow_mut() = cloned_balances);
    TOTAL_SUPPLY.with(|r| *r.borrow_mut() = cloned_supply);
    ADMIN_PRINCIPAL.with(|r| *r.borrow_mut() = cloned_admin);
    APPROVALS.with(|r| *r.borrow_mut() = cloned_approvals);
}
