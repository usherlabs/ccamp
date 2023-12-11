use candid::Principal;
use ic_cdk::{caller, storage};
use ic_cdk_macros::*;

use core::panic;
use std::{cell::RefCell, collections::HashMap};
use utils::vec_u8_to_string;

mod owner;
mod random;
mod remittance;
mod utils;
use lib::{
    self,
    ethereum::{recover_address_from_eth_signature, sign_message}, remittance::{Config, Environment},
};

const REMITTANCE_EVENT: &str = "REMITTANCE";
thread_local! {
    static REMITTANCE: RefCell<remittance::AvailableBalanceStore> = RefCell::default();
    static WITHHELD_REMITTANCE: RefCell<remittance::WithheldBalanceStore> = RefCell::default();
    static WITHHELD_AMOUNTS: RefCell<remittance::WithheldAmountsStore> = RefCell::default();

    static IS_PDC_CANISTER: RefCell<HashMap<Principal, bool>> = RefCell::default();

    static DC_CANISTERS: RefCell<Vec<Principal>> = RefCell::default();

    static REMITTANCE_RECIEPTS: RefCell<remittance::RemittanceRecieptsStore> = RefCell::default();
    static CANISTER_BALANCE: RefCell<remittance::CanisterBalanceStore> = RefCell::default();

    static CONFIG: RefCell<Config> = RefCell::default();
}

// ----------------------------------- init hooks
#[init]
fn init(env_opt: Option<Environment>) {
    lib::owner::init_owner();
    random::init_ic_rand();

    // save the environment this is running in
    if let Some(env) = env_opt {
        CONFIG.with(|s| {
            let mut state = s.borrow_mut();
            *state = Config::from(env);
        })
    }
}
// ----------------------------------- init hooks

// get deployer of contract
#[query]
fn owner() -> String {
    lib::owner::get_owner()
}

// @dev test function
#[query]
fn name() -> String {
    format!("remittance canister")
}

// we call this method, with the id of the data_collection canister
// this then subscribes the remittance canister to "REMITTANCE" events from the data cannister
#[update]
async fn subscribe_to_dc(canister_id: Principal) {
    lib::owner::only_owner();
    let subscriber = lib::Subscriber {
        topic: REMITTANCE_EVENT.to_string(),
    };
    let _call_result: Result<(), _> = ic_cdk::call(canister_id, "subscribe", (subscriber,)).await;
    // update the list of all the publishers subscribed to while avoiding duplicates
    DC_CANISTERS.with(|dc_canister| {
        let mut borrowed_canister = dc_canister.borrow_mut();
        if !borrowed_canister.contains(&canister_id) {
            borrowed_canister.push(canister_id)
        }
    });
}

// we call this method to subscribe to a pdc
// it can only be called by the address who deployed the contract
#[update]
async fn subscribe_to_pdc(pdc_canister_id: Principal) {
    lib::owner::only_owner();
    subscribe_to_dc(pdc_canister_id).await;
    IS_PDC_CANISTER.with(|is_pdc_canister| {
        is_pdc_canister.borrow_mut().insert(pdc_canister_id, true);
    });
}

// this is an external function which is going to be called by  the data collection canister
// when there is a new data
#[update]
fn update_remittance(
    new_remittances: Vec<lib::DataModel>,
    dc_canister: Principal,
) -> Result<(), String> {
    owner::only_publisher();
    let is_pdc =
        IS_PDC_CANISTER.with(|is_pdc_canister| is_pdc_canister.borrow().contains_key(&caller()));

    // add checks here to make sure that the input data is error free
    // if there is any error, return it to the calling dc canister
    if let Err(text) = remittance::validate_remittance_data(is_pdc, &new_remittances, dc_canister) {
        return Err(text);
    }

    // process each 'MESSAGE' sent to the DC canister based on
    // the request type and if the canister calling the method is a request canister
    for new_remittance in new_remittances {
        // leave it named as underscore until we have implemented a use for the response

        let _: Result<(), String> = match new_remittance.action.clone() {
            lib::Action::Adjust => {
                remittance::update_balance(&new_remittance, dc_canister);
                Ok(())
            }
            lib::Action::Deposit => {
                remittance::update_balance(&new_remittance, dc_canister);
                // upon deposit, we increment the canister's balance of that token
                remittance::update_canister_balance(
                    new_remittance.token,
                    new_remittance.chain,
                    dc_canister,
                    new_remittance.amount,
                );
                Ok(())
            }
            lib::Action::Withdraw => {
                remittance::confirm_withdrawal(
                    new_remittance.token.to_string(),
                    new_remittance.chain.to_string(),
                    new_remittance.account.to_string(),
                    new_remittance.amount.abs() as u64,
                    dc_canister,
                );
                // upon withdrawal we can remove the withdrawn amount from the canister's pool for that amount
                remittance::update_canister_balance(
                    new_remittance.token,
                    new_remittance.chain,
                    dc_canister,
                    -new_remittance.amount,
                );
                Ok(())
            }
            lib::Action::CancelWithdraw => {
                remittance::cancel_withdrawal(
                    new_remittance.token.to_string(),
                    new_remittance.chain.to_string(),
                    new_remittance.account.to_string(),
                    new_remittance.amount.abs() as u64,
                    dc_canister,
                );
                Ok(())
            }
        };
    }

    Ok(())
}

