use crate::{
    crypto::{
        ecdsa::{self, derive_pk},
        ethereum,
    },
    remittance::config::Config,
};
use candid::Principal;
use easy_hasher::easy_hasher;

use super::{remove_leading, string_to_vec_u8, vec_u8_to_string};

/// Preprocess and hash an ethereum message
pub fn hash_eth_message<T: AsRef<[u8]>>(message: T) -> Vec<u8> {
    const PREFIX: &str = "\x19Ethereum Signed Message:\n";

    let message = message.as_ref();
    let len = message.len();
    let len_string = len.to_string();

    let mut eth_message = Vec::with_capacity(PREFIX.len() + len_string.len() + len);
    eth_message.extend_from_slice(PREFIX.as_bytes());
    eth_message.extend_from_slice(len_string.as_bytes());
    eth_message.extend_from_slice(message);

    easy_hasher::raw_keccak256(eth_message)
        .to_vec()
        .try_into()
        .unwrap()
}

/// Get the public address responsible for a signature
pub fn recover_address_from_eth_signature(
    metamask_signature: String,
    message: String,
) -> Result<String, String> {
    let metamask_signature = string_to_vec_u8(&metamask_signature);
    if metamask_signature.len() != 65 {
        return Err("INVALID_ETH_SIGNATURE".to_string());
    }

    let signature_bytes: [u8; 64] = metamask_signature[0..64].try_into().unwrap();
    let signature_bytes_64 = libsecp256k1::Signature::parse_standard(&signature_bytes).unwrap();

    let recovery_id = metamask_signature[64];
    let recovery_id_byte = libsecp256k1::RecoveryId::parse_rpc(recovery_id).unwrap();

    let message_bytes: [u8; 32] = hash_eth_message(message).try_into().unwrap();
    let message_bytes_32 = libsecp256k1::Message::parse(&message_bytes);

    let public_key =
        libsecp256k1::recover(&message_bytes_32, &signature_bytes_64, &recovery_id_byte).unwrap();

    let address = get_address_from_public_key(public_key.serialize_compressed().to_vec()).unwrap();

    Ok(address)
}

/// append an extra discriminator byte to the ecdsa signature
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

/// convert a compressed SEC1 address(it is 33bytes instead of 65bytes)
/// we get back the ethereum version of the address (20bytes)
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

/// Sign a message which is provided in bytes
pub async fn sign_message(
    message: &Vec<u8>,
    config: &Config,
) -> Result<ecdsa::SignatureReply, String> {
    // hash the message to be signed
    let message_hash = ethereum::hash_eth_message(&message);

    // sign the message
    let public_key = derive_pk(config).await;
    let request = ecdsa::SignWithECDSA {
        message_hash: message_hash.clone(),
        derivation_path: vec![],
        key_id: config.key.to_key_id(),
    };

    let (response,): (ecdsa::SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (request,),
        config.sign_cycles,
    )
    .await
    .map_err(|e| format!("SIGN_WITH_ECDSA_FAILED {}", e.1))?;

    let full_signature = ethereum::get_signature(&response.signature, &message_hash, &public_key);
    Ok(ecdsa::SignatureReply {
        signature_hex: vec_u8_to_string(&full_signature),
    })
}
