// Unix IPC implementation using Unix Domain Sockets
//
// This module provides Unix-specific IPC communication using Unix domain sockets,
// which are the standard IPC mechanism on Unix-like systems (Linux, macOS, BSD).

#[cfg(unix)]
use anyhow::{Context as _, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use super::ipc::{Request, Response};
use super::ipc_common::{IpcClient, IpcTransport};
use async_trait::async_trait;

/// Maximum concurrent connections
const MAX_CONCURRENT_CONNECTIONS: usize = 100;

/// Maximum message size (1MB)
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Unix Domain Socket IPC server
#[cfg(unix)]
pub struct UnixIpcServer {
    socket_path: PathBuf,
    semaphore: Arc<Semaphore>,
}

#[cfg(unix)]
impl UnixIpcServer {
    /// Create a new Unix IPC server
    ///
    /// # Arguments
    /// * `name` - The socket file name (will be created in /tmp or XDG_RUNTIME_DIR)
    pub fn new(name: &str) -> Result<Self> {
        // Determine socket directory
        let socket_dir = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir)
        } else {
            PathBuf::from("/tmp")
        };

        let socket_path = socket_dir.join(format!("{}.sock", name));

        Ok(Self {
            socket_path,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS)),
        })
    }

    /// Create a new Unix IPC server with a specific path
    pub fn with_path<P: AsRef<Path>>(socket_path: P) -> Result<Self> {
        Ok(Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS)),
        })
    }

    /// Start the IPC server
    pub async fn start(&self) -> Result<()> {
        // Remove socket if it already exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)
                .context("Failed to remove existing socket")?;
        }

        // Create socket directory if it doesn't exist
        if let Some(parent) = self.socket_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
        }

        let listener = UnixListener::bind(&self.socket_path)
            .context("Failed to bind Unix socket")?;

        info!("Unix socket server listening on: {:?}", self.socket_path);

        // Set permissions (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.socket_path)?.permissions();
            perms.set_mode(0o600); // Owner read/write only
            std::fs::set_permissions(&self.socket_path, perms)?;
        }

        loop {
            // Accept connection
            let (stream, _) = listener.accept().await.context("Failed to accept connection")?;

            debug!("Client connected to Unix socket");

            // Acquire semaphore permit for connection limiting
            let permit = self.semaphore.clone().acquire_owned().await;

            // Handle connection in a separate task
            tokio::spawn(async move {
                let _permit = permit; // Hold permit for connection duration

                if let Err(e) = Self::handle_connection(stream).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }

    /// Handle a single client connection
    async fn handle_connection(mut stream: UnixStream) -> Result<()> {
        let mut buffer = vec![0u8; MAX_MESSAGE_SIZE];

        loop {
            // Read message length (4 bytes)
            let mut len_buf = [0u8; 4];
            match stream.read_exact(&mut len_buf).await {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    debug!("Client disconnected");
                    break;
                }
                Err(e) => {
                    error!("Failed to read message length: {}", e);
                    break;
                }
            }

            let msg_len = u32::from_le_bytes(len_buf) as usize;

            if msg_len > MAX_MESSAGE_SIZE {
                warn!("Message too large: {} bytes", msg_len);
                break;
            }

            // Read message data
            let data = &mut buffer[..msg_len];
            match stream.read_exact(data).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to read message: {}", e);
                    break;
                }
            }

            // Parse request
            let request: Request = match serde_json::from_slice(data) {
                Ok(req) => req,
                Err(e) => {
                    error!("Failed to parse request: {}", e);
                    let error_response = Response::Error {
                        message: format!("Invalid request: {}", e),
                    };
                    Self::send_response(&mut stream, &error_response).await.ok();
                    continue;
                }
            };

            debug!("Received request: {:?}", request);

            // Process request (this would integrate with actual handler)
            let response = Self::process_request(request).await;

            // Send response
            if let Err(e) = Self::send_response(&mut stream, &response).await {
                error!("Failed to send response: {}", e);
                break;
            }
        }

        Ok(())
    }

    /// Send a response to the client
    async fn send_response(stream: &mut UnixStream, response: &Response) -> Result<()> {
        let data = serde_json::to_vec(response)?;

        // Send message length
        let len_bytes = (data.len() as u32).to_le_bytes();
        stream.write_all(&len_bytes).await?;

        // Send message data
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Process an IPC request
    async fn process_request(request: Request) -> Response {
        // This is a stub - in the real implementation, this would integrate
        // with the actual request handler from the server

        match request {
            Request::Command { input, cwd, shell } => {
                debug!(
                    "Processing command: {} (cwd: {}, shell: {})",
                    input, cwd, shell
                );

                // Stub response - real implementation would call classifier, etc.
                Response::Passthrough
            }

            Request::Feedback {
                input,
                executed,
                result,
            } => {
                debug!("Processing feedback: {} -> {} ({:?})", input, executed, result);
                Response::Ok
            }

            Request::Status => Response::Status {
                uptime_secs: 0,
                commands_processed: 0,
            },

            Request::Shutdown => {
                info!("Shutdown requested via IPC");
                Response::Ok
            }
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

#[cfg(unix)]
#[async_trait]
impl IpcTransport for UnixIpcServer {
    async fn start(&self) -> Result<()> {
        Self::start(self).await
    }

    async fn ping(&self) -> bool {
        // Server-side ping - check if socket exists
        self.socket_path.exists()
    }
}

/// Unix Domain Socket client
#[cfg(unix)]
pub struct UnixIpcClient {
    socket_path: PathBuf,
}

#[cfg(unix)]
impl UnixIpcClient {
    /// Create a new Unix IPC client
    pub fn new(name: &str) -> Self {
        // Determine socket directory
        let socket_dir = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir)
        } else {
            PathBuf::from("/tmp")
        };

        Self {
            socket_path: socket_dir.join(format!("{}.sock", name)),
        }
    }

    /// Create a new Unix IPC client with a specific path
    pub fn with_path<P: AsRef<Path>>(socket_path: P) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Send a request and receive a response
    pub async fn send_request(&self, request: &Request) -> Result<Response> {
        // Connect to the Unix socket
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context("Failed to connect to Unix socket")?;

        // Serialize request
        let data = serde_json::to_vec(request)?;

        // Send message length
        let len_bytes = (data.len() as u32).to_le_bytes();
        stream.write_all(&len_bytes).await?;

        // Send message data
        stream.write_all(&data).await?;
        stream.flush().await?;

        // Read response length
        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf).await?;
        let resp_len = u32::from_le_bytes(len_buf) as usize;

        if resp_len > MAX_MESSAGE_SIZE {
            return Err(anyhow::anyhow!("Response too large"));
        }

        // Read response data
        let mut buffer = vec![0u8; resp_len];
        stream.read_exact(&mut buffer).await?;

        // Parse response
        let response: Response = serde_json::from_slice(&buffer)?;

        Ok(response)
    }

    /// Check if the server is running
    pub async fn ping(&self) -> bool {
        match self.send_request(&Request::Status).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Get the socket path
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

#[cfg(unix)]
#[async_trait]
impl IpcClient for UnixIpcClient {
    async fn send_request(&self, request: &Request) -> Result<Response> {
        Self::send_request(self, request).await
    }

    async fn ping(&self) -> bool {
        Self::ping(self).await
    }
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unix_socket_creation() {
        let server = UnixIpcServer::new("orbit-test");
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = UnixIpcClient::new("orbit-test");
        assert!(client.socket_path().to_string_lossy().contains("orbit-test.sock"));
    }

    #[tokio::test]
    async fn test_custom_path() {
        let test_path = "/tmp/test-orbit.sock";
        let server = UnixIpcServer::with_path(test_path).unwrap();
        assert_eq!(server.socket_path(), Path::new(test_path));

        let client = UnixIpcClient::with_path(test_path);
        assert_eq!(client.socket_path(), Path::new(test_path));
    }

    #[tokio::test]
    async fn test_server_client_communication() {
        let socket_path = format!("/tmp/orbit-test-{}.sock", uuid::Uuid::new_v4());

        // Start server in background
        let server = UnixIpcServer::with_path(&socket_path).unwrap();
        tokio::spawn(async move {
            server.start().await.ok();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create client and send request
        let client = UnixIpcClient::with_path(&socket_path);

        let request = Request::Status;
        let response = client.send_request(&request).await;

        assert!(response.is_ok());
        match response.unwrap() {
            Response::Status { .. } => {
                // Success
            }
            _ => panic!("Unexpected response type"),
        }

        // Cleanup
        std::fs::remove_file(&socket_path).ok();
    }
}