// this function is called by the user to get their signature which they can use to claim funds from the network
#[update]
async fn remit(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
    amount: u64,
    proof: String,
) -> remittance::RemittanceReply {
    // make sure the 'proof' is a signature of the amount by the provided address
    let _derived_address =
        recover_address_from_eth_signature(proof, format!("{amount}")).expect("INVALID_SIGNATURE");

    // make sure the signature belongs to the provided account
    assert!(
        _derived_address == account.to_lowercase(),
        "INVALID_SIGNATURE"
    );
    // make sure the amount being remitted is none zero
    assert!(amount > 0, "AMOUNT < 0");

    // generate key values
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let hash_key = (
        token.clone(),
        chain.clone(),
        account.clone(),
        dc_canister.clone(),
    );

    // check if there is a withheld 'balance' for this particular amount
    let withheld_balance = remittance::get_remitted_balance(
        token.clone(),
        chain.clone(),
        account.clone(),
        dc_canister.clone(),
        amount,
    );

    let response: remittance::RemittanceReply;
    // if the amount exists in a withheld map then return the cached signature and nonce
    if withheld_balance.balance == amount {
        let message_hash = remittance::hash_remittance_parameters(
            withheld_balance.nonce,
            amount,
            &account.to_string(),
            &chain.to_string(),
            &dc_canister.to_string(),
            &token.to_string(),
        );

        response = remittance::RemittanceReply {
            hash: vec_u8_to_string(&message_hash),
            signature: withheld_balance.signature.clone(),
            nonce: withheld_balance.nonce,
            amount,
        };
    } else {
        let nonce = random::get_random_number();
        let message_hash = remittance::hash_remittance_parameters(
            nonce,
            amount,
            &account.to_string(),
            &chain.to_string(),
            &dc_canister.to_string(),
            &token.to_string(),
        );
        let balance = get_available_balance(
            token.to_string(),
            chain.to_string(),
            account.to_string(),
            dc_canister.clone(),
        )
        .balance;

        // make sure this user actually has enough funds to withdraw
        if amount > balance {
            panic!("REMIT_AMOUNT:{amount} > AVAILABLE_BALANCE:{balance}")
        }

        // generate a signature for these parameters
        let config_store = CONFIG.with(|store| store.borrow().clone());
        let signature_reply = sign_message(&message_hash, &config_store)
            .await
            .expect("ERROR_SIGNING_MESSAGE");
        let signature_string = format!("0x{}", signature_reply.signature_hex);

        // deduct amount to remit from main balance
        REMITTANCE.with(|remittance| {
            if let Some(existing_data) = remittance.borrow_mut().get_mut(&hash_key) {
                existing_data.balance = existing_data.balance - amount;
            }
        });
        // add amount to mapping (token, chain, recipient) => [amount_1, amount_2, amount_3]
        // to keep track of individual amounts remitted per (token, chain, recipient) combination
        WITHHELD_AMOUNTS.with(|withheld_amount| {
            // Append value to existing entry or create new entry
            withheld_amount
                .borrow_mut()
                .entry(hash_key.clone())
                .or_insert(Vec::new())
                .push(amount);
        });
        // update the withheld balance of the said user and generate a new signature for it
        WITHHELD_REMITTANCE.with(|withheld| {
            let mut withheld_remittance_store = withheld.borrow_mut();
            withheld_remittance_store.insert(
                (
                    token.clone(),
                    chain.clone(),
                    account.clone(),
                    dc_canister.clone(),
                    amount,
                ),
                remittance::WithheldAccount {
                    balance: amount,
                    signature: signature_string.clone(),
                    nonce,
                },
            );
        });
        // create response object
        response = remittance::RemittanceReply {
            hash: format!("0x{}", vec_u8_to_string(&message_hash)),
            signature: signature_string.clone(),
            nonce,
            amount,
        };
    }

    response
}

// use this function to get the un remitted balance of the 'account' provided
// i.e the portion of their balance which has not been claimed or is in the process of being claimed
#[query]
fn get_available_balance(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
) -> remittance::Account {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();
    // validate the address and the chain

    // get available balance for this key
    let amount = remittance::get_available_balance(token, chain, account, dc_canister);

    amount
}

