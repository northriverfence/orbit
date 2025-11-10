// File Transfer Module
//
// Implements chunked file transfers over WebTransport with:
// - 1 MB chunk size
// - Parallel stream support
// - Resume capability
// - BLAKE3 integrity validation

pub mod handler;
pub mod messages;
pub mod storage;
pub mod validation;

pub use handler::FileTransferHandler;
pub use messages::*;
pub use storage::TransferStorage;
pub use validation::HashValidator;

use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransferError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Chunk hash mismatch: chunk {chunk_index}, expected {expected}, got {actual}")]
    ChunkHashMismatch {
        chunk_index: u32,
        expected: String,
        actual: String,
    },

    #[error("File hash mismatch: expected {expected}, got {actual}")]
    FileHashMismatch { expected: String, actual: String },

    #[error("Disk full")]
    DiskFull,

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Transfer timeout")]
    TransferTimeout,

    #[error("Chunk out of order: expected {expected}, got {received}")]
    ChunkOutOfOrder { expected: u32, received: u32 },

    #[error("Invalid chunk size: chunk {chunk_index}, expected {expected}, got {actual}")]
    InvalidChunkSize {
        chunk_index: u32,
        expected: usize,
        actual: usize,
    },

    #[error("Transfer not found: {0}")]
    TransferNotFound(String),

    #[error("Resume not supported")]
    ResumeNotSupported,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, TransferError>;

/// Configuration for file transfers
#[derive(Debug, Clone)]
pub struct TransferConfig {
    /// Default chunk size (1 MB)
    pub chunk_size: usize,
    /// Maximum concurrent chunks
    pub max_parallel_chunks: usize,
    /// Storage path for transfers
    pub storage_path: PathBuf,
    /// Maximum transfer size (100 GB)
    pub max_file_size: u64,
    /// Transfer timeout (30 minutes)
    pub transfer_timeout_secs: u64,
}

impl Default for TransferConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1 MB
            max_parallel_chunks: 4,
            storage_path: PathBuf::from("/tmp/pulsar/transfers"),
            max_file_size: 100 * 1024 * 1024 * 1024, // 100 GB
            transfer_timeout_secs: 30 * 60,           // 30 minutes
        }
    }
}
