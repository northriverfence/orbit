// File Transfer Handler
//
// Handles incoming file transfer requests over WebTransport

use super::messages::*;
use super::storage::{TransferState, TransferStatus, TransferStorage};
use super::validation::{hash_data, verify_hash, HashValidator};
use super::{Result, TransferConfig, TransferError};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Active transfer session
#[derive(Debug, Clone)]
pub struct TransferSession {
    pub state: TransferState,
    pub validator: Arc<RwLock<HashValidator>>,
    pub started_at: SystemTime,
    pub last_activity: SystemTime,
}

/// File transfer handler
pub struct FileTransferHandler {
    config: TransferConfig,
    storage: Arc<TransferStorage>,
    active_transfers: Arc<RwLock<HashMap<String, Arc<RwLock<TransferSession>>>>>,
}

impl FileTransferHandler {
    /// Create a new file transfer handler
    pub fn new(config: TransferConfig) -> Self {
        let storage = Arc::new(TransferStorage::new(config.storage_path.clone()));

        Self {
            config,
            storage,
            active_transfers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the handler
    pub async fn initialize(&self) -> Result<()> {
        self.storage.initialize().await?;
        info!("File transfer handler initialized at {:?}", self.config.storage_path);
        Ok(())
    }

    /// Handle transfer start message
    pub async fn handle_transfer_start(
        &self,
        msg: TransferStartMessage,
    ) -> Result<TransferAckMessage> {
        info!("Starting transfer: {} ({})", msg.transfer_id, msg.file_name);

        // Validate file size
        if msg.file_size > self.config.max_file_size {
            return Err(TransferError::PermissionDenied(format!(
                "File size {} exceeds maximum {}",
                msg.file_size, self.config.max_file_size
            )));
        }

        // Validate chunk size
        if msg.chunk_size > self.config.chunk_size * 4 {
            return Err(TransferError::InvalidChunkSize {
                chunk_index: 0,
                expected: self.config.chunk_size,
                actual: msg.chunk_size,
            });
        }

        // Create transfer directories
        self.storage.create_transfer(&msg.transfer_id).await?;

        // Create transfer state
        let state = TransferState {
            transfer_id: msg.transfer_id.clone(),
            file_name: msg.file_name.clone(),
            file_size: msg.file_size,
            chunk_size: msg.chunk_size,
            total_chunks: msg.total_chunks,
            received_chunks: Default::default(),
            blake3_hash: msg.blake3_hash.clone(),
            started_at: chrono::Utc::now().to_rfc3339(),
            last_activity: chrono::Utc::now().to_rfc3339(),
            status: TransferStatus::InProgress,
        };

        // Save metadata
        self.storage.save_metadata(&state).await?;

        // Create transfer session
        let session = Arc::new(RwLock::new(TransferSession {
            state,
            validator: Arc::new(RwLock::new(HashValidator::new())),
            started_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        }));

        // Register transfer
        self.active_transfers
            .write()
            .await
            .insert(msg.transfer_id.clone(), session);

        Ok(TransferAckMessage {
            transfer_id: msg.transfer_id,
            timestamp: current_timestamp(),
            accepted: true,
            resume_supported: true,
            max_chunk_size: self.config.chunk_size,
        })
    }

    /// Handle chunk data message
    pub async fn handle_chunk_data(&self, msg: ChunkDataMessage, data: Vec<u8>) -> Result<ChunkAckMessage> {
        debug!(
            "Received chunk {} for transfer {}",
            msg.chunk_index, msg.transfer_id
        );

        // Get transfer session
        let transfers = self.active_transfers.read().await;
        let session = transfers
            .get(&msg.transfer_id)
            .ok_or_else(|| TransferError::TransferNotFound(msg.transfer_id.clone()))?
            .clone();
        drop(transfers);

        // Validate chunk size
        if data.len() != msg.chunk_size {
            return Err(TransferError::InvalidChunkSize {
                chunk_index: msg.chunk_index,
                expected: msg.chunk_size,
                actual: data.len(),
            });
        }

        // Validate chunk hash
        let computed_hash = hash_data(&data);
        if !computed_hash.eq_ignore_ascii_case(&msg.chunk_hash) {
            return Err(TransferError::ChunkHashMismatch {
                chunk_index: msg.chunk_index,
                expected: msg.chunk_hash,
                actual: computed_hash,
            });
        }

        // Save chunk to disk
        self.storage
            .save_chunk(&msg.transfer_id, msg.chunk_index, &data)
            .await?;

        // Update session state
        let mut session_guard = session.write().await;
        session_guard.state.received_chunks.insert(msg.chunk_index);
        session_guard.state.last_activity = chrono::Utc::now().to_rfc3339();
        session_guard.last_activity = SystemTime::now();

        // Update validator with chunk data
        session_guard.validator.write().await.update(&data);

        // Save updated metadata
        self.storage.save_metadata(&session_guard.state).await?;

        Ok(ChunkAckMessage {
            transfer_id: msg.transfer_id,
            timestamp: current_timestamp(),
            chunk_index: msg.chunk_index,
            received: true,
            hash_valid: true,
        })
    }

    /// Handle transfer complete message
    pub async fn handle_transfer_complete(
        &self,
        msg: TransferCompleteMessage,
    ) -> Result<TransferSuccessMessage> {
        info!("Completing transfer: {}", msg.transfer_id);

        // Get transfer session
        let transfers = self.active_transfers.read().await;
        let session = transfers
            .get(&msg.transfer_id)
            .ok_or_else(|| TransferError::TransferNotFound(msg.transfer_id.clone()))?
            .clone();
        drop(transfers);

        let session_guard = session.read().await;

        // Verify all chunks received
        if session_guard.state.received_chunks.len() as u32 != msg.total_chunks {
            return Err(TransferError::ChunkOutOfOrder {
                expected: msg.total_chunks,
                received: session_guard.state.received_chunks.len() as u32,
            });
        }

        // Assemble file
        let final_path = self
            .storage
            .assemble_file(
                &msg.transfer_id,
                &session_guard.state.file_name,
                msg.total_chunks,
            )
            .await?;

        // Compute final hash from validator
        let computed_hash = session_guard.validator.read().await.finalize_hex();

        // Verify final hash
        if !computed_hash.eq_ignore_ascii_case(&msg.final_hash) {
            return Err(TransferError::FileHashMismatch {
                expected: msg.final_hash,
                actual: computed_hash.clone(),
            });
        }

        drop(session_guard);

        // Update transfer state to complete
        let mut session_guard = session.write().await;
        session_guard.state.status = TransferStatus::Complete;
        self.storage.save_metadata(&session_guard.state).await?;

        // Remove from active transfers
        self.active_transfers.write().await.remove(&msg.transfer_id);

        info!(
            "Transfer complete: {} -> {:?}",
            msg.transfer_id, final_path
        );

        Ok(TransferSuccessMessage {
            transfer_id: msg.transfer_id,
            timestamp: current_timestamp(),
            verified: true,
            saved_path: final_path.to_string_lossy().to_string(),
            received_chunks: msg.total_chunks,
            received_bytes: msg.total_bytes,
            computed_hash,
        })
    }

    /// Handle resume request message
    pub async fn handle_resume_request(
        &self,
        msg: ResumeRequestMessage,
    ) -> Result<ResumeInfoMessage> {
        info!("Resume request for transfer: {}", msg.transfer_id);

        // Load transfer metadata
        let state = self.storage.load_metadata(&msg.transfer_id).await?;

        // Validate file matches
        if state.file_name != msg.file_name || state.file_size != msg.file_size {
            return Err(TransferError::TransferNotFound(format!(
                "File mismatch: expected {}/{}, got {}/{}",
                state.file_name, state.file_size, msg.file_name, msg.file_size
            )));
        }

        // Scan for received chunks
        let received_chunks = self.storage.scan_received_chunks(&msg.transfer_id).await?;
        let missing_chunks = TransferStorage::find_missing_chunks(&received_chunks, state.total_chunks);

        let next_chunk_index = missing_chunks.first().copied().unwrap_or(0);
        let received_bytes = TransferStorage::calculate_received_bytes(&received_chunks, state.chunk_size);

        info!(
            "Resume info: {} received, {} missing",
            received_chunks.len(),
            missing_chunks.len()
        );

        Ok(ResumeInfoMessage {
            transfer_id: msg.transfer_id,
            timestamp: current_timestamp(),
            resumable: !missing_chunks.is_empty(),
            received_chunks: received_chunks.into_iter().collect(),
            missing_chunks,
            next_chunk_index,
            received_bytes,
        })
    }

    /// Handle transfer abort message
    pub async fn handle_transfer_abort(&self, msg: TransferAbortMessage) -> Result<()> {
        warn!("Aborting transfer: {} ({})", msg.transfer_id, msg.reason);

        // Remove from active transfers
        self.active_transfers.write().await.remove(&msg.transfer_id);

        // Update state if exists
        if let Ok(mut state) = self.storage.load_metadata(&msg.transfer_id).await {
            state.status = TransferStatus::Failed;
            self.storage.save_metadata(&state).await?;
        }

        Ok(())
    }

    /// Clean up expired transfers
    pub async fn cleanup_expired_transfers(&self) -> Result<usize> {
        let mut cleaned = 0;
        let timeout = std::time::Duration::from_secs(self.config.transfer_timeout_secs);

        let transfers = self.active_transfers.read().await;
        let now = SystemTime::now();

        for (transfer_id, session) in transfers.iter() {
            let session_guard = session.read().await;
            if let Ok(elapsed) = now.duration_since(session_guard.last_activity) {
                if elapsed > timeout {
                    info!("Cleaning up expired transfer: {}", transfer_id);
                    drop(session_guard);
                    self.storage.cleanup_transfer(transfer_id).await?;
                    cleaned += 1;
                }
            }
        }

        Ok(cleaned)
    }

    /// Get active transfer count
    pub async fn active_transfer_count(&self) -> usize {
        self.active_transfers.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn test_config() -> TransferConfig {
        let temp_dir = TempDir::new().unwrap();
        TransferConfig {
            storage_path: temp_dir.into_path(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_handler_initialization() {
        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();
        assert_eq!(handler.active_transfer_count().await, 0);
    }

    #[tokio::test]
    async fn test_transfer_start() {
        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();

        let msg = TransferStartMessage {
            transfer_id: "test-123".to_string(),
            timestamp: current_timestamp(),
            file_name: "test.txt".to_string(),
            file_size: 1024,
            chunk_size: 512,
            total_chunks: 2,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "abc123".to_string(),
            metadata: None,
        };

        let ack = handler.handle_transfer_start(msg).await.unwrap();
        assert!(ack.accepted);
        assert!(ack.resume_supported);
        assert_eq!(handler.active_transfer_count().await, 1);
    }

    #[tokio::test]
    async fn test_chunk_data_validation() {
        use crate::file_transfer::validation::hash_data;

        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();

        // Start transfer
        let transfer_id = "test-chunk-validation".to_string();
        let start_msg = TransferStartMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            file_name: "test.txt".to_string(),
            file_size: 1024,
            chunk_size: 512,
            total_chunks: 2,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "abc123".to_string(),
            metadata: None,
        };

        handler.handle_transfer_start(start_msg).await.unwrap();

        // Send chunk with valid hash
        let chunk_data = b"Hello, World!";
        let chunk_hash = hash_data(chunk_data);

        let chunk_msg = ChunkDataMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            chunk_index: 0,
            chunk_size: chunk_data.len(),
            chunk_hash: chunk_hash.clone(),
        };

        let ack = handler.handle_chunk_data(chunk_msg, chunk_data.to_vec()).await.unwrap();
        assert!(ack.received);
        assert!(ack.hash_valid);
        assert_eq!(ack.chunk_index, 0);
    }

    #[tokio::test]
    async fn test_chunk_hash_mismatch() {
        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();

        // Start transfer
        let transfer_id = "test-hash-mismatch".to_string();
        let start_msg = TransferStartMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            file_name: "test.txt".to_string(),
            file_size: 1024,
            chunk_size: 512,
            total_chunks: 2,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "abc123".to_string(),
            metadata: None,
        };

        handler.handle_transfer_start(start_msg).await.unwrap();

        // Send chunk with invalid hash
        let chunk_data = b"Hello, World!";
        let chunk_msg = ChunkDataMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            chunk_index: 0,
            chunk_size: chunk_data.len(),
            chunk_hash: "invalid_hash".to_string(),
        };

        let result = handler.handle_chunk_data(chunk_msg, chunk_data.to_vec()).await;
        assert!(result.is_err());

        match result {
            Err(TransferError::ChunkHashMismatch { chunk_index, .. }) => {
                assert_eq!(chunk_index, 0);
            }
            _ => panic!("Expected ChunkHashMismatch error"),
        }
    }

    #[tokio::test]
    async fn test_chunk_size_validation() {
        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();

        // Start transfer
        let transfer_id = "test-chunk-size".to_string();
        let chunk_size = 512;
        let start_msg = TransferStartMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            file_name: "test.txt".to_string(),
            file_size: 1024,
            chunk_size,
            total_chunks: 2,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "abc123".to_string(),
            metadata: None,
        };

        handler.handle_transfer_start(start_msg).await.unwrap();

        // Send chunk with incorrect size
        let chunk_data = vec![0u8; chunk_size + 100];
        let chunk_hash = crate::file_transfer::validation::hash_data(&chunk_data);

        let chunk_msg = ChunkDataMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            chunk_index: 0,
            chunk_size: chunk_size, // Declared size doesn't match actual
            chunk_hash,
        };

        let result = handler.handle_chunk_data(chunk_msg, chunk_data).await;
        assert!(result.is_err());

        match result {
            Err(TransferError::InvalidChunkSize { .. }) => {}
            _ => panic!("Expected InvalidChunkSize error"),
        }
    }

