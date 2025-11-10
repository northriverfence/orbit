//! WebTransport server for modern HTTP/3-based streaming
//!
//! Provides:
//! - QUIC-based transport (HTTP/3)
//! - 0-RTT connection establishment
//! - Multiple concurrent streams
//! - Better congestion control than WebSocket
//! - File transfer support

use anyhow::{Context, Result};
use quinn::{Endpoint, ServerConfig};
use rustls::crypto::ring;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::file_transfer::{FileTransferHandler, TransferMessage};
use crate::session_manager::SessionManager;

/// WebTransport server state
pub struct WebTransportServer {
    endpoint: Endpoint,
    session_manager: Arc<SessionManager>,
    file_transfer: Arc<FileTransferHandler>,
}

impl WebTransportServer {
    /// Create a new WebTransport server
    pub fn new(
        bind_addr: SocketAddr,
        session_manager: Arc<SessionManager>,
        file_transfer: Arc<FileTransferHandler>,
        cert: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
    ) -> Result<Self> {
        // Create server config
        let server_config = configure_server(cert, key)?;

        // Create endpoint
        let endpoint = Endpoint::server(server_config, bind_addr)
            .context("Failed to create QUIC endpoint")?;

        Ok(Self {
            endpoint,
            session_manager,
            file_transfer,
        })
    }

    /// Start accepting connections
    pub async fn run(self) -> Result<()> {
        info!(
            "WebTransport server listening on {}",
            self.endpoint.local_addr()?
        );

        while let Some(connecting) = self.endpoint.accept().await {
            let session_manager = Arc::clone(&self.session_manager);
            let file_transfer = Arc::clone(&self.file_transfer);

            tokio::spawn(async move {
                match connecting.await {
                    Ok(connection) => {
                        info!("New WebTransport connection from {}", connection.remote_address());

                        if let Err(e) = handle_connection(connection, session_manager, file_transfer).await {
                            error!("Connection handler error: {}", e);
                        }
                    }
                    Err(e) => {
                        error!("Connection failed: {}", e);
                    }
                }
            });
        }

        Ok(())
    }
}

