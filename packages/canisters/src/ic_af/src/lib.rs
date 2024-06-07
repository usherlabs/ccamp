use std::time::SystemTime;

use candid::{parser::value::IDLValue, CandidType};
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

/// verifying the signatures for data-package
/// Sample file : ../../../../fixtures/data-package.json
#[query]
fn verify_data_proof(data_package : String) -> Result<(), String> {
    let data_package : DataPackage = serde_json::from_str(data_package.as_str()).unwrap();
    // todo!();
    Ok(())
}

/// verifying the tls proofs
/// Sample file : ../../../../fixtures/tiwtter_proof.json
#[query]
fn verify_tls_proof(tls_proof : String) -> (String, String) {
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

    let sent_string = String::from_utf8(sent.data().to_vec()).unwrap();
    let recv_string = String::from_utf8(recv.data().to_vec()).unwrap();

    (sent_string, recv_string)
}
