//! IPC server for communication with desktop client
//!
//! Implements Unix socket server with JSON-RPC style protocol

use anyhow::{anyhow, Context, Result};
use base64::{Engine as _, engine::general_purpose};
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::protocol::{
    error_codes, AttachSessionParams, CreateSessionParams, CreateSessionResult,
    DetachSessionParams, ListSessionsResult, ReceiveOutputParams, Request, Response,
    ResizeTerminalParams, SendInputParams, StatusResult, TerminateSessionParams,
};
use crate::session_manager::{SessionManager, SessionType};
use terminal_core::SessionConfig;

/// IPC server managing Unix socket communication
pub struct IpcServer {
    listener: UnixListener,
    session_manager: Arc<SessionManager>,
    start_time: SystemTime,
    shutdown: Arc<RwLock<bool>>,
}

impl IpcServer {
    /// Create a new IPC server
    pub async fn new<P: AsRef<Path>>(
        socket_path: P,
        session_manager: Arc<SessionManager>,
    ) -> Result<Self> {
        let socket_path = socket_path.as_ref();

        // Remove existing socket file if it exists
        if socket_path.exists() {
            std::fs::remove_file(socket_path)
                .with_context(|| format!("Failed to remove existing socket: {:?}", socket_path))?;
        }

        // Create parent directory if needed
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create socket directory: {:?}", parent))?;
        }

        // Bind Unix socket
        let listener = UnixListener::bind(socket_path)
            .with_context(|| format!("Failed to bind socket: {:?}", socket_path))?;

        info!("IPC server listening on {:?}", socket_path);

