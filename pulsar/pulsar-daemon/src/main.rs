//! Pulsar Daemon
//!
//! Background service that manages:
//! - Active terminal sessions (local, SSH, serial)
//! - Multi-client session sharing
//! - IPC communication via Unix sockets
//! - Session persistence and restoration
//! - File transfers over WebTransport

use anyhow::Result;
use std::sync::Arc;
use tokio::signal;
use tokio::time::{interval, Duration};
use tracing::{error, info, warn};

mod config;
mod file_transfer;
mod grpc;
mod ipc;
mod protocol;
mod session_manager;
mod websocket;
mod webtransport;
mod workspace;

use config::DaemonConfig;
use file_transfer::{FileTransferHandler, TransferConfig};
use ipc::IpcServer;
use session_manager::SessionManager;
use workspace::WorkspaceService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("Starting Pulsar Daemon v{}", env!("CARGO_PKG_VERSION"));

    // Load configuration
    let config = DaemonConfig::load()?;
    info!("Configuration loaded from {:?}", config.socket_path);

    // Initialize session manager
    let session_manager = Arc::new(SessionManager::new());
    info!("Session manager initialized");

    // Initialize file transfer handler
    let file_transfer = Arc::new(FileTransferHandler::new(TransferConfig::default()));
    file_transfer.initialize().await?;
    info!("File transfer handler initialized");

    // Initialize workspace service (database)
    let db_path = dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?
        .join("pulsar")
        .join("workspaces.db");

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let db_url = format!("sqlite:{}", db_path.display());
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    let workspace_service = Arc::new(WorkspaceService::new(Arc::new(pool)));
    workspace_service.initialize().await?;
    info!("Workspace service initialized");

    // TODO: Restore persisted sessions from database

    // Start IPC server
    let ipc_server = Arc::new(
        IpcServer::new(&config.socket_path, Arc::clone(&session_manager)).await?,
    );
    info!("IPC server initialized");

    // Spawn IPC server task
    let ipc_server_handle = {
        let ipc_server = Arc::clone(&ipc_server);
        tokio::spawn(async move {
            if let Err(e) = ipc_server.run().await {
                error!("IPC server error: {}", e);
            }
        })
    };

    // Spawn WebSocket server task
    let ws_server_handle = {
        let session_manager = Arc::clone(&session_manager);
        let ws_port = config.websocket_port;
        tokio::spawn(async move {
            if let Err(e) = websocket::start_server(session_manager, ws_port).await {
                error!("WebSocket server error: {}", e);
            }
        })
    };

    // Spawn gRPC server task
    let grpc_server_handle = {
        let session_manager = Arc::clone(&session_manager);
        let grpc_port = config.grpc_port;
        tokio::spawn(async move {
            if let Err(e) = grpc::start_server(session_manager, grpc_port).await {
                error!("gRPC server error: {}", e);
            }
        })
    };

    // Spawn WebTransport server task
    let wt_server_handle = {
        let session_manager = Arc::clone(&session_manager);
        let file_transfer = Arc::clone(&file_transfer);
        let wt_port = config.webtransport_port;
        tokio::spawn(async move {
            if let Err(e) = webtransport::start_server(session_manager, file_transfer, wt_port).await {
                error!("WebTransport server error: {}", e);
            }
        })
    };

    // Spawn cleanup task (runs every 60 seconds)
    let cleanup_handle = {
        let session_manager = Arc::clone(&session_manager);
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(60));
            loop {
                cleanup_interval.tick().await;
                session_manager.cleanup_dead_sessions().await;
            }
        })
    };

    // Wait for shutdown signal
    info!("Daemon running. Press Ctrl+C to stop.");
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal");
        }
        Err(e) => {
            error!("Failed to listen for shutdown signal: {}", e);
        }
    }

    // Graceful shutdown
    info!("Shutting down daemon...");

    // Signal IPC server to shutdown
    ipc_server.shutdown().await;

    // Wait for tasks to complete (with timeout)
    let shutdown_timeout = Duration::from_secs(5);
    tokio::select! {
        _ = ipc_server_handle => {
            info!("IPC server stopped");
        }
        _ = ws_server_handle => {
            info!("WebSocket server stopped");
        }
        _ = grpc_server_handle => {
            info!("gRPC server stopped");
        }
        _ = wt_server_handle => {
            info!("WebTransport server stopped");
        }
        _ = tokio::time::sleep(shutdown_timeout) => {
            warn!("Server shutdown timed out");
        }
    }

    // Abort cleanup task
    cleanup_handle.abort();

    // TODO: Save session state to database

    // Cleanup socket file
    if config.socket_path.exists() {
        std::fs::remove_file(&config.socket_path).ok();
    }

    info!("Pulsar Daemon stopped");
    Ok(())
}
