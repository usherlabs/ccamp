use crate::{utils, ethereum, ecdsa::{self, derive_pk}};
use candid::Principal;
use easy_hasher::easy_hasher;

// this is the bytes equivalent of "\x19Ethereum Signed Message:\n32";
// which is used to prefix signed messages in ethereum
pub const SIGNATURE_HASH_PREFIX: [u8; 28] = [
    0x19, 0x45, 0x74, 0x68, 0x65, 0x72, 0x65, 0x75, 0x6d, 0x20, 0x53, 0x69, 0x67, 0x6e, 0x65, 0x64,
    0x20, 0x4d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x3a, 0x0a, 0x33, 0x32,
];

// use this function to double has a message
// so it can be verified on the ethereum network
pub fn hash_eth_message(message: &Vec<u8>) -> Vec<u8> {
    let message_hash = easy_hasher::raw_keccak256(message.clone()).to_vec();
    let eth_prefixed_hash =
        easy_hasher::raw_keccak256(concatenate_vectors(&[&SIGNATURE_HASH_PREFIX, &message_hash]))
            .to_vec();

    return eth_prefixed_hash;
}

// the equivalent of the abi.encodepacked function
pub fn concatenate_vectors(values: &[&[u8]]) -> Vec<u8> {
    let total_length: usize = values.iter().map(|v| v.len()).sum();
    let mut result = Vec::with_capacity(total_length);

    for value in values {
        result.extend_from_slice(value);
    }

    result
}

// append an extra discriminator byte to the ecdsa signature
pub fn get_signature(signature: &Vec<u8>, message: &Vec<u8>, public_key: &Vec<u8>) -> Vec<u8> {
    let r = utils::remove_leading(&signature[..32].to_vec(), 0);
    let s = utils::remove_leading(&signature[32..].to_vec(), 0);
    let recovery_id = get_recovery_id(message, signature, public_key).unwrap();

    let v = if recovery_id == 0 {
        hex::decode(format!("{:X}", 27)).unwrap()
    } else {
        hex::decode(format!("{:X}", 28)).unwrap()
    };

    let eth_sig = [&r[..], &s[..], &v[..]].concat();

    eth_sig
}

// use this function to derive a discriminator "v"
// the ecdsa signatures produces by the icp only consists of 64bytes
// so we use this function to add the additional byte needed by the evm
pub fn get_recovery_id(
    message: &Vec<u8>,
    signature: &Vec<u8>,
    public_key: &Vec<u8>,
) -> Result<u8, String> {
    if signature.len() != 64 {
        return Err("INVALID_SIGNATURE".to_string());
    }
    if message.len() != 32 {
        return Err("INVALID_MESSAGE".to_string());
    }
    if public_key.len() != 33 {
        return Err("INVALID_PUBLIC_KEY".to_string());
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
    return Err("DISCRIMINATOR_NOT_FOUND".to_string());
}

// convert a compressed SEC1 address(it is 33bytes instead of 65bytes)
// we get back the ethereum version of the address (20bytes)
pub fn get_address_from_public_key(public_key: Vec<u8>) -> Result<String, String> {
    if public_key.len() != 33 {
        return Err("INVALID_PK_LENGTH".to_string());
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


pub async fn sign_message(message: &Vec<u8>) -> Result<ecdsa::SignatureReply, String> {
    // hash the message to be signed
    let message_hash = ethereum::hash_eth_message(&message);

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
        25_000_000_000,
    )
    .await
    .map_err(|e| format!("SIGN_WITH_ECDSA_FAILED {}", e.1))?;

    let full_signature = ethereum::get_signature(&response.signature, &message_hash, &public_key);
    Ok(ecdsa::SignatureReply {
        signature_hex: utils::vec_u8_to_string(&full_signature),
    })
}