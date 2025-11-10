// Transfer Storage - Manages transfer state and file assembly

use super::{Result, TransferError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Transfer state for resume capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferState {
    pub transfer_id: String,
    pub file_name: String,
    pub file_size: u64,
    pub chunk_size: usize,
    pub total_chunks: u32,
    pub received_chunks: HashSet<u32>,
    pub blake3_hash: String,
    pub started_at: String,
    pub last_activity: String,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    InProgress,
    Complete,
    Incomplete,
    Failed,
}

/// Storage manager for file transfers
pub struct TransferStorage {
    base_path: PathBuf,
}

impl TransferStorage {
    /// Create a new transfer storage instance
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Initialize storage (create directories)
    pub async fn initialize(&self) -> Result<()> {
        fs::create_dir_all(&self.base_path).await?;
        Ok(())
    }

    /// Get transfer directory path
    pub fn transfer_path(&self, transfer_id: &str) -> PathBuf {
        self.base_path.join(transfer_id)
    }

    /// Get chunks directory path
    pub fn chunks_path(&self, transfer_id: &str) -> PathBuf {
        self.transfer_path(transfer_id).join("chunks")
    }

    /// Get final file directory path
    pub fn final_path(&self, transfer_id: &str) -> PathBuf {
        self.transfer_path(transfer_id).join("final")
    }

    /// Get metadata file path
    pub fn metadata_path(&self, transfer_id: &str) -> PathBuf {
        self.transfer_path(transfer_id).join("metadata.json")
    }

    /// Get chunk file path
    pub fn chunk_file_path(&self, transfer_id: &str, chunk_index: u32) -> PathBuf {
        self.chunks_path(transfer_id)
            .join(format!("chunk-{:06}.bin", chunk_index))
    }

    /// Create transfer directories
    pub async fn create_transfer(&self, transfer_id: &str) -> Result<()> {
        fs::create_dir_all(self.chunks_path(transfer_id)).await?;
        fs::create_dir_all(self.final_path(transfer_id)).await?;
        Ok(())
    }

    /// Save transfer metadata
    pub async fn save_metadata(&self, state: &TransferState) -> Result<()> {
        let path = self.metadata_path(&state.transfer_id);
        let json = serde_json::to_string_pretty(state)?;
        fs::write(path, json).await?;
        Ok(())
    }

    /// Load transfer metadata
    pub async fn load_metadata(&self, transfer_id: &str) -> Result<TransferState> {
        let path = self.metadata_path(transfer_id);
        if !path.exists() {
            return Err(TransferError::TransferNotFound(transfer_id.to_string()));
        }

        let json = fs::read_to_string(path).await?;
        let state: TransferState = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// Check if transfer exists
    pub async fn transfer_exists(&self, transfer_id: &str) -> bool {
        self.metadata_path(transfer_id).exists()
    }

    /// Save a chunk to disk
    pub async fn save_chunk(&self, transfer_id: &str, chunk_index: u32, data: &[u8]) -> Result<()> {
        let path = self.chunk_file_path(transfer_id, chunk_index);
        fs::write(path, data).await?;
        Ok(())
    }

    /// Load a chunk from disk
    pub async fn load_chunk(&self, transfer_id: &str, chunk_index: u32) -> Result<Vec<u8>> {
        let path = self.chunk_file_path(transfer_id, chunk_index);
        let data = fs::read(path).await?;
        Ok(data)
    }

    /// Check if a chunk exists
    pub async fn chunk_exists(&self, transfer_id: &str, chunk_index: u32) -> bool {
        self.chunk_file_path(transfer_id, chunk_index).exists()
    }

    /// Assemble all chunks into final file
    pub async fn assemble_file(
        &self,
        transfer_id: &str,
        file_name: &str,
        total_chunks: u32,
    ) -> Result<PathBuf> {
        let final_path = self.final_path(transfer_id).join(file_name);
        let mut file = fs::File::create(&final_path).await?;

        // Write chunks in order
        for chunk_index in 0..total_chunks {
            let chunk_data = self.load_chunk(transfer_id, chunk_index).await?;
            file.write_all(&chunk_data).await?;
        }

        file.flush().await?;
        Ok(final_path)
    }

    /// Scan for received chunks
    pub async fn scan_received_chunks(&self, transfer_id: &str) -> Result<HashSet<u32>> {
        let chunks_dir = self.chunks_path(transfer_id);
        let mut received = HashSet::new();

        if !chunks_dir.exists() {
            return Ok(received);
        }

        let mut entries = fs::read_dir(chunks_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            // Parse "chunk-XXXXXX.bin"
            if let Some(index_str) = name_str.strip_prefix("chunk-").and_then(|s| s.strip_suffix(".bin")) {
                if let Ok(index) = index_str.parse::<u32>() {
                    received.insert(index);
                }
            }
        }

        Ok(received)
    }

    /// Calculate received bytes
    pub fn calculate_received_bytes(received_chunks: &HashSet<u32>, chunk_size: usize) -> u64 {
        received_chunks.len() as u64 * chunk_size as u64
    }

    /// Find missing chunks
    pub fn find_missing_chunks(received_chunks: &HashSet<u32>, total_chunks: u32) -> Vec<u32> {
        (0..total_chunks)
            .filter(|i| !received_chunks.contains(i))
            .collect()
    }

    /// Clean up transfer (remove all files)
    pub async fn cleanup_transfer(&self, transfer_id: &str) -> Result<()> {
        let transfer_dir = self.transfer_path(transfer_id);
        if transfer_dir.exists() {
            fs::remove_dir_all(transfer_dir).await?;
        }
        Ok(())
    }

    /// Get disk space information
    pub async fn get_disk_space(&self) -> Result<(u64, u64)> {
        // This is a simplified version - in production, use statvfs or similar
        // Returns (available_bytes, total_bytes)
        // For now, return placeholder values
        Ok((100 * 1024 * 1024 * 1024, 500 * 1024 * 1024 * 1024))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TransferStorage::new(temp_dir.path().to_path_buf());

        storage.initialize().await.unwrap();
        assert!(temp_dir.path().exists());
    }

    #[tokio::test]
    async fn test_save_and_load_chunk() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TransferStorage::new(temp_dir.path().to_path_buf());

        let transfer_id = "test-transfer";
        let chunk_data = b"Hello, chunk!";

        storage.create_transfer(transfer_id).await.unwrap();
        storage.save_chunk(transfer_id, 0, chunk_data).await.unwrap();

        let loaded = storage.load_chunk(transfer_id, 0).await.unwrap();
        assert_eq!(loaded, chunk_data);
    }

    #[tokio::test]
    async fn test_scan_received_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let storage = TransferStorage::new(temp_dir.path().to_path_buf());

        let transfer_id = "test-transfer";
        storage.create_transfer(transfer_id).await.unwrap();

        // Save chunks 0, 1, 3
        storage.save_chunk(transfer_id, 0, b"chunk0").await.unwrap();
        storage.save_chunk(transfer_id, 1, b"chunk1").await.unwrap();
        storage.save_chunk(transfer_id, 3, b"chunk3").await.unwrap();

        let received = storage.scan_received_chunks(transfer_id).await.unwrap();
        assert_eq!(received.len(), 3);
        assert!(received.contains(&0));
        assert!(received.contains(&1));
        assert!(received.contains(&3));
        assert!(!received.contains(&2));
    }

    #[tokio::test]
    async fn test_find_missing_chunks() {
        let mut received = HashSet::new();
        received.insert(0);
        received.insert(1);
        received.insert(3);

        let missing = TransferStorage::find_missing_chunks(&received, 5);
        assert_eq!(missing, vec![2, 4]);
    }
}