        Ok(Self {
            listener,
            session_manager,
            start_time: SystemTime::now(),
            shutdown: Arc::new(RwLock::new(false)),
        })
    }

    /// Run the IPC server (accepts connections)
    pub async fn run(&self) -> Result<()> {
        info!("IPC server started");

        loop {
            // Check for shutdown signal
            if *self.shutdown.read().await {
                info!("IPC server shutting down");
                break;
            }

            // Accept connection
            match self.listener.accept().await {
                Ok((stream, _addr)) => {
                    debug!("New IPC client connected");

                    let session_manager = Arc::clone(&self.session_manager);
                    let start_time = self.start_time;

                    // Spawn task to handle this client
                    tokio::spawn(async move {
                        if let Err(e) = Self::handle_client(stream, session_manager, start_time).await {
                            error!("Client handler error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Signal shutdown to the server
    pub async fn shutdown(&self) {
        *self.shutdown.write().await = true;
    }

    /// Handle a single client connection
    async fn handle_client(
        stream: UnixStream,
        session_manager: Arc<SessionManager>,
        start_time: SystemTime,
    ) -> Result<()> {
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();

            // Read line (JSON-RPC request)
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    // EOF - client disconnected
                    debug!("Client disconnected");
                    break;
                }
                Ok(_) => {
                    // Parse and handle request
                    let request: Request = match serde_json::from_str(&line) {
                        Ok(req) => req,
                        Err(e) => {
                            error!("Failed to parse request: {}", e);
                            let response = Response::error(
                                "unknown".to_string(),
                                error_codes::INVALID_REQUEST,
                                format!("Invalid JSON: {}", e),
                            );
                            Self::send_response(&mut writer, &response).await?;
                            continue;
                        }
                    };

                    debug!("Received request: method={}", request.method);

                    // Handle request
                    let response = Self::handle_request(
                        request,
                        Arc::clone(&session_manager),
                        start_time,
                    ).await;

                    // Send response
                    Self::send_response(&mut writer, &response).await?;
                }
                Err(e) => {
                    error!("Failed to read from client: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Handle a single request
    async fn handle_request(
        request: Request,
        session_manager: Arc<SessionManager>,
        start_time: SystemTime,
    ) -> Response {
        match request.method.as_str() {
            "create_session" => {
                Self::handle_create_session(request, session_manager).await
            }
            "list_sessions" => {
                Self::handle_list_sessions(request, session_manager).await
            }
            "attach_session" => {
                Self::handle_attach_session(request, session_manager).await
            }
            "detach_session" => {
                Self::handle_detach_session(request, session_manager).await
            }
            "terminate_session" => {
                Self::handle_terminate_session(request, session_manager).await
            }
            "resize_terminal" => {
                Self::handle_resize_terminal(request, session_manager).await
            }
            "send_input" => {
                Self::handle_send_input(request, session_manager).await
            }
            "receive_output" => {
                Self::handle_receive_output(request, session_manager).await
            }
            "get_status" => {
                Self::handle_get_status(request, session_manager, start_time).await
            }
            _ => Response::error(
                request.id,
                error_codes::METHOD_NOT_FOUND,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    /// Send response to client
    async fn send_response(writer: &mut tokio::net::unix::OwnedWriteHalf, response: &Response) -> Result<()> {
        let json = serde_json::to_string(response)?;
        writer.write_all(json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;
        Ok(())
    }

    // ===== Request Handlers =====

    async fn handle_create_session(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: CreateSessionParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        // Create session config
        let mut config = SessionConfig::new(params.name.clone());
        if let Some(cols) = params.cols {
            config.pty_config.cols = cols;
        }
        if let Some(rows) = params.rows {
            config.pty_config.rows = rows;
        }

        // Create session
        match session_manager
            .create_session(params.name, params.session_type, config)
            .await
        {
            Ok(session_id) => {
                Response::success(request.id, CreateSessionResult { session_id })
            }
            Err(e) => Response::error(
                request.id,
                error_codes::INTERNAL_ERROR,
                format!("Failed to create session: {}", e),
            ),
        }
    }

    async fn handle_list_sessions(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let sessions = session_manager.list_sessions().await;
        Response::success(request.id, ListSessionsResult { sessions })
    }

    async fn handle_attach_session(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: AttachSessionParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        match session_manager
            .attach_client(params.session_id, params.client_id)
            .await
        {
            Ok(_) => Response::success(request.id, serde_json::json!({"success": true})),
            Err(e) => Response::error(
                request.id,
                error_codes::SESSION_NOT_FOUND,
                format!("Failed to attach to session: {}", e),
            ),
        }
    }

    async fn handle_detach_session(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: DetachSessionParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        match session_manager
            .detach_client(params.session_id, params.client_id)
            .await
        {
            Ok(_) => Response::success(request.id, serde_json::json!({"success": true})),
            Err(e) => Response::error(
                request.id,
                error_codes::SESSION_NOT_FOUND,
                format!("Failed to detach from session: {}", e),
            ),
        }
    }

    async fn handle_terminate_session(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: TerminateSessionParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        match session_manager.terminate_session(params.session_id).await {
            Ok(_) => Response::success(request.id, serde_json::json!({"success": true})),
            Err(e) => Response::error(
                request.id,
                error_codes::SESSION_NOT_FOUND,
                format!("Failed to terminate session: {}", e),
            ),
        }
    }

    async fn handle_resize_terminal(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: ResizeTerminalParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        match session_manager.get_session(params.session_id).await {
            Ok(session) => {
                let mut terminal_session = session.terminal_session.write().await;
                match terminal_session.resize(params.cols, params.rows) {
                    Ok(_) => Response::success(request.id, serde_json::json!({"success": true})),
                    Err(e) => Response::error(
                        request.id,
                        error_codes::INTERNAL_ERROR,
                        format!("Failed to resize terminal: {}", e),
                    ),
                }
            }
            Err(e) => Response::error(
                request.id,
                error_codes::SESSION_NOT_FOUND,
                format!("Session not found: {}", e),
            ),
        }
    }

    async fn handle_send_input(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: SendInputParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        // Decode base64 data
        let data = match general_purpose::STANDARD.decode(&params.data) {
            Ok(d) => d,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid base64 data: {}", e),
                );
            }
        };

        // Get session and write to PTY
        match session_manager.get_session(params.session_id).await {
            Ok(session) => {
                let mut terminal_session = session.terminal_session.write().await;
                match terminal_session.write(&data) {
                    Ok(bytes_written) => {
                        Response::success(request.id, serde_json::json!({
                            "bytes_written": bytes_written
                        }))
                    }
                    Err(e) => Response::error(
                        request.id,
                        error_codes::INTERNAL_ERROR,
                        format!("Failed to write to PTY: {}", e),
                    ),
                }
            }
            Err(e) => Response::error(
                request.id,
                error_codes::SESSION_NOT_FOUND,
                format!("Session not found: {}", e),
            ),
        }
    }

    async fn handle_receive_output(
        request: Request,
        session_manager: Arc<SessionManager>,
    ) -> Response {
        let params: ReceiveOutputParams = match serde_json::from_value(request.params) {
            Ok(p) => p,
            Err(e) => {
                return Response::error(
                    request.id,
                    error_codes::INVALID_PARAMS,
                    format!("Invalid parameters: {}", e),
                );
            }
        };

        // Get session and read from PTY
        match session_manager.get_session(params.session_id).await {
            Ok(session) => {
                let mut terminal_session = session.terminal_session.write().await;

                // Buffer for reading PTY output
                let mut buffer = vec![0u8; 4096];

                // Try to read (non-blocking if no timeout specified)
                let read_result = if params.timeout_ms.is_some() {
                    // Blocking read with timeout
                    terminal_session.read(&mut buffer)
                } else {
                    // Non-blocking read
                    terminal_session.try_read(&mut buffer)
                };

                match read_result {
                    Ok(bytes_read) => {
                        // Encode output as base64
                        let output_data = general_purpose::STANDARD.encode(&buffer[..bytes_read]);
                        Response::success(request.id, serde_json::json!({
                            "data": output_data,
                            "bytes_read": bytes_read
                        }))
                    }
                    Err(e) => Response::error(
                        request.id,
                        error_codes::INTERNAL_ERROR,
                        format!("Failed to read from PTY: {}", e),
                    ),
                }
            }
            Err(e) => Response::error(
                request.id,
                error_codes::SESSION_NOT_FOUND,
                format!("Session not found: {}", e),
            ),
        }
    }

    async fn handle_get_status(
        request: Request,
        session_manager: Arc<SessionManager>,
        start_time: SystemTime,
    ) -> Response {
        let uptime = start_time
            .elapsed()
            .unwrap_or_default()
            .as_secs();

        let status = StatusResult {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: uptime,
            num_sessions: session_manager.count_sessions().await,
            num_clients: session_manager.count_clients().await,
        };

        Response::success(request.id, status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_ipc_server_creation() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");
        let session_manager = Arc::new(SessionManager::new());

        let server = IpcServer::new(&socket_path, session_manager).await.unwrap();
        assert!(socket_path.exists());
    }
}
