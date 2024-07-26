pub mod config;
pub mod random;
pub mod types;
pub mod utils;
pub mod state;

use crate::{
    crypto::{
        self,
        ethereum::{recover_address_from_eth_signature, sign_message},
        vec_u8_to_string,
    },
    owner,
};
use candid::Principal;
use config::Config;
use ic_cdk::caller;
use types::Subscriber;
use state::*;

const REMITTANCE_EVENT: &str = "REMITTANCE";

/// Helper function to initialize the state of the remittance canister
pub fn init(env_opt: Option<config::Environment>) {
    owner::init_owner();
    random::init_ic_rand();

    // save the environment this is running in
    if let Some(env) = env_opt {
        CONFIG.with(|s| {
            let mut state = s.borrow_mut();
            *state = Config::from(env);
        })
    }
}

/// Get the owner of this contract
pub fn owner() -> String {
    owner::get_owner()
}

/// Get the name of this contract
pub fn name() -> String {
    format!("remittance canister")
}

/// Subscribe to the DC canister via its canister_id
pub async fn subscribe_to_dc(canister_id: Principal) {
    let subscriber = Subscriber {
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

/// Subscribe to the PCD canister via its canister_id
pub async fn subscribe_to_pdc(pdc_canister_id: Principal) {
    subscribe_to_dc(pdc_canister_id).await;
    IS_PDC_CANISTER.with(|is_pdc_canister| {
        is_pdc_canister.borrow_mut().insert(pdc_canister_id, true);
    });
}

/// validate and update user balance and canister balance 
pub fn update_remittance(
    new_remittances: Vec<types::DataModel>,
    dc_canister: Principal,
) -> Result<(), String> {
    let is_pdc =
        IS_PDC_CANISTER.with(|is_pdc_canister| is_pdc_canister.borrow().contains_key(&caller()));

    // add checks here to make sure that the input data is error free
    // if there is any error, return it to the calling dc canister
    if let Err(text) = utils::validate_remittance_data(is_pdc, &new_remittances, dc_canister) {
        return Err(text);
    }

    // process each 'MESSAGE' sent to the DC canister based on
    // the request type and if the canister calling the method is a request canister
    for new_remittance in new_remittances {
        // leave it named as underscore until we have implemented a use for the response

        let _: Result<(), String> = match new_remittance.action.clone() {
            types::Action::Adjust => {
                utils::update_balance(&new_remittance, dc_canister);
                Ok(())
            }
            types::Action::Deposit => {
                utils::update_balance(&new_remittance, dc_canister);
                // upon deposit, we increment the canister's balance of that token
                utils::update_canister_balance(
                    new_remittance.token,
                    new_remittance.chain,
                    dc_canister,
                    new_remittance.amount,
                );
                Ok(())
            }
            types::Action::Withdraw => {
                utils::confirm_withdrawal(
                    new_remittance.token.to_string(),
                    new_remittance.chain.to_string(),
                    new_remittance.account.to_string(),
                    new_remittance.amount.abs() as u64,
                    dc_canister,
                );
                // upon withdrawal we can remove the withdrawn amount from the canister's pool for that amount
                utils::update_canister_balance(
                    new_remittance.token,
                    new_remittance.chain,
                    dc_canister,
                    -new_remittance.amount,
                );
                Ok(())
            }
            types::Action::CancelWithdraw => {
                utils::cancel_withdrawal(
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

/// Create a remittance request
pub async fn remit(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
    amount: u64,
    proof: String,
) -> Result<types::RemittanceReply, Box<dyn std::error::Error>> {
    // make sure the 'proof' is a signature of the amount by the provided address
    let _derived_address = recover_address_from_eth_signature(proof, format!("{amount}"))?;

    // make sure the signature belongs to the provided account
    assert!(
        _derived_address == account.to_lowercase(),
        "INVALID_SIGNATURE"
    );
    // make sure the amount being remitted is none zero
    assert!(amount > 0, "AMOUNT < 0");

    // generate key values
    let chain: types::Chain = chain.try_into()?;
    let token: types::Wallet = token.try_into()?;
    let account: types::Wallet = account.try_into()?;

    let hash_key = (
        token.clone(),
        chain.clone(),
        account.clone(),
        dc_canister.clone(),
    );

    // check if there is a withheld 'balance' for this particular amount
    let withheld_balance = utils::get_remitted_balance(
        token.clone(),
        chain.clone(),
        account.clone(),
        dc_canister.clone(),
        amount,
    );

    let response: types::RemittanceReply;
    // if the amount exists in a withheld map then return the cached signature and nonce
    if withheld_balance.balance == amount {
        let message_hash = utils::hash_remittance_parameters(
            withheld_balance.nonce,
            amount,
            &account.to_string(),
            &chain.to_string(),
            &dc_canister.to_string(),
            &token.to_string(),
        );

        response = types::RemittanceReply {
            hash: vec_u8_to_string(&message_hash),
            signature: withheld_balance.signature.clone(),
            nonce: withheld_balance.nonce,
            amount,
        };
    } else {
        let nonce = random::get_random_number();
        let message_hash = utils::hash_remittance_parameters(
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
        )?
        .balance;

        // make sure this user actually has enough funds to withdraw
        if amount > balance {
            panic!("REMIT_AMOUNT:{amount} > AVAILABLE_BALANCE:{balance}")
        }

        // generate a signature for these parameters
        let config_store = CONFIG.with(|store| store.borrow().clone());
        let signature_reply = sign_message(&message_hash, &config_store).await?;
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
                types::WithheldAccount {
                    balance: amount,
                    signature: signature_string.clone(),
                    nonce,
                },
            );
        });
        // create response object
        response = types::RemittanceReply {
            hash: format!("0x{}", vec_u8_to_string(&message_hash)),
            signature: signature_string.clone(),
            nonce,
            amount,
        };
    }

    Ok(response)
}

/// Get the available balance of the account associated with a particular canister
pub fn get_available_balance(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
) -> Result<types::Account, Box<dyn std::error::Error>> {
    let chain: types::Chain = chain.try_into()?;
    let token: types::Wallet = token.try_into()?;
    let account: types::Wallet = account.try_into()?;
    // validate the address and the chain

    // get available balance for this key
    let amount = utils::get_available_balance(token, chain, account, dc_canister);

    Ok(amount)
}

/// Get the canister balance of the provided token for this particular chain
pub fn get_canister_balance(
    token: String,
    chain: String,
    dc_canister: Principal,
) -> Result<types::Account, Box<dyn std::error::Error>> {
    let chain: types::Chain = chain.try_into().unwrap();
    let token: types::Wallet = token.try_into().unwrap();
    // validate the address and the chain

    // get available balance for this key
    let amount = utils::get_canister_balance(token, chain, dc_canister);

    Ok(amount)
}

/// Get the witheld balance of this account on the specified canister and chain
pub fn get_withheld_balance(
    token: String,
    chain: String,
    account: String,
    dc_canister: Principal,
) -> Result<types::Account, Box<dyn std::error::Error>> {
    let chain: types::Chain = chain.try_into()?;
    let token: types::Wallet = token.try_into()?;
    let account: types::Wallet = account.try_into()?;

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

    Ok(types::Account { balance: sum })
}

/// Get the reciept of a successfull withdrawal
pub fn get_reciept(
    dc_canister: Principal,
    nonce: u64,
) -> Result<types::RemittanceReciept, Box<dyn std::error::Error>> {
    let key = (dc_canister.clone(), nonce.clone());
    Ok(REMITTANCE_RECIEPTS.with(|remittance_reciepts| {
        remittance_reciepts
            .borrow()
            .get(&key)
            .expect("RECIEPT_NOT_FOUND")
            .clone()
    }))
}

/// Get the public key associated with this particular canister
pub async fn public_key() -> Result<crypto::ecdsa::PublicKeyReply, Box<dyn std::error::Error>> {
    let config = CONFIG.with(|c| c.borrow().clone());

    let request = crypto::ecdsa::ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        key_id: config.key.to_key_id(),
    };

    let (res,): (crypto::ecdsa::ECDSAPublicKeyReply,) = ic_cdk::call(
        Principal::management_canister(),
        "ecdsa_public_key",
        (request,),
    )
    .await
    .map_err(|e| format!("ECDSA_PUBLIC_KEY_FAILED {}", e.1))?;

    let address = crypto::ethereum::get_address_from_public_key(res.public_key.clone())?;

    Ok(crypto::ecdsa::PublicKeyReply {
        sec1_pk: hex::encode(res.public_key),
        etherum_pk: address,
    })
}