/// Handle a WebTransport connection
async fn handle_connection(
    connection: quinn::Connection,
    session_manager: Arc<SessionManager>,
    file_transfer: Arc<FileTransferHandler>,
) -> Result<()> {
    debug!("Handling WebTransport connection");

    loop {
        tokio::select! {
            // Accept bidirectional streams
            stream_result = connection.accept_bi() => {
                match stream_result {
                    Ok((send, recv)) => {
                        let session_manager = Arc::clone(&session_manager);
                        let file_transfer = Arc::clone(&file_transfer);
                        tokio::spawn(async move {
                            if let Err(e) = handle_bidirectional_stream(send, recv, session_manager, file_transfer).await {
                                error!("Bidirectional stream error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        debug!("Connection closed: {}", e);
                        break;
                    }
                }
            }

            // Accept unidirectional streams
            stream_result = connection.accept_uni() => {
                match stream_result {
                    Ok(recv) => {
                        let session_manager = Arc::clone(&session_manager);
                        tokio::spawn(async move {
                            if let Err(e) = handle_unidirectional_stream(recv, session_manager).await {
                                error!("Unidirectional stream error: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        debug!("Connection closed: {}", e);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Handle a bidirectional stream (for terminal I/O or file transfer)
async fn handle_bidirectional_stream(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    session_manager: Arc<SessionManager>,
    file_transfer: Arc<FileTransferHandler>,
) -> Result<()> {
    // Read initial message to determine stream type
    let mut buf = vec![0u8; 4096];
    let n = recv.read(&mut buf).await?.context("Stream closed")?;

    // Try to parse as JSON (file transfer) first
    if let Ok(message) = TransferMessage::from_json(&buf[..n]) {
        debug!("File transfer stream: {}", message.transfer_id());
        return handle_file_transfer_stream(send, recv, file_transfer, message).await;
    }

    // Otherwise treat as terminal stream
    let session_id_str = String::from_utf8_lossy(&buf[..n]);
    let session_id = Uuid::parse_str(session_id_str.trim())
        .context("Invalid session ID")?;

    debug!("WebTransport terminal stream for session: {}", session_id);

    // Get session
    let session = session_manager
        .get_session(session_id)
        .await
        .context("Session not found")?;

    // Subscribe to output
    let mut output_rx = session.output_broadcast.subscribe();

    // Spawn output task
    let output_task = tokio::spawn(async move {
        while let Ok(data) = output_rx.recv().await {
            if let Err(e) = send.write_all(&data).await {
                debug!("Failed to write to stream: {}", e);
                break;
            }
        }
    });

    // Handle input
    let input_task = tokio::spawn(async move {
        let mut buf = vec![0u8; 8192];
        loop {
            match recv.read(&mut buf).await {
                Ok(Some(n)) => {
                    let mut terminal = session.terminal_session.write().await;
                    if let Err(e) = terminal.write(&buf[..n]) {
                        error!("Failed to write to PTY: {}", e);
                        break;
                    }
                }
                Ok(None) => {
                    debug!("Stream finished");
                    break;
                }
                Err(e) => {
                    error!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    // Wait for both tasks
    tokio::select! {
        _ = output_task => {},
        _ = input_task => {},
    }

    Ok(())
}

/// Handle a unidirectional stream (for control messages)
async fn handle_unidirectional_stream(
    mut recv: quinn::RecvStream,
    session_manager: Arc<SessionManager>,
) -> Result<()> {
    let mut buf = vec![0u8; 1024];
    let n = recv.read(&mut buf).await?.context("Stream closed")?;
    let message = String::from_utf8_lossy(&buf[..n]);

    debug!("Control message: {}", message);

    // TODO: Handle control messages (resize, signal, etc.)

    Ok(())
}

/// Configure QUIC server with TLS
fn configure_server(
    cert: CertificateDer<'static>,
    key: PrivateKeyDer<'static>,
) -> Result<ServerConfig> {
    // Install crypto provider before using rustls (ignore error if already installed)
    let _ = ring::default_provider().install_default();

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)
        .context("Failed to create TLS config")?;

    // Enable ALPN for WebTransport
    server_crypto.alpn_protocols = vec![b"h3".to_vec()];

    let mut server_config = ServerConfig::with_crypto(Arc::new(
        quinn::crypto::rustls::QuicServerConfig::try_from(server_crypto)
            .context("Failed to create QUIC config")?,
    ));

    // Configure transport
    let mut transport_config = quinn::TransportConfig::default();
    transport_config.max_concurrent_bidi_streams(100u32.into());
    transport_config.max_concurrent_uni_streams(100u32.into());
    transport_config.max_idle_timeout(Some(quinn::IdleTimeout::from(
        quinn::VarInt::from_u32(30_000), // 30 seconds
    )));

    server_config.transport_config(Arc::new(transport_config));

    Ok(server_config)
}

/// Generate self-signed certificate for development
pub fn generate_self_signed_cert() -> Result<(CertificateDer<'static>, PrivateKeyDer<'static>)> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .context("Failed to generate certificate")?;

    let cert_der = cert.cert.der().clone();
    let key_der = PrivateKeyDer::Pkcs8(cert.key_pair.serialize_der().into());

    info!("Generated self-signed certificate for WebTransport");

    Ok((cert_der, key_der))
}

/// Handle a file transfer stream
async fn handle_file_transfer_stream(
    mut send: quinn::SendStream,
    mut recv: quinn::RecvStream,
    file_transfer: Arc<FileTransferHandler>,
    initial_message: TransferMessage,
) -> Result<()> {
    use crate::file_transfer::messages::*;

    debug!("Handling file transfer: {}", initial_message.transfer_id());

    // Process initial message
    let response = match initial_message {
        TransferMessage::TransferStart(msg) => {
            match file_transfer.handle_transfer_start(msg).await {
                Ok(ack) => TransferMessage::TransferAck(ack),
                Err(e) => TransferMessage::Error(ErrorMessage {
                    transfer_id: String::new(),
                    timestamp: current_timestamp(),
                    error_type: "transfer_start_failed".to_string(),
                    error_message: e.to_string(),
                }),
            }
        }
        TransferMessage::ResumeRequest(msg) => {
            match file_transfer.handle_resume_request(msg).await {
                Ok(info) => TransferMessage::ResumeInfo(info),
                Err(e) => TransferMessage::Error(ErrorMessage {
                    transfer_id: String::new(),
                    timestamp: current_timestamp(),
                    error_type: "resume_failed".to_string(),
                    error_message: e.to_string(),
                }),
            }
        }
        _ => {
            error!("Unexpected initial message type");
            return Ok(());
        }
    };

    // Send response
    let response_json = response.to_json()?;
    send.write_all(&response_json).await?;

    // Handle subsequent messages
    loop {
        let mut header_buf = vec![0u8; 4096];
        match recv.read(&mut header_buf).await? {
            Some(n) => {
                if let Ok(message) = TransferMessage::from_json(&header_buf[..n]) {
                    let response = match message {
                        TransferMessage::ChunkData(msg) => {
                            // Read chunk data
                            let chunk_size = msg.chunk_size;
                            let mut chunk_data = vec![0u8; chunk_size];
                            let mut bytes_read = 0;
                            while bytes_read < chunk_size {
                                match recv.read(&mut chunk_data[bytes_read..]).await? {
                                    Some(n) => bytes_read += n,
                                    None => break,
                                }
                            }

                            match file_transfer.handle_chunk_data(msg, chunk_data).await {
                                Ok(ack) => TransferMessage::ChunkAck(ack),
                                Err(e) => TransferMessage::Error(ErrorMessage {
                                    transfer_id: String::new(),
                                    timestamp: current_timestamp(),
                                    error_type: "chunk_failed".to_string(),
                                    error_message: e.to_string(),
                                }),
                            }
                        }
                        TransferMessage::TransferComplete(msg) => {
                            match file_transfer.handle_transfer_complete(msg).await {
                                Ok(success) => TransferMessage::TransferSuccess(success),
                                Err(e) => TransferMessage::Error(ErrorMessage {
                                    transfer_id: String::new(),
                                    timestamp: current_timestamp(),
                                    error_type: "complete_failed".to_string(),
                                    error_message: e.to_string(),
                                }),
                            }
                        }
                        TransferMessage::TransferAbort(msg) => {
                            let _ = file_transfer.handle_transfer_abort(msg).await;
                            break;
                        }
                        _ => {
                            error!("Unexpected message type");
                            continue;
                        }
                    };

                    // Send response
                    let response_json = response.to_json()?;
                    send.write_all(&response_json).await?;

                    // If transfer complete, close stream
                    if matches!(response, TransferMessage::TransferSuccess(_)) {
                        break;
                    }
                } else {
                    error!("Failed to parse transfer message");
                    break;
                }
            }
            None => {
                debug!("File transfer stream closed");
                break;
            }
        }
    }

    Ok(())
}

/// Start WebTransport server
pub async fn start_server(
    session_manager: Arc<SessionManager>,
    file_transfer: Arc<FileTransferHandler>,
    port: u16,
) -> Result<()> {
    // Install crypto provider globally before any rustls operations
    let _ = ring::default_provider().install_default();

    // Generate certificate
    let (cert, key) = generate_self_signed_cert()?;

    // Create server
    let bind_addr: SocketAddr = format!("127.0.0.1:{}", port).parse()?;
    let server = WebTransportServer::new(bind_addr, session_manager, file_transfer, cert, key)?;

    // Run server
    server.run().await
}
