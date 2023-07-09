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
    static REMITTANCE: RefCell<remittance::Store> = RefCell::default();
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
fn greet(name: String) -> String {
    format!("Hello data_collection canister, {}!", name)
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
        update_balance(new_remittance);
    }
}

// it essentially uses the mapping (ticker, chain, recipientaddress) => {DataModel}
// so if an entry exists for a particular combination of (ticker, chain, recipientaddress)
// then the price is updated, otherwise the entry is created
fn update_balance(new_remittance: lib::DataModel) {
    owner::only_publisher();
    REMITTANCE.with(|remittance| {
        let mut remittance_store = remittance.borrow_mut();

        let hash_key = (
            new_remittance.ticker.clone(),
            new_remittance.chain.clone(),
            new_remittance.recipient_address.clone(),
        );

        if let Some(existing_data) = remittance_store.get_mut(&hash_key) {
            existing_data.balance =
                (existing_data.balance as i64 + new_remittance.amount as i64) as u64;
        } else {
            remittance_store.insert(
                hash_key,
                remittance::Account {
                    balance: new_remittance.amount as u64,
                },
            );
        }
    });
}

// use this to get the signature, nonce and amount to get some remittance for the input params
#[update]
async fn get_remittance(
    ticker: String,
    chain_name: String,
    chain_id: String,
    recipient_address: String,
) -> remittance::RemittanceReply {
    let amount = get_balance(
        ticker,
        chain_name.clone(),
        chain_id.clone(),
        recipient_address.clone(),
    )
    .balance;
    let nonce = random::get_random_number();
    let chain = lib::Chain::from_chain_details(&chain_name, &chain_id).expect("INVALID_CHAIN");

    let (bytes_hash, _) = remittance::produce_remittance_hash(
        nonce,
        amount,
        &recipient_address[..],
        &chain.to_string(),
    );
    let message = ethereum::sign_message(&bytes_hash)
        .await
        .expect("ERROR_SIGNING_MESSAGE");

    remittance::RemittanceReply {
        hash: vec_u8_to_string(&easy_hasher::raw_keccak256(bytes_hash).to_vec()),
        signature: format!("0x{}", message.signature_hex),
        nonce,
        amount,
    }
}

#[query]
fn get_balance(
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
            .expect("REMITTANCE_NOT_FOUND ")
            .clone()
    });

    amount
}

async fn derive_pk() -> Vec<u8> {
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
    .map_err(|e| format!("ECDSA_PUBLIC_KEY_FAILED {}", e.1))
    .unwrap();

    res.public_key
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

#[update]
async fn sign(message: String) -> Result<ecdsa::SignatureReply, String> {
    // hash the message to be signed
    let message_hash = ethereum::hash_eth_message(&message.into_bytes());

    // sign the message
    let public_key = derive_pk().await;
    let request = ecdsa::SignWithECDSA {
        message_hash: message_hash.clone(),
        derivation_path: vec![],
        key_id: ecdsa::EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };

    let (response,): (ecdsa::SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (request,),
        remittance::MAX_CYCLE,
    )
    .await
    .map_err(|e| format!("SIGN_WITH_ECDSA_FAILED {}", e.1))?;

    let full_signature = ethereum::get_signature(&response.signature, &message_hash, &public_key);
    Ok(ecdsa::SignatureReply {
        signature_hex: utils::vec_u8_to_string(&full_signature),
    })
}

#[query]
async fn verify(
    signature_hex: String,
    message: String,
    sec1_pk: String,
) -> Result<ecdsa::SignatureVerificationReply, String> {
    let signature_bytes = hex::decode(&signature_hex).expect("FAILED_TO_HEXDECODE_SIGNATURE");
    let pubkey_bytes = hex::decode(&sec1_pk).expect("FAILED_TO_HEXDECODE_PUBLIC_KEY");
    let message_bytes = ethereum::hash_eth_message(&message.into_bytes());

    use k256::ecdsa::signature::Verifier;
    let signature = k256::ecdsa::Signature::try_from(signature_bytes.as_slice())
        .expect("DESERIALIZE_SIGNATURE_FAILED");
    let is_signature_valid = k256::ecdsa::VerifyingKey::from_sec1_bytes(&pubkey_bytes)
        .expect("DESERIALIZE_SEC1_ENCODING_FAILED")
        .verify(&message_bytes, &signature)
        .is_ok();

    Ok(ecdsa::SignatureVerificationReply { is_signature_valid })
}
