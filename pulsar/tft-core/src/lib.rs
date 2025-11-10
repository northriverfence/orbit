//! TFT (Terminal File Transfer) Protocol Core
//!
//! This crate provides the core protocol implementation for TFT, including:
//! - NDJSON message definitions
//! - File chunking and integrity verification
//! - Encryption/decryption primitives
//! - Merkle tree construction for chunk verification

pub mod protocol;
pub mod chunking;
pub mod crypto;
pub mod merkle;

pub use protocol::{Message, MessageType};
pub use chunking::{FileChunker, ChunkInfo};
pub use crypto::{EncryptionKey, encrypt_chunk, decrypt_chunk};
pub use merkle::MerkleTree;

/// TFT protocol version
pub const PROTOCOL_VERSION: &str = "1.0";

/// Default chunk size (1MB)
pub const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_version() {
        assert_eq!(PROTOCOL_VERSION, "1.0");
    }
}
