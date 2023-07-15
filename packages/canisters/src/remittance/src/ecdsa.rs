#![allow(dead_code)]

use crate::{ecdsa, ethereum, remittance, utils};
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
    Principal,
};

#[derive(CandidType, Serialize, Debug)]
pub struct PublicKeyReply {
    pub sec1_pk: String,
    pub etherum_pk: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureReply {
    pub signature_hex: String,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignatureVerificationReply {
    pub is_signature_valid: bool,
}

type CanisterId = Principal;

#[derive(CandidType, Serialize, Debug)]
pub struct ECDSAPublicKey {
    pub canister_id: Option<CanisterId>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct ECDSAPublicKeyReply {
    pub public_key: Vec<u8>,
    pub chain_code: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug)]
pub struct SignWithECDSA {
    pub message_hash: Vec<u8>,
    pub derivation_path: Vec<Vec<u8>>,
    pub key_id: EcdsaKeyId,
}

#[derive(CandidType, Deserialize, Debug)]
pub struct SignWithECDSAReply {
    pub signature: Vec<u8>,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub struct EcdsaKeyId {
    pub curve: EcdsaCurve,
    pub name: String,
}

#[derive(CandidType, Serialize, Debug, Clone)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

pub enum EcdsaKeyIds {
    #[allow(unused)]
    TestKeyLocalDevelopment,
    #[allow(unused)]
    TestKey1,
    #[allow(unused)]
    ProductionKey1,
}

impl EcdsaKeyIds {
    pub fn to_key_id(&self) -> EcdsaKeyId {
        EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: match self {
                Self::TestKeyLocalDevelopment => "dfx_test_key",
                Self::TestKey1 => "test_key_1",
                Self::ProductionKey1 => "key_1",
            }
            .to_string(),
        }
    }
}

pub async fn verify(
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

pub async fn derive_pk() -> Vec<u8> {
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

pub async fn sign(message: String) -> Result<ecdsa::SignatureReply, String> {
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
// In the following, we register a custom getrandom implementation because
// otherwise getrandom (which is a dependency of k256) fails to compile.
// This is necessary because getrandom by default fails to compile for the
// wasm32-unknown-unknown target (which is required for deploying a canister).
// Our custom implementation always fails, which is sufficient here because
// we only use the k256 crate for verifying secp256k1 signatures, and such
// signature verification does not require any randomness.
// getrandom::register_custom_getrandom!(always_fail);
// pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
//     Err(getrandom::Error::UNSUPPORTED)
// }
