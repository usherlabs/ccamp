use candid::Principal;
use easy_hasher::easy_hasher;
use ic_cdk_macros::*;
use lib;
use std::cell::{Ref, RefCell};
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
fn update_remittance(new_remittances: Vec<lib::DataModel>) {
    owner::only_publisher();
    for new_remittance in new_remittances {
        remittance::update_balance(new_remittance);
    }
}

// use this to get the signature, nonce and amount to get some remittance for the input params
#[update]
async fn remit(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
    amount: u64,
) -> remittance::RemittanceReply {
    // make sure the amount being remitted is none zero
    assert!(amount > 0, "AMOUNT < 0");
    // generate key values
    let chain = lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");
    let hash_key = (ticker.clone(), chain.clone(), recipient_address.clone());

    // check if there is a witheld 'balance' for this particular amount
    let witheld_balance = remittance::get_remitted_balance(
        ticker.clone(),
        chain_name.clone(),
        chain_id.clone(),
        recipient_address.clone(),
        amount,
    );

    let response: remittance::RemittanceReply;
    // if the amount exists in a witheld map then return the cached signature and nonce
    if witheld_balance.balance == amount {
        let (bytes_hash, _) = remittance::produce_remittance_hash(
            witheld_balance.nonce,
            amount,
            &recipient_address[..],
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
            &recipient_address[..],
            &chain.to_string(),
        );
        let balance = get_available_balance(
            ticker.clone(),
            chain_name.clone(),
            chain_id.clone(),
            recipient_address.clone(),
        )
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
        // add amount to mapping (ticker, chain, recipient) => [amount_1, amount_2, amount_3]
        // to keep track of individual amounts remitted per (ticker, chain, recipient) combination
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
                (
                    ticker.clone(),
                    chain.clone(),
                    recipient_address.clone(),
                    amount,
                ),
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

#[query]
fn get_available_balance(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
) -> remittance::Account {
    // validate the address and the chain
    if recipient_address.len() != 42 {
        panic!("INVALID_ADDRESS")
    };
    let chain = lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");
    // validate the address and the chain

    let amount = REMITTANCE.with(|remittance| {
        let existing_key = (ticker, chain, recipient_address.clone());
        remittance
            .borrow()
            .get(&existing_key)
            .cloned()
            .unwrap_or_default()
    });

    amount
}

#[query]
fn get_witheld_balance(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
) -> remittance::Account {
    // validate the address and the chain
    if recipient_address.len() != 42 {
        panic!("INVALID_ADDRESS")
    };
    let chain: lib::Chain =
        lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");
    let existing_key = (ticker.clone(), chain.clone(), recipient_address.clone());

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
fn clear_witheld_balance(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
) -> remittance::Account {
    let chain: lib::Chain =
        lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");
    let hash_key = (ticker.clone(), chain.clone(), recipient_address.clone());

    let redeemed_balance = get_witheld_balance(
        ticker.clone(),
        chain_name.clone(),
        chain_id.clone(),
        recipient_address.clone(),
    );
    // if this user has some pending withdrawals for these parameters
    if redeemed_balance.balance > 0 {
        // then for each amount delete the entry from th ewitheld balane
        WITHELD_AMOUNTS.with(|witheld_amount| {
            let mut mut_witheld_amount = witheld_amount.borrow_mut();
            mut_witheld_amount
                .get(&hash_key)
                .unwrap()
                .iter()
                .for_each(|amount| {
                    WITHELD_REMITTANCE.with(|witheld_remittance| {
                        witheld_remittance.borrow_mut().remove(&(
                            ticker.clone(),
                            chain.clone(),
                            recipient_address.clone(),
                            *amount,
                        ));
                    })
                });
            mut_witheld_amount.remove(&hash_key);
        });
        // add the total back to the available balance
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
