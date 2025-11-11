// Windows IPC implementation using Named Pipes
//
// This module provides Windows-specific IPC communication using Named Pipes,
// which are the Windows equivalent of Unix domain sockets.

#[cfg(windows)]
use anyhow::{Context as _, Result};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

#[cfg(windows)]
use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};

use super::ipc::{Request, Response};
use super::ipc_common::{IpcClient, IpcTransport};
use async_trait::async_trait;

/// Maximum concurrent connections
const MAX_CONCURRENT_CONNECTIONS: usize = 100;

/// Maximum message size (1MB)
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Windows Named Pipe IPC server
#[cfg(windows)]
pub struct WindowsIpcServer {
    pipe_name: String,
    semaphore: Arc<Semaphore>,
}

#[cfg(windows)]
impl WindowsIpcServer {
    /// Create a new Windows IPC server
    ///
    /// # Arguments
    /// * `name` - The pipe name (without `\\.\pipe\` prefix)
    pub fn new(name: &str) -> Result<Self> {
        let pipe_name = format!(r"\\.\pipe\{}", name);

        Ok(Self {
            pipe_name,
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS)),
        })
    }

    /// Start the IPC server
    pub async fn start(&self) -> Result<()> {
        info!("Starting Windows Named Pipe server: {}", self.pipe_name);

        loop {
            // Create a new named pipe instance
            let server = ServerOptions::new()
                .first_pipe_instance(true)
                .create(&self.pipe_name)
                .context("Failed to create named pipe")?;

            // Wait for a client to connect
            let mut pipe = server;
            pipe.connect()
                .await
                .context("Failed to accept connection")?;

            debug!("Client connected to named pipe");

            // Acquire semaphore permit for connection limiting
            let permit = self.semaphore.clone().acquire_owned().await;

            // Handle connection in a separate task
            tokio::spawn(async move {
                let _permit = permit; // Hold permit for connection duration

                if let Err(e) = Self::handle_connection(pipe).await {
                    error!("Connection error: {}", e);
                }
            });
        }
    }

    /// Handle a single client connection
    async fn handle_connection(mut pipe: NamedPipeServer) -> Result<()> {
        let mut buffer = vec![0u8; MAX_MESSAGE_SIZE];

        loop {
            // Read message length (4 bytes)
            let mut len_buf = [0u8; 4];
            match pipe.read_exact(&mut len_buf).await {
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
            match pipe.read_exact(data).await {
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
                    Self::send_response(&mut pipe, &error_response).await.ok();
                    continue;
                }
            };

            debug!("Received request: {:?}", request);

            // Process request (this would integrate with actual handler)
            let response = Self::process_request(request).await;

            // Send response
            if let Err(e) = Self::send_response(&mut pipe, &response).await {
                error!("Failed to send response: {}", e);
                break;
            }
        }

        Ok(())
    }

    /// Send a response to the client
    async fn send_response(pipe: &mut NamedPipeServer, response: &Response) -> Result<()> {
        let data = serde_json::to_vec(response)?;

        // Send message length
        let len_bytes = (data.len() as u32).to_le_bytes();
        pipe.write_all(&len_bytes).await?;

        // Send message data
        pipe.write_all(&data).await?;
        pipe.flush().await?;

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
}

#[cfg(windows)]
#[async_trait]
impl IpcTransport for WindowsIpcServer {
    async fn start(&self) -> Result<()> {
        Self::start(self).await
    }

    async fn ping(&self) -> bool {
        // Server-side ping - check if pipe exists (always true if server is running)
        true
    }
}

/// Windows Named Pipe client
#[cfg(windows)]
pub struct WindowsIpcClient {
    pipe_name: String,
}

#[cfg(windows)]
impl WindowsIpcClient {
    /// Create a new Windows IPC client
    pub fn new(name: &str) -> Self {
        Self {
            pipe_name: format!(r"\\.\pipe\{}", name),
        }
    }

    /// Send a request and receive a response
    pub async fn send_request(&self, request: &Request) -> Result<Response> {
        use tokio::net::windows::named_pipe::ClientOptions;

        // Connect to the named pipe
        let mut client = ClientOptions::new()
            .open(&self.pipe_name)
            .context("Failed to connect to named pipe")?;

        // Serialize request
        let data = serde_json::to_vec(request)?;

        // Send message length
        let len_bytes = (data.len() as u32).to_le_bytes();
        client.write_all(&len_bytes).await?;

        // Send message data
        client.write_all(&data).await?;
        client.flush().await?;

        // Read response length
        let mut len_buf = [0u8; 4];
        client.read_exact(&mut len_buf).await?;
        let resp_len = u32::from_le_bytes(len_buf) as usize;

        if resp_len > MAX_MESSAGE_SIZE {
            return Err(anyhow::anyhow!("Response too large"));
        }

        // Read response data
        let mut buffer = vec![0u8; resp_len];
        client.read_exact(&mut buffer).await?;

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
}

#[cfg(windows)]
#[async_trait]
impl IpcClient for WindowsIpcClient {
    async fn send_request(&self, request: &Request) -> Result<Response> {
        Self::send_request(self, request).await
    }

    async fn ping(&self) -> bool {
        Self::ping(self).await
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_named_pipe_creation() {
        let server = WindowsIpcServer::new("orbit-test");
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = WindowsIpcClient::new("orbit-test");
        assert_eq!(client.pipe_name, r"\\.\pipe\orbit-test");
    }

    #[tokio::test]
    async fn test_server_client_communication() {
        let pipe_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

        // Start server in background
        let server = WindowsIpcServer::new(&pipe_name).unwrap();
        tokio::spawn(async move {
            server.start().await.ok();
        });

        // Give server time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Create client and send request
        let client = WindowsIpcClient::new(&pipe_name);

        let request = Request::Status;
        let response = client.send_request(&request).await;

        assert!(response.is_ok());
        match response.unwrap() {
            Response::Status { .. } => {
                // Success
            }
            _ => panic!("Unexpected response type"),
        }
    }
}
