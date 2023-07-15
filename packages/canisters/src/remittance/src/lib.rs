use candid::Principal;
use easy_hasher::easy_hasher;
use ic_cdk_macros::*;
use lib;
use std::cell::RefCell;
use utils::vec_u8_to_string;

mod ecdsa;
mod ethereum;
mod owner;
mod random;
mod remittance;
mod utils;

// TODO research on preserving state of dc and r canisters when upgrade happens

const REMITTANCE_EVENT: &str = "REMITTANCE";
thread_local! {
    static REMITTANCE: RefCell<remittance::AvailableBalanceStore> = RefCell::default();
    static WITHELD_REMITTANCE: RefCell<remittance::WitheldBalanceStore> = RefCell::default();
    static WITHELD_AMOUNTS: RefCell<remittance::WitheldAmountsStore> = RefCell::default();
    static PUBLISHERS: RefCell<Vec<Principal>> = RefCell::default();
}

// ----------------------------------- init and upgrade hooks
#[init]
fn init() {
    owner::init_owner();
    random::init_ic_rand();
}

// upon upgrade of contracts, state is  lost
// so we need to reinitialize important variables here
#[post_upgrade]
fn upgrade() {
    init();
}
// ----------------------------------- init and upgrade hooks

// get deployer of contract
#[query]
fn owner() -> String {
    owner::get_owner()
}

// @dev test function
#[query]
fn name() -> String {
    format!("remittance canister")
}

// we call this method, with the id of the data_collection canister
// this then subscribes the remittance canister to "REMITTANCE" events from the data cannister
#[update]
async fn setup_subscribe(publisher_id: Principal) {
    owner::only_owner();
    let subscriber = lib::Subscriber {
        topic: REMITTANCE_EVENT.to_string(),
    };
    let _call_result: Result<(), _> = ic_cdk::call(publisher_id, "subscribe", (subscriber,)).await;
    // update the list of all the publishers subscribed to while avoiding duplicates
    PUBLISHERS.with(|publisher| {
        let mut borrowed_publisher = publisher.borrow_mut();
        if !borrowed_publisher.contains(&publisher_id) {
            borrowed_publisher.push(publisher_id)
        }
    });
}

