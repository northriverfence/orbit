// File Transfer Protocol Messages

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransferMessage {
    TransferStart(TransferStartMessage),
    TransferAck(TransferAckMessage),
    ChunkData(ChunkDataMessage),
    ChunkAck(ChunkAckMessage),
    TransferComplete(TransferCompleteMessage),
    TransferSuccess(TransferSuccessMessage),
    TransferAbort(TransferAbortMessage),
    ResumeRequest(ResumeRequestMessage),
    ResumeInfo(ResumeInfoMessage),
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferStartMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub file_name: String,
    pub file_size: u64,
    pub chunk_size: usize,
    pub total_chunks: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    pub blake3_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<FileMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAckMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub accepted: bool,
    pub resume_supported: bool,
    pub max_chunk_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkDataMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub chunk_index: u32,
    pub chunk_size: usize,
    pub chunk_hash: String,
    // Note: actual binary data is sent separately after JSON header
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAckMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub chunk_index: u32,
    pub received: bool,
    pub hash_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferCompleteMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub total_chunks: u32,
    pub total_bytes: u64,
    pub final_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferSuccessMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub verified: bool,
    pub saved_path: String,
    pub received_chunks: u32,
    pub received_bytes: u64,
    pub computed_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferAbortMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeRequestMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub file_name: String,
    pub file_size: u64,
    pub original_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeInfoMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub resumable: bool,
    pub received_chunks: Vec<u32>,
    pub missing_chunks: Vec<u32>,
    pub next_chunk_index: u32,
    pub received_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub transfer_id: String,
    pub timestamp: u64,
    pub error_type: String,
    pub error_message: String,
}

impl TransferMessage {
    /// Get the transfer ID from any message type
    pub fn transfer_id(&self) -> &str {
        match self {
            Self::TransferStart(m) => &m.transfer_id,
            Self::TransferAck(m) => &m.transfer_id,
            Self::ChunkData(m) => &m.transfer_id,
            Self::ChunkAck(m) => &m.transfer_id,
            Self::TransferComplete(m) => &m.transfer_id,
            Self::TransferSuccess(m) => &m.transfer_id,
            Self::TransferAbort(m) => &m.transfer_id,
            Self::ResumeRequest(m) => &m.transfer_id,
            Self::ResumeInfo(m) => &m.transfer_id,
            Self::Error(m) => &m.transfer_id,
        }
    }

    /// Get the timestamp from any message type
    pub fn timestamp(&self) -> u64 {
        match self {
            Self::TransferStart(m) => m.timestamp,
            Self::TransferAck(m) => m.timestamp,
            Self::ChunkData(m) => m.timestamp,
            Self::ChunkAck(m) => m.timestamp,
            Self::TransferComplete(m) => m.timestamp,
            Self::TransferSuccess(m) => m.timestamp,
            Self::TransferAbort(m) => m.timestamp,
            Self::ResumeRequest(m) => m.timestamp,
            Self::ResumeInfo(m) => m.timestamp,
            Self::Error(m) => m.timestamp,
        }
    }

    /// Parse a JSON message
    pub fn from_json(data: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(data)
    }

    /// Serialize to JSON
    pub fn to_json(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Helper to generate current timestamp in milliseconds
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
