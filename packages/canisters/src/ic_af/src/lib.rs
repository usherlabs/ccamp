mod tls_parse;
mod redstone_data;

use candid::Principal;
use tls_parse::{ParsedRequest, ParsedResponse};
use redstone_data::{DataPackage, StreamData, StreamRawData};
use ic_cdk_macros::*;
use tlsn_core::proof::{SessionProof, TlsProof};
use elliptic_curve::pkcs8::DecodePublicKey;

use lib::{ ethereum, utils };
use lib::ecdsa::{EcdsaKeyIds, SignWithECDSA, SignWithECDSAReply, SignatureReply, ECDSAPublicKey, ECDSAPublicKeyReply};

#[init]
fn init() {
    unsafe {
        ic_wasi_polyfill::init(&[0u8; 32], &[]);
    }
}

/// Sha256 hash
fn sha256(input: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(input);
    hasher.finalize().into()
}

/// verifying the signatures for data-package
/// Sample file : ../../../../fixtures/data-package.json
#[update]
async fn verify_data_proof(stream_raw_data : String) -> (String, String) {
    let stream_raw_data : StreamRawData = serde_json::from_str(stream_raw_data.as_str()).unwrap();
    let stream_data = StreamData::from(stream_raw_data.clone());

    let json_result : String;
    // This will be handling more different types of stream data
    match stream_data {
        StreamData::RedstoneData(redstone_data) => {
            let mut result = Vec::new();
            for data_package in redstone_data.data_packages {
                let data_package_bytes = hex::decode(data_package.as_str()[2..].to_owned()).unwrap();
                let data_package = DataPackage::extract_and_verify(data_package_bytes.as_slice(), &redstone_data.address);
                result.push(data_package);
            }
            json_result = serde_json::to_string(&result).unwrap();
        },
        StreamData::UnknownType => {
            panic!("UnknownType");
        }
    }

    let bin_data = bincode::serialize(&stream_raw_data).unwrap();

    let request = SignWithECDSA {
        message_hash: sha256(bin_data.as_slice()).to_vec(),
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };

    let (response,): (SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (request,),
        25_000_000_000,
    )
    .await
    .map_err(|e| format!("sign_with_ecdsa failed {}", e.1)).unwrap();

    let reply = SignatureReply {
        signature_hex: hex::encode(&response.signature),
    };

    (json_result, reply.signature_hex)
}


/// verifying the tls proofs
/// Sample file : ../../../../fixtures/tiwtter_proof.json
#[update]
async fn verify_tls_proof(tls_proof : String) -> (ParsedRequest, ParsedResponse, String) {
    let tls_proof: TlsProof = serde_json::from_str(tls_proof.as_str()).unwrap();

    let TlsProof {
        session,
        substrings,
    } = tls_proof;

    // Verify the session proof against the Notary's public key
    //
    // This verifies the identity of the server using a default certificate verifier which trusts
    // the root certificates from the `webpki-roots` crate.
    let pub_key_raw = r#"
        -----BEGIN PUBLIC KEY-----
        MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEBv36FI4ZFszJa0DQFJ3wWCXvVLFr
        cRzMG5kaTeHGoSzDu6cFqx3uEWYpFGo6C0EOUgf+mEgbktLrXocv5yHzKg==
        -----END PUBLIC KEY-----
    "#;
    let pub_key = p256::PublicKey::from_public_key_pem(pub_key_raw)
        .unwrap_or_else(|_| panic!("INVALID PUBLIC KEY"));
    session
        .verify_with_default_cert_verifier(pub_key)
        .unwrap_or_else(|_| panic!("FAILED TO VERIFY SESSION"));

    let SessionProof {
        // The session header that was signed by the Notary is a succinct commitment to the TLS transcript.
        header,
        // This is the session_info, which contains the server_name, that is checked against the
        // certificate chain shared in the TLS handshake.
        // session_info,
        ..
    } = session;

    let (sent, recv) = substrings.verify(&header).unwrap();

    // // Replace the bytes which the Prover chose not to disclose with 'X'
    // sent.set_redacted(b'X');
    // recv.set_redacted(b'X');

    // Parsing http request and response
    let mut headers: [httparse::Header<'_>; 16] = [httparse::EMPTY_HEADER; 16];
    let mut http_req = httparse::Request::new(&mut headers);
    let parsed_http_req: ParsedRequest = match http_req.parse(sent.data()).unwrap() {
        httparse::Status::Complete(_) => http_req.into(),
        httparse::Status::Partial => {
            panic!("Failed to parse the TLS request");
        }
    };

    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut http_res = httparse::Response::new(&mut headers);
    let response_bytes = recv.data();
    let parsed_http_res: ParsedResponse = match http_res.parse(response_bytes).unwrap() {
        httparse::Status::Complete(body_start) => {
            let body = &response_bytes[body_start..];
            (http_res, body).into()
        }
        httparse::Status::Partial => {
            panic!("Failed to parse the TLS response");
        }
    };

    let mut bin_data = bincode::serialize(&sent.data()).unwrap();
    bin_data.extend(&bincode::serialize(&parsed_http_res).unwrap());

    let message_hash = ethereum::hash_eth_message(&bin_data.as_slice());
    let sign_request = SignWithECDSA {
        message_hash: message_hash.clone(),
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(), // TODO: Ensure env aware
    };

    let (sign_response,): (SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
        Principal::management_canister(),
        "sign_with_ecdsa",
        (sign_request,),
        25_000_000_000,
    )
    .await
    .map_err(|e| format!("sign_with_ecdsa failed {}", e.1)).unwrap();

    let derive_pk_request = ECDSAPublicKey {
        canister_id: None,
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(), // TODO: Ensure env aware
    };
    let (pk_res,): (ECDSAPublicKeyReply,) = ic_cdk::call(
        Principal::management_canister(),
        "ecdsa_public_key",
        (derive_pk_request,),
    )
    .await
    .map_err(|e| format!("ECDSA_PUBLIC_KEY_FAILED {}", e.1))
    .unwrap();

    let full_signature = ethereum::get_signature(&sign_response.signature, &message_hash, &pk_res.public_key);
    let reply = SignatureReply {
        signature_hex: utils::vec_u8_to_string(&full_signature)
    };

    (parsed_http_req, parsed_http_res, reply.signature_hex)
}
