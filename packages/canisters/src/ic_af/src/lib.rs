mod tls_parse;
mod redstone_data;

use tls_parse::{ParsedRequest, ParsedResponse};
use redstone_data::{DataPackage, RedstoneData};
use std::str::FromStr;

use ic_cdk::api::management_canister::main::CanisterId;
use ic_cdk_macros::*;
use tlsn_substrings_verifier::proof::{SessionProof, TlsProof};

use lib::ecdsa::{EcdsaKeyIds, SignWithECDSA, SignWithECDSAReply, SignatureReply};

/// Get the management canister
fn mgmt_canister_id() -> CanisterId {
    CanisterId::from_str(&"aaaaa-aa").unwrap()
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
fn verify_data_proof(redstone_data : String) -> Vec<DataPackage> {
    let redstone_data : RedstoneData = serde_json::from_str(redstone_data.as_str()).unwrap();
    let mut signer_bytes = [0u8; 20];
    let mut result = Vec::new();

    hex::decode_to_slice(redstone_data.address[2..].to_owned(), &mut signer_bytes).unwrap();
    for data_package in redstone_data.dataPackages {
        let data_package_bytes = hex::decode(data_package.as_str()[2..].to_owned()).unwrap();
        let data_package = DataPackage::extract_and_verify(data_package_bytes.as_slice(), &signer_bytes);
        result.push(data_package);
    }

    result
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

    let SessionProof {
        // The session header that was signed by the Notary is a succinct commitment to the TLS transcript.
        header,
        // This is the session_info, which contains the server_name, that is checked against the
        // certificate chain shared in the TLS handshake.
        // session_info,
        ..
    } = session;

    let (mut sent, mut recv) = substrings.verify(&header).unwrap();

    // Replace the bytes which the Prover chose not to disclose with 'X'
    sent.set_redacted(b'X');
    recv.set_redacted(b'X');

    // Parsing http request and response
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut http_req = httparse::Request::new(&mut headers);
    http_req.parse(sent.data()).unwrap();
    let parsed_http_req = ParsedRequest::from(http_req);

    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut http_res = httparse::Response::new(&mut headers);
    http_res.parse(recv.data()).unwrap();
    let parsed_http_res = ParsedResponse::from(http_res);

    let mut bin_data = bincode::serialize(&parsed_http_req).unwrap();
    bin_data.extend(&bincode::serialize(&parsed_http_res).unwrap());

    let request = SignWithECDSA {
        message_hash: sha256(bin_data.as_slice()).to_vec(),
        derivation_path: vec![],
        key_id: EcdsaKeyIds::TestKeyLocalDevelopment.to_key_id(),
    };

    let (response,): (SignWithECDSAReply,) = ic_cdk::api::call::call_with_payment(
        mgmt_canister_id(),
        "sign_with_ecdsa",
        (request,),
        25_000_000_000,
    )
    .await
    .map_err(|e| format!("sign_with_ecdsa failed {}", e.1)).unwrap();

    let reply = SignatureReply {
        signature_hex: hex::encode(&response.signature),
    };

    (parsed_http_req, parsed_http_res, reply.signature_hex)
}

ic_cdk::export_candid!();