// Cross-platform IPC abstraction layer
//
// This module provides a unified interface for IPC communication that works
// across Unix (domain sockets) and Windows (named pipes) platforms.

use anyhow::Result;
use async_trait::async_trait;

use super::ipc::{Request, Response};

#[cfg(unix)]
use super::ipc_unix::UnixIpcServer;

#[cfg(windows)]
use super::ipc_windows::WindowsIpcServer;

/// Cross-platform IPC server type alias
/// On Unix: Uses Unix domain sockets
/// On Windows: Uses Named Pipes
#[cfg(unix)]
pub type PlatformIpcServer = UnixIpcServer;

#[cfg(windows)]
pub type PlatformIpcServer = WindowsIpcServer;

/// Trait defining the interface for IPC transport implementations
///
/// This trait allows the main server code to work with both Unix sockets
/// and Windows Named Pipes transparently.
#[async_trait]
pub trait IpcTransport: Send + Sync {
    /// Start the IPC server and begin accepting connections
    async fn start(&self) -> Result<()>;

    /// Check if the server is running (client-side)
    async fn ping(&self) -> bool;
}

/// Request handler trait for processing IPC requests
///
/// Implementations of this trait contain the business logic for handling
/// various request types (Command, Feedback, Status, Shutdown).
#[async_trait]
pub trait RequestHandler: Send + Sync {
    /// Process a request and return a response
    async fn handle_request(&self, request: Request) -> Response;
}

/// Configuration for IPC server
#[derive(Debug, Clone)]
pub struct IpcConfig {
    /// Name/identifier for the IPC endpoint
    /// Unix: socket file name
    /// Windows: named pipe name
    pub name: String,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Maximum message size in bytes
    pub max_message_size: usize,
}

impl Default for IpcConfig {
    fn default() -> Self {
        Self {
            name: "orbit".to_string(),
            max_connections: 100,
            max_message_size: 1024 * 1024, // 1MB
        }
    }
}

/// Creates a platform-specific IPC server
///
/// # Arguments
/// * `config` - IPC configuration
///
/// # Returns
/// A boxed IPC server implementation appropriate for the current platform
pub fn create_ipc_server(config: IpcConfig) -> Result<Box<dyn IpcTransport>> {
    #[cfg(unix)]
    {
        let server = UnixIpcServer::new(&config.name)?;
        Ok(Box::new(server))
    }

    #[cfg(windows)]
    {
        let server = WindowsIpcServer::new(&config.name)?;
        Ok(Box::new(server))
    }
}

/// Creates a platform-specific IPC client
///
/// # Arguments
/// * `name` - Name/identifier for the IPC endpoint
///
/// # Returns
/// A boxed IPC client implementation appropriate for the current platform
#[cfg(unix)]
pub fn create_ipc_client(name: &str) -> Box<dyn IpcClient> {
    use super::ipc_unix::UnixIpcClient;
    Box::new(UnixIpcClient::new(name))
}

#[cfg(windows)]
pub fn create_ipc_client(name: &str) -> Box<dyn IpcClient> {
    use super::ipc_windows::WindowsIpcClient;
    Box::new(WindowsIpcClient::new(name))
}

/// Trait for IPC client operations
#[async_trait]
pub trait IpcClient: Send + Sync {
    /// Send a request and wait for a response
    async fn send_request(&self, request: &Request) -> Result<Response>;

    /// Check if the server is running
    async fn ping(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = IpcConfig::default();
        assert_eq!(config.name, "orbit");
        assert_eq!(config.max_connections, 100);
        assert_eq!(config.max_message_size, 1024 * 1024);
    }

    #[test]
    fn test_custom_config() {
        let config = IpcConfig {
            name: "test-orbit".to_string(),
            max_connections: 50,
            max_message_size: 512 * 1024,
        };
        assert_eq!(config.name, "test-orbit");
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.max_message_size, 512 * 1024);
    }

    #[tokio::test]
    async fn test_create_server() {
        let config = IpcConfig {
            name: "test-create".to_string(),
            ..Default::default()
        };

        let server = create_ipc_server(config);
        assert!(server.is_ok());
    }
}
