//! Transport layer abstraction

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protocol error: {0}")]
    Protocol(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    pub host: String,
    pub port: u16,
    pub timeout_ms: u64,
}

#[async_trait]
pub trait Transport: Send + Sync {
    async fn connect(&mut self, config: &TransportConfig) -> Result<(), TransportError>;
    async fn send(&mut self, data: &[u8]) -> Result<(), TransportError>;
    async fn receive(&mut self) -> Result<Vec<u8>, TransportError>;
    async fn disconnect(&mut self) -> Result<(), TransportError>;
}
