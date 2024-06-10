mod tls_parse;

use tls_parse::{ParsedRequest, ParsedResponse};
use std::time::SystemTime;
use std::str::FromStr;

use candid::{parser::value::IDLValue, CandidType};
use ic_cdk::api::management_canister::main::CanisterId;
use ic_cdk_macros::*;
use serde::Deserialize;
use tlsn_substrings_verifier::proof::{SessionProof, TlsProof};

/// Ref : https://docs.rs/candid/latest/candid/types/value/enum.IDLValue.html#variant.Record
type Metadata = IDLValue;

/// Ref : https://github.com/redstone-finance/redstone-oracles-monorepo/blob/main/packages/protocol/src/data-point/DataPoint.ts
/// ```typescript
///     export interface IStandardDataPoint {
///     dataFeedId: ConvertibleToBytes32;
///     value: string; // base64-encoded bytes
///     metadata?: Metadata;
/// }
/// ```
#[derive(CandidType, Deserialize)]
struct DataPointPlainObj {
    dataFeedId: String,
    value: f32,
    metadata: Option<Metadata>,

}

/// Ref : https://github.com/redstone-finance/redstone-oracles-monorepo/blob/main/packages/cache-service/src/data-packages/data-packages.model.ts
/// ```typescript
/// ...
/// export type DataPackageDocumentMostRecentAggregated = {
///  _id: { signerAddress: string; dataFeedId: string };
///  timestampMilliseconds: Date;
///  signature: string;
///  dataPoints: DataPointPlainObj[];
///  dataServiceId: string;
///  dataFeedId: string;
///  dataPackageId: string;
///  isSignatureValid: boolean;
/// };
/// ...
/// ``````
#[derive(CandidType, Deserialize)]
struct DataPackage {
    timestampMilliseconds : SystemTime,
    signature: String,
    dataPoints: Vec<DataPointPlainObj>,
}

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
fn verify_data_proof(data_package : String) -> (String, String) {
    let data_package : DataPackage = serde_json::from_str(data_package.as_str()).unwrap();
    // todo!();
    (String::new(), String::new())
}


/// verifying the tls proofs
/// Sample file : ../../../../fixtures/tiwtter_proof.json
#[update]
async fn verify_tls_proof(tls_proof : String) -> (ParsedRequest, ParsedResponse) {
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

    (parsed_http_req, parsed_http_res)
}
