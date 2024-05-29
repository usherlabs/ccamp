use candid::{parser::value::IDLValue, CandidType};
use ic_cdk_macros::*;
use serde::Deserialize;
use tlsn_core::{proof::{SubstringsProof, TlsProof}, SessionHeader};
use std::time::SystemTime;

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
    todo!();
    Ok(())
}

/// verifying the tls proofs
/// Sample file : ../../../../fixtures/tiwtter_proof.json
#[query]
fn verify_tls_proof(tls_proof : String) -> Result<(), String> {
    let tls_proof : TlsProof = serde_json::from_str(tls_proof.as_str()).unwrap();
    todo!();
    Ok(())
}