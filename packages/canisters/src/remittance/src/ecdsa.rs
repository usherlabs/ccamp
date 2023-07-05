use easy_hasher::easy_hasher;
use ic_cdk::export::{
    candid::CandidType,
    serde::{Deserialize, Serialize},
    Principal,
};
use ic_cdk_macros::*;
use k256::pkcs8::der::Encode;
use std::convert::TryFrom;
use std::str::FromStr;

// this is the bytes equivalent of "\x19Ethereum Signed Message:\n32";
// which is used to prefix signed messages in ethereum
pub const ETH_SIGN_PREFIX: [u8; 28] = [
    0x19, 0x45, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x20, 0x53, 0x69, 0x67, 0x6e, 0x65, 0x64,
    0x20, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x3a, 0x0a, 0x33, 0x32,
];

#[derive(CandidType, Serialize, Debug)]
pub struct PublicKeyReply {
    pub public_key_hex: String,
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

pub fn sha256(input: &String) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input.as_bytes());
    hasher.finalize().into()
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

pub fn get_address_from_public_key(public_key: Vec<u8>) -> Result<String, String> {
    if public_key.len() != 33 {
        return Err("Invalid length of public key".to_string());
    }

    let pub_key_arr: [u8; 33] = public_key[..].try_into().unwrap();
    let pub_key = libsecp256k1::PublicKey::parse_compressed(&pub_key_arr)
        .map_err(|e| format!("{}", e))?
        .serialize();

    let keccak256 = easy_hasher::raw_keccak256(pub_key[1..].to_vec());
    let keccak256_hex = keccak256.to_hex_string();
    let address: String = "0x".to_owned() + &keccak256_hex[24..];

    Ok(address)
}

pub fn string_to_vec_u8(str: &str) -> Vec<u8> {
    let starts_from: usize;
    if str.starts_with("0x") {
        starts_from = 2;
    } else {
        starts_from = 0;
    }

    (starts_from..str.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&str[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>()
}

pub fn vec_u8_to_string(vec: &Vec<u8>) -> String {
    vec.iter()
        .map(|r| format!("{:02x}", r))
        .collect::<Vec<String>>()
        .join("")
        .to_string()
}

pub fn remove_leading(vec: &Vec<u8>, element: u8) -> Vec<u8> {
    let start = vec.iter().position(|&x| x != element).unwrap();
    let result = &vec[start..];
    result.to_vec()
}

pub fn get_signature(signature: &Vec<u8>, message: &Vec<u8>, public_key: &Vec<u8>) -> Vec<u8> {
    let r = remove_leading(&signature[..32].to_vec(), 0);
    let s = remove_leading(&signature[32..].to_vec(), 0);
    let recovery_id = get_recovery_id(message, signature, public_key).unwrap();
    
    let v = if recovery_id == 0 {
        hex::decode(format!("{:X}", 27)).unwrap()
    } else {
        hex::decode(format!("{:X}", 28)).unwrap()
    };

    
    let eth_sig = [&r[..], &s[..], &v[..]].concat();

    eth_sig
}

pub fn hash_eth_message(message: Vec<u8>) -> Vec<u8> {
    let message_hash = easy_hasher::raw_keccak256(message).to_vec();
    let eth_prefixed_hash =
        easy_hasher::raw_keccak256(encode_packed(&[&ETH_SIGN_PREFIX, &message_hash])).to_vec();

    return eth_prefixed_hash;
}

fn encode_packed(values: &[&[u8]]) -> Vec<u8> {
    let total_length: usize = values.iter().map(|v| v.len()).sum();
    let mut result = Vec::with_capacity(total_length);

    for value in values {
        result.extend_from_slice(value);
    }

    result
}

fn get_recovery_id(
    message: &Vec<u8>,
    signature: &Vec<u8>,
    public_key: &Vec<u8>,
) -> Result<u8, String> {
    if signature.len() != 64 {
        return Err("Invalid signature".to_string());
    }
    if message.len() != 32 {
        return Err("Invalid message".to_string());
    }
    if public_key.len() != 33 {
        return Err("Invalid public key".to_string());
    }

    for i in 0..3 {
        let recovery_id = libsecp256k1::RecoveryId::parse_rpc(27 + i).unwrap();

        let signature_bytes: [u8; 64] = signature[..].try_into().unwrap();
        let signature_bytes_64 = libsecp256k1::Signature::parse_standard(&signature_bytes).unwrap();

        let message_bytes: [u8; 32] = message[..].try_into().unwrap();
        let message_bytes_32 = libsecp256k1::Message::parse(&message_bytes);

        let key =
            libsecp256k1::recover(&message_bytes_32, &signature_bytes_64, &recovery_id).unwrap();
        if key.serialize_compressed() == public_key[..] {
            return Ok(i as u8);
        }
    }
    return Err("Not found".to_string());
}

// In the following, we register a custom getrandom implementation because
// otherwise getrandom (which is a dependency of k256) fails to compile.
// This is necessary because getrandom by default fails to compile for the
// wasm32-unknown-unknown target (which is required for deploying a canister).
// Our custom implementation always fails, which is sufficient here because
// we only use the k256 crate for verifying secp256k1 signatures, and such
// signature verification does not require any randomness.
getrandom::register_custom_getrandom!(always_fail);
pub fn always_fail(_buf: &mut [u8]) -> Result<(), getrandom::Error> {
    Err(getrandom::Error::UNSUPPORTED)
}
