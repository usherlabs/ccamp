//! Parsing and verifying Redstone datapackages
//! Ref : https://github.com/redstone-finance/redstone-rust-sdk
use serde_derive::{Serialize, Deserialize};
use serde_json::Value;
use std::{time::{Duration, SystemTime, UNIX_EPOCH}, u8};
use sha3::{Keccak256, Digest};
use candid::CandidType;

const REDSTONE_MARKER_BS: usize = 9;
const UNSIGNED_METADATA_BYTE_SIZE_BS: usize = 3;
const DATA_PACKAGES_COUNT_BS: usize = 2;
const DATA_POINTS_COUNT_BS: usize = 3;
const SIGNATURE_BS: usize = 65;
const MAX_SIGNERS_COUNT: usize = 256;
const DATA_POINT_VALUE_BYTE_SIZE_BS: usize = 4;
const DATA_FEED_ID_BS: usize = 32;
const TIMESTAMP_BS: usize = 6;
const MAX_TIMESTAMP_DELAY_MS: u128 = 3 * 60 * 1000; // 3 minutes in milliseconds
const REDSTONE_MARKER: [u8; 9] = [0, 0, 2, 237, 87, 1, 30, 0, 0]; // 0x000002ed57011e0000I
const REDSTONE_STREAM_IDS : [&str; 5] = [
	"0xBa24aFB019D768263edD606a9Fd10D3d0806E039/redstone-oracle-node/0x8BB8F32Df04c8b654987DAaeD53D6B6091e3B774/data-packages",
	"0x65684fcb6f470BF2eEa3949CDd771b45F601481b/redstone-oracle-node/0xdEB22f54738d54976C4c0fe5ce6d408E40d88499/data-packages",
	"0x6425466cBB7Cd64F3F2AD3b65aB0D0b5471A5483/redstone-oracle-node/0x51Ce04Be4b3E32572C4Ec9135221d0691Ba7d202/data-packages",
	"0x8a0108B1c5B646f71BB679Bc2E18f83df2D65cfd/redstone-oracle-node/0xDD682daEC5A90dD295d14DA4b0bec9281017b5bE/data-packages",
	"0x871221720E3965773bEBAf157E204Ab22c9BE309/redstone-oracle-node/0x9c5AE89C4Af6aA32cE58588DBaF90d18a855B6de/data-packages",
];

/// Ref : https://docs.rs/candid/latest/candid/types/value/enum.IDLValue.html#variant.Record
// type Metadata = IDLValue;

/// Ref : https://github.com/redstone-finance/redstone-oracles-monorepo/blob/main/packages/protocol/src/data-point/DataPoint.ts
/// ```typescript
///     export interface IStandardDataPoint {
///     dataFeedId: ConvertibleToBytes32;
///     value: string; // base64-encoded bytes
///     metadata?: Metadata;
/// }
/// ```
#[derive(CandidType, Deserialize, Serialize)]
struct DataPointPlainObj {
    data_feed_id: Vec<u8>,
    value: f32,
}

impl DataPointPlainObj {
    pub fn new(data_feed_id: &[u8], value: f32) -> Self {
        Self {
            data_feed_id : data_feed_id.to_vec(),
            value,
        }
    }
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
#[derive(CandidType, Deserialize, Serialize)]
pub struct DataPackage {
    timestamp_ms : SystemTime,
    signature: Vec<u8>,
    data_points: Vec<DataPointPlainObj>,
}

fn bytes_arr_to_number(number_bytes: &[u8]) -> u128 {
    let mut result_number = 0;
    let mut multiplier = 1;

    for i in (0..number_bytes.len()).rev() {
        // To prevent overflow error
        if i == 16 {
            break;
        }
        result_number += u128::from(number_bytes[i]) * multiplier;
        multiplier *= 256;
    }

    result_number
}

impl DataPackage {
    fn assert_valid_redstone_marker(redstone_payload: &[u8]) {
        let marker_start_index = redstone_payload.len() - REDSTONE_MARKER_BS;
        let redstone_marker = &redstone_payload[marker_start_index..];
        if REDSTONE_MARKER != redstone_marker {
            panic!("Invalid redstone marker");
        }
    }

    fn extract_usize_num_from_redstone_payload(
        redstone_payload: &[u8],
        start: usize,
        end: usize,
    ) -> usize {
        let number_bytes = &redstone_payload[start..end];
        usize::try_from(bytes_arr_to_number(number_bytes)).unwrap()
    }

    fn extract_unsigned_metadata_offset(redstone_payload: &[u8]) -> usize {
        let end_index = redstone_payload.len() - REDSTONE_MARKER_BS; // not inclusive
        let start_index = end_index - UNSIGNED_METADATA_BYTE_SIZE_BS;
        let unsigned_metadata_bs =
            Self::extract_usize_num_from_redstone_payload(redstone_payload, start_index, end_index);
    
        unsigned_metadata_bs + UNSIGNED_METADATA_BYTE_SIZE_BS + REDSTONE_MARKER_BS
    }
    
    fn extract_number_of_data_packages(
        redstone_payload: &[u8],
        unsigned_metadata_offset: usize,
    ) -> usize {
        let end_index = redstone_payload.len() - unsigned_metadata_offset;
        let start_index = end_index - DATA_PACKAGES_COUNT_BS;
        Self::extract_usize_num_from_redstone_payload(redstone_payload, start_index, end_index)
    }

