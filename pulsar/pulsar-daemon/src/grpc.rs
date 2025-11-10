//! gRPC server implementation for terminal services
//!
//! Provides high-performance RPC interface using Protocol Buffers

use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::session_manager::{SessionManager, SessionState, SessionType};

// Include generated proto code
pub mod pb {
    include!("generated/pulsar.terminal.rs");
}

use pb::terminal_service_server::{TerminalService, TerminalServiceServer};
use pb::*;

/// gRPC service implementation
pub struct TerminalServiceImpl {
    session_manager: Arc<SessionManager>,
}

impl TerminalServiceImpl {
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Convert internal SessionType to proto SessionType
    fn session_type_to_proto(st: &SessionType) -> i32 {
        match st {
            SessionType::Local => pb::SessionType::Local as i32,
            SessionType::Ssh { .. } => pb::SessionType::Ssh as i32,
            SessionType::Serial { .. } => pb::SessionType::Serial as i32,
        }
    }

    /// Convert internal SessionState to proto SessionState
    fn session_state_to_proto(ss: &SessionState) -> i32 {
        match ss {
            SessionState::Running => pb::SessionState::Running as i32,
            SessionState::Detached => pb::SessionState::Detached as i32,
            SessionState::Stopped => pb::SessionState::Stopped as i32,
        }
    }
}

