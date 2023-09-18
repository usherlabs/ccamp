#![allow(dead_code)]

use crate::ecdsa;
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

pub async fn derive_pk() -> Vec<u8> {
    let request = ecdsa::ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        // value to be changed between testing and production
        key_id: ecdsa::EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(), //for testing/development
        // key_id: ecdsa::EcdsaKeyIds::ProductionKey1.to_key_id(), //for production

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