// this is an external function which is going to be called by  the data collection canister
// when there is a new data
#[update]
fn update_remittance(new_remittances: Vec<lib::DataModel>) -> Result<(), String> {
    owner::only_publisher();
    // TODO derive this value by comparing the caller to a list of registered protocol canisters
    let is_protocol_dc = false;

    // add checks here to make sure that the input data is error free
    // if there is any error, return it to the calling dc canister
    if let Err(text) = remittance::validate_remittance_data(is_protocol_dc, &new_remittances) {
        // TODO to return error or throw error
        // return Err(text);
        panic!("{text}");
    }

    // process each 'MESSAGE' sent to the DC canister based on
    // the request type and if the canister calling the method is a request canister
    for new_remittance in new_remittances {
        // leave it named as underscore until we have implemented a use for the response
        let _ = match (is_protocol_dc, new_remittance.action.clone()) {
            (_, lib::Action::Adjust) => {
                remittance::update_balance(new_remittance);
                Ok(())
            }
            // ignore every other condition we have not created yet
            _ => Err("INVALID_ACTION"),
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
    amount: u64,
) -> remittance::RemittanceReply {
    // make sure the amount being remitted is none zero
    assert!(amount > 0, "AMOUNT < 0");
    // generate key values
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let hash_key = (token.clone(), chain.clone(), account.clone());

    // check if there is a witheld 'balance' for this particular amount
    let witheld_balance =
        remittance::get_remitted_balance(token.clone(), chain.clone(), account.clone(), amount);

    let response: remittance::RemittanceReply;
    // if the amount exists in a witheld map then return the cached signature and nonce
    if witheld_balance.balance == amount {
        let (bytes_hash, _) = remittance::produce_remittance_hash(
            witheld_balance.nonce,
            amount,
            &account.to_string(),
            &chain.to_string(),
        );

        response = remittance::RemittanceReply {
            hash: vec_u8_to_string(&easy_hasher::raw_keccak256(bytes_hash).to_vec()),
            signature: witheld_balance.signature.clone(),
            nonce: witheld_balance.nonce,
            amount,
        };
    } else {
        let nonce = random::get_random_number();
        let (bytes_hash, _) = remittance::produce_remittance_hash(
            nonce,
            amount,
            &account.to_string(),
            &chain.to_string(),
        );
        let balance =
            get_available_balance(token.to_string(), chain.to_string(), account.to_string())
                .balance;

        // make sure this user actually has enough funds to withdraw
        assert!(balance > amount, "REMIT_AMOUNT > AVAILABLE_BALANCE");

        // generate a signature for these parameters
        let signature_reply = ethereum::sign_message(&bytes_hash)
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
        WITHELD_AMOUNTS.with(|witheld_amount| {
            // Append value to existing entry or create new entry
            witheld_amount
                .borrow_mut()
                .entry(hash_key.clone())
                .or_insert(Vec::new())
                .push(amount);
        });
        // update the witheld balance of the said user and generate a new signature for it
        WITHELD_REMITTANCE.with(|witheld| {
            let mut witheld_remittance_store = witheld.borrow_mut();
            witheld_remittance_store.insert(
                (token.clone(), chain.clone(), account.clone(), amount),
                remittance::WitheldAccount {
                    balance: amount,
                    signature: signature_string.clone(),
                    nonce,
                },
            );
        });
        // create response object
        response = remittance::RemittanceReply {
            hash: vec_u8_to_string(&easy_hasher::raw_keccak256(bytes_hash).to_vec()),
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
fn get_available_balance(token: String, chain: String, account: String) -> remittance::Account {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();
    // validate the address and the chain

    // get available balance for this key
    let amount = remittance::get_available_balance(token, chain, account);

    amount
}

// the users use this function to get the witheld balance
// i.e the balance which has been deducted from the main balance
// because it can be potentially claimed from the smart contract
#[query]
fn get_witheld_balance(token: String, chain: String, account: String) -> remittance::Account {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let existing_key = (token.clone(), chain.clone(), account.clone());

    // sum up all the amounts in the witheld_amount value of this key
    let sum = WITHELD_AMOUNTS.with(|witheld_amount| {
        let witheld_amount = witheld_amount.borrow();
        let values = witheld_amount.get(&existing_key);

        match values {
            Some(vec) => vec.iter().sum::<u64>(),
            None => 0,
        }
    });

    remittance::Account { balance: sum }
}

#[update]
fn clear_witheld_balance(token: String, chain: String, account: String) -> remittance::Account {
    let chain: lib::Chain = chain.try_into().unwrap();
    let token: lib::Wallet = token.try_into().unwrap();
    let account: lib::Wallet = account.try_into().unwrap();

    let hash_key = (token.clone(), chain.clone(), account.clone());

    let redeemed_balance =
        get_witheld_balance(token.to_string(), chain.to_string(), account.to_string());
    // if this user has some pending withdrawals for these parameters
    if redeemed_balance.balance > 0 {
        // then for each amount delete the entry/cache from the witheld balance
        WITHELD_AMOUNTS.with(|witheld_amount| {
            let mut mut_witheld_amount = witheld_amount.borrow_mut();
            mut_witheld_amount
                .get(&hash_key)
                .unwrap()
                .iter()
                .for_each(|amount| {
                    WITHELD_REMITTANCE.with(|witheld_remittance| {
                        witheld_remittance.borrow_mut().remove(&(
                            token.clone(),
                            chain.clone(),
                            account.clone(),
                            *amount,
                        ));
                    })
                });
            mut_witheld_amount.remove(&hash_key);
        });
        // add the witheld total back to the available balance
        REMITTANCE.with(|remittance| {
            if let Some(existing_data) = remittance.borrow_mut().get_mut(&hash_key) {
                existing_data.balance = existing_data.balance + redeemed_balance.balance;
            }
        });
    }
    return redeemed_balance;
}

#[update]
async fn public_key() -> Result<ecdsa::PublicKeyReply, String> {
    let request = ecdsa::ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        // TODO set this as an environment variable
        key_id: ecdsa::EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };

    let (res,): (ecdsa::ECDSAPublicKeyReply,) = ic_cdk::call(
        Principal::management_canister(),
        "ecdsa_public_key",
        (request,),
    )
    .await
    .map_err(|e| format!("ECDSA_PUBLIC_KEY_FAILED {}", e.1))?;

    let address =
        ethereum::get_address_from_public_key(res.public_key.clone()).expect("INVALID_PUBLIC_KEY");

    Ok(ecdsa::PublicKeyReply {
        sec1_pk: hex::encode(res.public_key),
        etherum_pk: address,
    })
}