#[tonic::async_trait]
impl TerminalService for TerminalServiceImpl {
    /// Create a new terminal session
    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionResponse>, Status> {
        let req = request.into_inner();

        debug!("gRPC CreateSession: name={}, cols={}, rows={}", req.name, req.cols, req.rows);

        // Parse session type
        let session_type = match req.config {
            Some(create_session_request::Config::Local(_)) => SessionType::Local,
            Some(create_session_request::Config::Ssh(ssh_config)) => SessionType::Ssh {
                host: ssh_config.host,
                port: ssh_config.port as u16,
            },
            Some(create_session_request::Config::Serial(serial_config)) => SessionType::Serial {
                device: serial_config.device,
            },
            None => SessionType::Local, // Default to local
        };

        // Create session config
        let config = terminal_core::SessionConfig::new(req.name.clone());

        // Create session
        match self.session_manager.create_session(req.name, session_type, config).await {
            Ok(session_id) => {
                info!("gRPC: Created session {}", session_id);
                Ok(Response::new(CreateSessionResponse {
                    session_id: session_id.to_string(),
                    success: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => {
                error!("gRPC: Failed to create session: {}", e);
                Ok(Response::new(CreateSessionResponse {
                    session_id: String::new(),
                    success: false,
                    error_message: e.to_string(),
                }))
            }
        }
    }

    /// List all sessions
    async fn list_sessions(
        &self,
        request: Request<ListSessionsRequest>,
    ) -> Result<Response<ListSessionsResponse>, Status> {
        debug!("gRPC ListSessions");

        let sessions = self.session_manager.list_sessions().await;

        let session_infos: Vec<SessionInfo> = sessions
            .iter()
            .map(|s| SessionInfo {
                session_id: s.id.to_string(),
                name: s.name.clone(),
                r#type: Self::session_type_to_proto(&s.session_type),
                state: Self::session_state_to_proto(&s.state),
                created_at: s.created_at.timestamp(),
                last_active: s.last_active.timestamp(),
                num_clients: s.num_clients as u32,
                cols: 0, // TODO: Get from session
                rows: 0, // TODO: Get from session
            })
            .collect();

        Ok(Response::new(ListSessionsResponse {
            sessions: session_infos,
        }))
    }

    /// Get session info
    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<GetSessionResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;

        debug!("gRPC GetSession: {}", session_id);

        match self.session_manager.get_session(session_id).await {
            Ok(session) => {
                let state = session.state.read().await;
                let last_active = *session.last_active.read().await;
                let num_clients = session.clients.read().await.len();

                Ok(Response::new(GetSessionResponse {
                    session: Some(SessionInfo {
                        session_id: session.id.to_string(),
                        name: session.name.clone(),
                        r#type: Self::session_type_to_proto(&session.session_type),
                        state: Self::session_state_to_proto(&*state),
                        created_at: session.created_at.timestamp(),
                        last_active: last_active.timestamp(),
                        num_clients: num_clients as u32,
                        cols: 0,
                        rows: 0,
                    }),
                    success: true,
                    error_message: String::new(),
                }))
            }
            Err(e) => Ok(Response::new(GetSessionResponse {
                session: None,
                success: false,
                error_message: e.to_string(),
            })),
        }
    }

    /// Terminate a session
    async fn terminate_session(
        &self,
        request: Request<TerminateSessionRequest>,
    ) -> Result<Response<TerminateSessionResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;

        debug!("gRPC TerminateSession: {}", session_id);

        match self.session_manager.terminate_session(session_id).await {
            Ok(()) => Ok(Response::new(TerminateSessionResponse {
                success: true,
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(TerminateSessionResponse {
                success: false,
                error_message: e.to_string(),
            })),
        }
    }

    /// Attach client to session
    async fn attach_session(
        &self,
        request: Request<AttachSessionRequest>,
    ) -> Result<Response<AttachSessionResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;
        let client_id = Uuid::parse_str(&req.client_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid client ID: {}", e)))?;

        debug!("gRPC AttachSession: session={}, client={}", session_id, client_id);

        match self.session_manager.attach_client(session_id, client_id).await {
            Ok(()) => Ok(Response::new(AttachSessionResponse {
                success: true,
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(AttachSessionResponse {
                success: false,
                error_message: e.to_string(),
            })),
        }
    }

    /// Detach client from session
    async fn detach_session(
        &self,
        request: Request<DetachSessionRequest>,
    ) -> Result<Response<DetachSessionResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;
        let client_id = Uuid::parse_str(&req.client_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid client ID: {}", e)))?;

        debug!("gRPC DetachSession: session={}, client={}", session_id, client_id);

        match self.session_manager.detach_client(session_id, client_id).await {
            Ok(()) => Ok(Response::new(DetachSessionResponse {
                success: true,
                error_message: String::new(),
            })),
            Err(e) => Ok(Response::new(DetachSessionResponse {
                success: false,
                error_message: e.to_string(),
            })),
        }
    }

    /// Stream terminal output (server streaming)
    type StreamOutputStream = Pin<Box<dyn Stream<Item = Result<TerminalOutput, Status>> + Send>>;

    async fn stream_output(
        &self,
        request: Request<StreamOutputRequest>,
    ) -> Result<Response<Self::StreamOutputStream>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;

        debug!("gRPC StreamOutput: {}", session_id);

        // Get session
        let session = self
            .session_manager
            .get_session(session_id)
            .await
            .map_err(|e| Status::not_found(e.to_string()))?;

        // Subscribe to output broadcast
        let mut output_rx = session.output_broadcast.subscribe();

        // Create channel for gRPC stream
        let (tx, rx) = mpsc::channel(128);

        // Spawn task to forward broadcast to gRPC stream
        tokio::spawn(async move {
            let mut sequence = 0u64;
            while let Ok(data) = output_rx.recv().await {
                let output = TerminalOutput {
                    data,
                    sequence,
                    timestamp: chrono::Utc::now().timestamp(),
                    session_id: session_id.to_string(),
                };
                sequence += 1;

                if tx.send(Ok(output)).await.is_err() {
                    break;
                }
            }
        });

        let output_stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::StreamOutputStream))
    }

    /// Stream terminal input (client streaming)
    async fn stream_input(
        &self,
        request: Request<tonic::Streaming<TerminalInput>>,
    ) -> Result<Response<StreamInputResponse>, Status> {
        let mut stream = request.into_inner();
        let mut bytes_written = 0u64;

        while let Some(result) = stream.next().await {
            match result {
                Ok(input) => {
                    let session_id = Uuid::parse_str(&input.session_id)
                        .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;

                    match self.session_manager.get_session(session_id).await {
                        Ok(session) => {
                            let mut terminal = session.terminal_session.write().await;
                            match terminal.write(&input.data) {
                                Ok(n) => bytes_written += n as u64,
                                Err(e) => {
                                    return Ok(Response::new(StreamInputResponse {
                                        bytes_written,
                                        success: false,
                                        error_message: e.to_string(),
                                    }));
                                }
                            }
                        }
                        Err(e) => {
                            return Ok(Response::new(StreamInputResponse {
                                bytes_written,
                                success: false,
                                error_message: e.to_string(),
                            }));
                        }
                    }
                }
                Err(e) => {
                    return Ok(Response::new(StreamInputResponse {
                        bytes_written,
                        success: false,
                        error_message: e.to_string(),
                    }));
                }
            }
        }

        Ok(Response::new(StreamInputResponse {
            bytes_written,
            success: true,
            error_message: String::new(),
        }))
    }

    /// Bidirectional streaming (full duplex)
    type StreamBidirectionalStream =
        Pin<Box<dyn Stream<Item = Result<TerminalOutput, Status>> + Send>>;

    async fn stream_bidirectional(
        &self,
        request: Request<tonic::Streaming<TerminalInput>>,
    ) -> Result<Response<Self::StreamBidirectionalStream>, Status> {
        // TODO: Implement bidirectional streaming
        // This combines StreamOutput and StreamInput for full duplex communication
        Err(Status::unimplemented("Bidirectional streaming not yet implemented"))
    }

    /// Resize terminal
    async fn resize_terminal(
        &self,
        request: Request<ResizeTerminalRequest>,
    ) -> Result<Response<ResizeTerminalResponse>, Status> {
        let req = request.into_inner();
        let session_id = Uuid::parse_str(&req.session_id)
            .map_err(|e| Status::invalid_argument(format!("Invalid session ID: {}", e)))?;

        debug!("gRPC ResizeTerminal: {} to {}x{}", session_id, req.cols, req.rows);

        match self.session_manager.get_session(session_id).await {
            Ok(session) => {
                let mut terminal = session.terminal_session.write().await;
                match terminal.resize(req.cols as u16, req.rows as u16) {
                    Ok(()) => Ok(Response::new(ResizeTerminalResponse {
                        success: true,
                        error_message: String::new(),
                    })),
                    Err(e) => Ok(Response::new(ResizeTerminalResponse {
                        success: false,
                        error_message: e.to_string(),
                    })),
                }
            }
            Err(e) => Ok(Response::new(ResizeTerminalResponse {
                success: false,
                error_message: e.to_string(),
            })),
        }
    }

    /// Send signal to terminal
    async fn send_signal(
        &self,
        request: Request<SendSignalRequest>,
    ) -> Result<Response<SendSignalResponse>, Status> {
        // TODO: Implement signal sending
        Err(Status::unimplemented("Send signal not yet implemented"))
    }

    /// Get daemon status
    async fn get_daemon_status(
        &self,
        _request: Request<GetDaemonStatusRequest>,
    ) -> Result<Response<GetDaemonStatusResponse>, Status> {
        let num_sessions = self.session_manager.count_sessions().await;
        let num_clients = self.session_manager.count_clients().await;

        Ok(Response::new(GetDaemonStatusResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 0, // TODO: Track uptime
            num_sessions: num_sessions as u32,
            num_clients: num_clients as u32,
            config: std::collections::HashMap::new(),
        }))
    }

    /// Health check
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            healthy: true,
            message: "Pulsar daemon is healthy".to_string(),
        }))
    }
}

/// Create gRPC server
pub fn create_server(session_manager: Arc<SessionManager>) -> TerminalServiceServer<TerminalServiceImpl> {
    let service = TerminalServiceImpl::new(session_manager);
    TerminalServiceServer::new(service)
}

/// Start gRPC server
pub async fn start_server(
    session_manager: Arc<SessionManager>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let addr = format!("127.0.0.1:{}", port).parse()?;
    let server = create_server(session_manager);

    info!("gRPC server listening on {}", addr);

    tonic::transport::Server::builder()
        .add_service(server)
        .serve(addr)
        .await?;

    Ok(())
}
