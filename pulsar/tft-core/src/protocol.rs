//! TFT Protocol Message Definitions

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// TFT protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    /// Initiate a file transfer
    TransferInit(TransferInit),
    /// Response to transfer init
    TransferResponse(TransferResponse),
    /// Send a file chunk
    Chunk(ChunkMessage),
    /// Acknowledge chunk receipt
    ChunkAck(ChunkAck),
    /// Transfer complete
    TransferComplete(TransferComplete),
    /// Error occurred
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInit {
    pub transfer_id: Uuid,
    pub filename: String,
    pub size: u64,
    pub chunk_size: usize,
    pub total_chunks: usize,
    pub merkle_root: String,
    pub encrypted: bool,
    pub compression: CompressionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    pub transfer_id: Uuid,
    pub accepted: bool,
    pub resume_from_chunk: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkMessage {
    pub transfer_id: Uuid,
    pub chunk_index: usize,
    pub data: Vec<u8>,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkAck {
    pub transfer_id: Uuid,
    pub chunk_index: usize,
    pub success: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferComplete {
    pub transfer_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    pub transfer_id: Option<Uuid>,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompressionType {
    None,
    Zstd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageType {
    TransferInit,
    TransferResponse,
    Chunk,
    ChunkAck,
    TransferComplete,
    Error,
}
