//! TFT Transport Layer Implementations
//!
//! This crate provides multiple transport backends for TFT:
//! - QUIC/HTTP/3 (primary, high-performance)
//! - SSH/SFTP (fallback, compatibility)
//! - WebRTC (peer-to-peer, future)

pub mod transport;

#[cfg(feature = "quic")]
pub mod quic;

#[cfg(feature = "ssh")]
pub mod ssh;

#[cfg(feature = "ssh")]
pub mod ssh_client;

#[cfg(feature = "ssh")]
pub mod ssh_simple;

#[cfg(feature = "ssh")]
pub mod known_hosts;

#[cfg(feature = "webrtc")]
pub mod webrtc;

pub use transport::{Transport, TransportConfig, TransportError};

#[cfg(feature = "ssh")]
pub use ssh_client::{SshSession, SshConfig, AuthMethod, spawn_ssh_io};

#[cfg(feature = "ssh")]
pub use known_hosts::{KnownHosts, HostKeyVerification};

#[cfg(test)]
mod tests {
    #[test]
    fn test_placeholder() {
        assert!(true);
    }
}
