//! SSH/SFTP transport implementation

use crate::transport::{Transport, TransportConfig, TransportError};
use async_trait::async_trait;

pub struct SshTransport {
    // TODO: russh session
}

impl SshTransport {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Transport for SshTransport {
    async fn connect(&mut self, _config: &TransportConfig) -> Result<(), TransportError> {
        // TODO: Implement SSH connection
        Ok(())
    }

    async fn send(&mut self, _data: &[u8]) -> Result<(), TransportError> {
        // TODO: Send over SSH
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        // TODO: Receive from SSH
        Ok(vec![])
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        // TODO: Close SSH connection
        Ok(())
    }
}