#[query]
fn get_canister_balance(
    token: String,
    chain: String,
    dc_canister: Principal,
) -> remittance::Account {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    // validate the address and the chain

    // get available balance for this key
    let amount = remittance::get_canister_balance(token, chain, dc_canister);

    amount
}

// the users use this function to get the withheld balance
// i.e the balance which has been deducted from the main balance
// because it can be potentially claimed from the smart contract
#[query]
fn get_withheld_balance(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
) -> remittance::Account {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let existing_key = (token.clone(), chain.clone(), account.clone(), dc_canister);

    // sum up all the amounts in the withheld_amount value of this key
    let sum = WITHHELD_AMOUNTS.with(|withheld_amount| {
        let withheld_amount = withheld_amount.borrow();
        let values = withheld_amount.get(&existing_key);

        match values {
            Some(vec) => vec.iter().sum::<u64>(),
            None => 0,
        }
    });

    remittance::Account { balance: sum }
}

#[query]
async fn get_reciept(dc_canister: Principal, nonce: u64) -> remittance::RemittanceReciept {
    let key = (dc_canister.clone(), nonce.clone());
    REMITTANCE_RECIEPTS.with(|remittance_reciepts| {
        remittance_reciepts
            .borrow()
            .get(&key)
            .expect("RECIEPT_NOT_FOUND")
            .clone()
    })
}

#[update]
async fn public_key() -> lib::ecdsa::PublicKeyReply {
    let config = crate::CONFIG.with(|c| c.borrow().clone());

    let request = lib::ecdsa::ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        key_id: config.key.to_key_id(),
    };

    let (res,): (lib::ecdsa::ECDSAPublicKeyReply,) = ic_cdk::call(
        Principal::management_canister(),
        "ecdsa_public_key",
        (request,),
    )
    .await
    .map_err(|e| format!("ECDSA_PUBLIC_KEY_FAILED {}", e.1))
    .unwrap();

    let address = lib::ethereum::get_address_from_public_key(res.public_key.clone())
        .expect("INVALID_PUBLIC_KEY");

    lib::ecdsa::PublicKeyReply {
        sec1_pk: hex::encode(res.public_key),
        etherum_pk: address,
    }
}

// --------------------------- upgrade hooks ------------------------- //
#[pre_upgrade]
fn pre_upgrade() {
    // clone all important variables
    let cloned_available_balance_store = REMITTANCE.with(|store| store.borrow().clone());
    let cloned_witheld_balance_store = WITHHELD_REMITTANCE.with(|store| store.borrow().clone());
    let cloned_witheld_amounts = WITHHELD_AMOUNTS.with(|store| store.borrow().clone());
    let cloned_is_pdc_canister = IS_PDC_CANISTER.with(|store| store.borrow().clone());
    let dc_canisters = DC_CANISTERS.with(|store| store.borrow().clone());
    let remittance_reciepts_store = REMITTANCE_RECIEPTS.with(|store| store.borrow().clone());
    let config_store = CONFIG.with(|store| store.borrow().clone());
    // save cloned memory
    storage::stable_save((
        cloned_available_balance_store,
        cloned_witheld_balance_store,
        cloned_witheld_amounts,
        cloned_is_pdc_canister,
        dc_canisters,
        remittance_reciepts_store,
        config_store,
    ))
    .unwrap()
}

#[post_upgrade]
async fn post_upgrade() {
    lib::owner::init_owner();
    random::init_ic_rand();
    // load the variables from memory
    let (
        cloned_available_balance_store,
        cloned_witheld_balance_store,
        cloned_witheld_amounts_store,
        cloned_is_pdc_canister,
        cloned_dc_canisters,
        cloned_remittance_reciepts,
        cloned_config,
    ): (
        remittance::AvailableBalanceStore,
        remittance::WithheldBalanceStore,
        remittance::WithheldAmountsStore,
        HashMap<Principal, bool>,
        Vec<Principal>,
        remittance::RemittanceRecieptsStore,
        Config,
    ) = storage::stable_restore().unwrap();
    //  restore by reassigning to vairiables
    REMITTANCE.with(|r| *r.borrow_mut() = cloned_available_balance_store);
    WITHHELD_REMITTANCE.with(|wr| *wr.borrow_mut() = cloned_witheld_balance_store);
    WITHHELD_AMOUNTS.with(|wa| *wa.borrow_mut() = cloned_witheld_amounts_store);
    IS_PDC_CANISTER.with(|ipc| *ipc.borrow_mut() = cloned_is_pdc_canister);
    DC_CANISTERS.with(|dc| *dc.borrow_mut() = cloned_dc_canisters);
    REMITTANCE_RECIEPTS.with(|rr| *rr.borrow_mut() = cloned_remittance_reciepts);
    CONFIG.with(|c| *c.borrow_mut() = cloned_config);
}
// --------------------------- upgrade hooks ------------------------- //
