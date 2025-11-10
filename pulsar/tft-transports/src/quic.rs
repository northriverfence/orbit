//! QUIC/HTTP/3 transport implementation

use crate::transport::{Transport, TransportConfig, TransportError};
use async_trait::async_trait;

pub struct QuicTransport {
    // TODO: Quinn connection
}

impl QuicTransport {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Transport for QuicTransport {
    async fn connect(&mut self, _config: &TransportConfig) -> Result<(), TransportError> {
        // TODO: Implement QUIC connection
        Ok(())
    }

    async fn send(&mut self, _data: &[u8]) -> Result<(), TransportError> {
        // TODO: Send over QUIC
        Ok(())
    }

    async fn receive(&mut self) -> Result<Vec<u8>, TransportError> {
        // TODO: Receive from QUIC
        Ok(vec![])
    }

    async fn disconnect(&mut self) -> Result<(), TransportError> {
        // TODO: Close QUIC connection
        Ok(())
    }
}