    pub fn extract_and_verify(redstone_payload : &[u8], signer : &[u8; 20]) -> Self {
        // Extracting signature
        let mut end_index = redstone_payload.len();
        let mut start_index = end_index - SIGNATURE_BS;
        let signature = &redstone_payload[start_index..end_index]; 

        // Extracting number of data points
        start_index -= DATA_POINTS_COUNT_BS;
        end_index = start_index + DATA_POINTS_COUNT_BS;
        let data_points_count = Self::extract_usize_num_from_redstone_payload(redstone_payload, start_index, end_index);

        // Extracting data points value byte size
        start_index -= DATA_POINT_VALUE_BYTE_SIZE_BS;
        end_index = start_index + DATA_POINT_VALUE_BYTE_SIZE_BS;
        let data_points_value_bs = Self::extract_usize_num_from_redstone_payload(redstone_payload, start_index, end_index);

        // Extracting and validating timestamp
        start_index -= TIMESTAMP_BS;
        end_index = start_index + TIMESTAMP_BS;
        let timestamp_milliseconds = bytes_arr_to_number(&redstone_payload[start_index..end_index]);
        let timestamp_ms = UNIX_EPOCH.checked_add(Duration::from_millis(timestamp_milliseconds as u64)).unwrap();

        let mut data_points = Vec::new();
        // Going through data points
        for _data_point_index in 0..data_points_count {
            // Extracting value
            start_index -= data_points_value_bs;
            end_index = start_index + data_points_value_bs;
            let data_point_value = bytes_arr_to_number(&redstone_payload[start_index..end_index]);

            // Extracting data feed id
            start_index -= DATA_FEED_ID_BS;
            end_index = start_index + DATA_FEED_ID_BS;
            let data_feed_id = &redstone_payload[start_index..end_index];
            data_points.push(DataPointPlainObj::new(data_feed_id, data_point_value as f32));
        }

        // Calculating total data package byte size
        let data_package_byte_size_without_sig = (data_points_value_bs + DATA_FEED_ID_BS)
        * data_points_count
        + TIMESTAMP_BS
        + DATA_POINT_VALUE_BYTE_SIZE_BS
        + DATA_POINTS_COUNT_BS;

        // Message construction
        end_index = redstone_payload.len() - SIGNATURE_BS;
        start_index = end_index - data_package_byte_size_without_sig;
        let signable_message = &redstone_payload[start_index..end_index];

        let recovery_id = libsecp256k1::RecoveryId::parse(signature[64] as u8 - 27).expect("Failed to parse recovery ID");
        let sig = libsecp256k1::Signature::parse_standard_slice(&signature[..64]).expect("Failed to parse signature");
        let mut hasher = Keccak256::new();
        hasher.update(signable_message);
        let hash_result = hasher.finalize();
        let msg = libsecp256k1::Message::parse_slice(&hash_result).expect("Failed to parse message slice");

        let pub_key = libsecp256k1::recover(&msg, &sig, &recovery_id).expect("Failed to recover public key");

        let pub_bytes = pub_key.serialize();
        let mut hasher = Keccak256::new();
        hasher.update(&pub_bytes[1..]);
        let recovered_addr = hasher.finalize()[12..].to_owned();

        if recovered_addr != signer.to_owned() {
            panic!("the signature is invalid");
        }

        Self {
            timestamp_ms,
            signature : signature.to_vec(),
            data_points
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct StreamRawData {
    pub stream_id : String,
    pub data_type : String,
    pub stream_data : Value,
}

pub enum StreamData {
    RedstoneData(RedstoneData),
    UnknownType,
}

impl From<StreamRawData> for StreamData {
    fn from(raw_data : StreamRawData) -> Self {
        // This will be handling more different types of stream data
        match raw_data.data_type.as_str() {
            "redstone_data_packages" => {
                // Check if the streamr id is valid
                if !REDSTONE_STREAM_IDS.contains(&raw_data.stream_id.as_str()) {
                    panic!("The streamr id is not a valid redstone publisher id");
                }

                let address : &str = raw_data.stream_id.split("/").collect::<Vec<&str>>().get(2).unwrap();
                let mut signer_bytes = [0u8; 20];

                hex::decode_to_slice(address[2..].to_owned(), &mut signer_bytes).unwrap();
                StreamData::RedstoneData(
                RedstoneData {
                    address : signer_bytes,
                    data_packages : serde_json::from_value(raw_data.stream_data).unwrap(),
                })
            },
            _ => StreamData::UnknownType,
        }
    }
}

#[derive(Deserialize)]
pub struct RedstoneData {
    pub address : [u8; 20],
    pub data_packages : Vec<String>,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse_datapackage() {
        let sample_datapackage_hex = "0x55534454000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000005f51fc50190085ef80000000020000001107dc708d9bbeb2fa822e5c1d313fe5884a703bc56abbe4431d14009b7f532e56a46004751153973cdf850b56ee41f3a66e6900100c5f3c2d02ae904f12b7a291b";
        let signer = "0x8BB8F32Df04c8b654987DAaeD53D6B6091e3B774";
        let datapackage_bytes = hex::decode(sample_datapackage_hex[2..].to_owned()).unwrap();
        let mut signer_bytes = [0u8; 20];
        hex::decode_to_slice(signer[2..].to_owned(), &mut signer_bytes).unwrap();
        let datapackage = DataPackage::extract_and_verify(datapackage_bytes.as_slice(), &signer_bytes);
    }
}