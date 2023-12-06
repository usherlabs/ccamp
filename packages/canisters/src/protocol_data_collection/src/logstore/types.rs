use serde::{Deserialize, Serialize};
use super::traits::PayloadContent;

#[derive(Debug)]
pub struct Options {
    pub message_id: MessageId,
    pub serialized_content: String,
    pub prev_msg_ref: Option<PrevMsgRef>,
    pub new_group_key: Option<EncryptedGroupKey>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedGroupKey {
    // Assuming EncryptedGroupKey has a serialize method
    // Add fields as needed
}
impl EncryptedGroupKey {
    fn serialize(&self) -> String {
        String::new()
    }
}

// #[derive(Debug, Serialize, Deserialize, Clone)]
// #[serde(rename_all = "camelCase")]

// pub struct BrokerPayload {
//     pub log_store_chain_id: String,
//     pub log_store_channel_id: String,
//     pub log_store_stream_id: String,
//     pub address: String,
//     pub block_hash: String,
//     pub data: String,
//     pub log_index: u32,
//     pub topics: Vec<String>,
//     pub transaction_hash: String,
// }

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MessageId {
    pub stream_id: String,
    pub stream_partition: u32,
    pub timestamp: u64,
    pub sequence_number: u32,
    pub publisher_id: String,
    pub msg_chain_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PrevMsgRef {
    pub timestamp: u64,
    pub sequence_number: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    // pub stream_id: String,
    // pub stream_partition: u32,
    // pub timestamp: u64,
    // pub sequence_number: u32,
    // pub signature: String,
    // pub publisher_id: String,
    // pub msg_chain_id: String,
    pub stream_message: StreamMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StreamMessage {
    pub message_id: MessageId,
    pub prev_msg_ref: Option<PrevMsgRef>,
    // pub message_type: u32,
    // pub content_type: u32,
    // pub encryption_type: u32,
    // pub group_key_id: Option<String>,
    pub new_group_key: Option<EncryptedGroupKey>,
    pub signature: String,
    // pub parsed_content: BrokerPayload,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrokerPayloadContent {
    pub log_store_chain_id: String,
    pub log_store_channel_id: String,
    pub log_store_stream_id: String,
    pub address: String,
    pub block_hash: String,
    pub data: String,
    pub log_index: u32,
    pub topics: Vec<String>,
    pub transaction_hash: String,
}

impl PayloadContent for BrokerPayloadContent {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PublisherPayloadContent {
    #[serde(rename(deserialize = "__logStoreChainId", serialize = "__logStoreChainId"))]
    pub __log_store_chain_id: String,
    #[serde(rename(deserialize = "__logStoreChannelId", serialize = "__logStoreChannelId"))]
    pub __log_store_channel_id: String,
    pub address: String,
    #[serde(rename(deserialize = "blockHash", serialize = "blockHash"))]
    pub block_hash: String,
    pub data: String,
    #[serde(rename(deserialize = "logIndex", serialize = "logIndex"))]
    pub log_index: u32,
    pub topics: Vec<String>,
    #[serde(rename(deserialize = "transactionHash", serialize = "transactionHash"))]
    pub transaction_hash: String,
}
impl PayloadContent for PublisherPayloadContent {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BrokerVerificationPayload {
    pub content: BrokerPayloadContent,
    pub metadata: Metadata,
}
impl BrokerVerificationPayload {
    pub fn get_publisher_address(&self) -> String {
        self.metadata.stream_message.message_id.publisher_id.clone()
    }

    pub fn get_signature_payload(&self) -> String {
        let opts = Options {
            message_id: self.metadata.stream_message.message_id.clone(),
            serialized_content: self.content.to_json_string(), // Assuming 'data' is the serialized content
            prev_msg_ref: self.metadata.stream_message.prev_msg_ref.clone(),
            new_group_key: self.metadata.stream_message.new_group_key.clone(),
        };

        derive_signature_payload(&opts)
    }
}

// TODO implement the source payload mechanism
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SourceVerificationPayload {
    pub content: PublisherPayloadContent,
    pub stream_message: StreamMessage,
}

impl SourceVerificationPayload {
    pub fn get_publisher_address(&self) -> String {
        self.stream_message.message_id.publisher_id.clone()
    }

    pub fn get_signature_payload(&self) -> String {
        let opts = Options {
            message_id: self.stream_message.message_id.clone(),
            serialized_content: self.content.to_json_string(), // Assuming 'data' is the serialized content
            prev_msg_ref: self.stream_message.prev_msg_ref.clone(),
            new_group_key: self.stream_message.new_group_key.clone(),
        };

        derive_signature_payload(&opts)
    }
}

fn derive_signature_payload(opts: &Options) -> String {
    let prev = if let Some(ref prev_msg_ref) = opts.prev_msg_ref {
        format!("{}{}", prev_msg_ref.timestamp, prev_msg_ref.sequence_number)
    } else {
        String::new()
    };

    let new_group_key = if let Some(ref new_group_key) = opts.new_group_key {
        new_group_key.serialize()
    } else {
        String::new()
    };

    format!(
        "{}{}{}{}{}{}{}{}{}",
        opts.message_id.stream_id,
        opts.message_id.stream_partition,
        opts.message_id.timestamp,
        opts.message_id.sequence_number,
        opts.message_id.publisher_id,
        opts.message_id.msg_chain_id,
        prev,
        opts.serialized_content,
        new_group_key
    )
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JSONPayload {
    pub source: SourceVerificationPayload,
    pub validation: Vec<BrokerVerificationPayload>,
}