    #[tokio::test]
    async fn test_resume_request() {
        use crate::file_transfer::validation::hash_data;

        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();

        // Start transfer
        let transfer_id = "test-resume".to_string();
        let start_msg = TransferStartMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            file_name: "test.txt".to_string(),
            file_size: 2048,
            chunk_size: 512,
            total_chunks: 4,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "abc123".to_string(),
            metadata: None,
        };

        handler.handle_transfer_start(start_msg).await.unwrap();

        // Send first 2 chunks
        for chunk_index in 0..2 {
            let chunk_data = vec![42u8; 512];
            let chunk_hash = hash_data(&chunk_data);

            let chunk_msg = ChunkDataMessage {
                transfer_id: transfer_id.clone(),
                timestamp: current_timestamp(),
                chunk_index,
                chunk_size: 512,
                chunk_hash,
            };

            handler.handle_chunk_data(chunk_msg, chunk_data).await.unwrap();
        }

        // Request resume
        let resume_msg = ResumeRequestMessage {
            transfer_id: transfer_id.clone(),
            timestamp: current_timestamp(),
            file_name: "test.txt".to_string(),
            file_size: 2048,
            original_hash: "abc123".to_string(),
        };

        let resume_info = handler.handle_resume_request(resume_msg).await.unwrap();
        assert!(resume_info.resumable);
        assert_eq!(resume_info.received_chunks.len(), 2);
        assert_eq!(resume_info.missing_chunks.len(), 2);
        assert_eq!(resume_info.next_chunk_index, 2);
        assert_eq!(resume_info.received_bytes, 1024);
    }

    #[tokio::test]
    async fn test_active_transfer_count() {
        let handler = FileTransferHandler::new(test_config());
        handler.initialize().await.unwrap();

        assert_eq!(handler.active_transfer_count().await, 0);

        // Start first transfer
        let msg1 = TransferStartMessage {
            transfer_id: "test-1".to_string(),
            timestamp: current_timestamp(),
            file_name: "test1.txt".to_string(),
            file_size: 1024,
            chunk_size: 512,
            total_chunks: 2,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "abc123".to_string(),
            metadata: None,
        };
        handler.handle_transfer_start(msg1).await.unwrap();
        assert_eq!(handler.active_transfer_count().await, 1);

        // Start second transfer
        let msg2 = TransferStartMessage {
            transfer_id: "test-2".to_string(),
            timestamp: current_timestamp(),
            file_name: "test2.txt".to_string(),
            file_size: 2048,
            chunk_size: 512,
            total_chunks: 4,
            mime_type: Some("text/plain".to_string()),
            blake3_hash: "def456".to_string(),
            metadata: None,
        };
        handler.handle_transfer_start(msg2).await.unwrap();
        assert_eq!(handler.active_transfer_count().await, 2);
    }
}
