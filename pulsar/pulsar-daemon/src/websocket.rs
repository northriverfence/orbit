//! WebSocket server for real-time PTY output streaming
//!
//! Provides event-driven output streaming instead of polling

use anyhow::{Context, Result};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use base64::Engine;
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::session_manager::SessionManager;

/// WebSocket server state
#[derive(Clone)]
pub struct WsState {
    pub session_manager: Arc<SessionManager>,
}

/// Create WebSocket router
pub fn create_router(session_manager: Arc<SessionManager>) -> Router {
    let state = WsState { session_manager };

    Router::new()
        .route("/ws/:session_id", get(ws_handler))
        .with_state(state)
}

/// WebSocket upgrade handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<String>,
    State(state): State<WsState>,
) -> impl IntoResponse {
    // Parse session ID
    let session_uuid = match Uuid::parse_str(&session_id) {
        Ok(uuid) => uuid,
        Err(e) => {
            error!("Invalid session ID: {}", e);
            return ws.on_upgrade(|socket| async move {
                let _ = handle_invalid_session(socket).await;
            });
        }
    };

    // Verify session exists
    match state.session_manager.get_session(session_uuid).await {
        Ok(_session) => {
            info!("WebSocket connection established for session: {}", session_uuid);
            ws.on_upgrade(move |socket| {
                handle_socket(socket, session_uuid, state.session_manager)
            })
        }
        Err(e) => {
            error!("Session not found: {}", e);
            ws.on_upgrade(|socket| async move {
                let _ = handle_session_not_found(socket).await;
            })
        }
    }
}

/// Handle invalid session error
async fn handle_invalid_session(mut socket: WebSocket) -> Result<()> {
    socket
        .send(Message::Text("Error: Invalid session ID".to_string()))
        .await?;
    socket.close().await?;
    Ok(())
}

/// Handle session not found error
async fn handle_session_not_found(mut socket: WebSocket) -> Result<()> {
    socket
        .send(Message::Text("Error: Session not found".to_string()))
        .await?;
    socket.close().await?;
    Ok(())
}

/// Handle WebSocket connection
async fn handle_socket(
    socket: WebSocket,
    session_id: Uuid,
    session_manager: Arc<SessionManager>,
) {
    let (mut sender, mut receiver) = socket.split();

    // Get session
    let session = match session_manager.get_session(session_id).await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get session: {}", e);
            return;
        }
    };

    // Subscribe to output broadcast
    let mut output_rx = session.output_broadcast.subscribe();

    // Spawn task to forward PTY output to WebSocket
    let output_task = tokio::spawn(async move {
        while let Ok(data) = output_rx.recv().await {
            // Encode as base64 for binary safety
            let base64_data = base64::engine::general_purpose::STANDARD.encode(&data);

            // Send as text message
            if let Err(e) = sender.send(Message::Text(base64_data)).await {
                debug!("WebSocket send error: {}", e);
                break;
            }
        }
        debug!("Output streaming task ended for session: {}", session_id);
    });

    // Handle incoming messages (input from client)
    let input_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Decode base64 input
                    let data = match base64::engine::general_purpose::STANDARD.decode(&text) {
                        Ok(d) => d,
                        Err(e) => {
                            warn!("Invalid base64 input: {}", e);
                            continue;
                        }
                    };

                    // Write to PTY
                    let mut terminal = session.terminal_session.write().await;
                    if let Err(e) = terminal.write(&data) {
                        error!("Failed to write to PTY: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    // Direct binary input
                    let mut terminal = session.terminal_session.write().await;
                    if let Err(e) = terminal.write(&data) {
                        error!("Failed to write to PTY: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    debug!("WebSocket closed by client");
                    break;
                }
                Ok(Message::Ping(_)) | Ok(Message::Pong(_)) => {
                    // Handled automatically by axum
                }
                Err(e) => {
                    debug!("WebSocket error: {}", e);
                    break;
                }
            }
        }
        debug!("Input handling task ended for session: {}", session_id);
    });

    // Wait for either task to complete
    tokio::select! {
        _ = output_task => {},
        _ = input_task => {},
    }

    info!("WebSocket connection closed for session: {}", session_id);
}

/// Start WebSocket server
pub async fn start_server(
    session_manager: Arc<SessionManager>,
    port: u16,
) -> Result<()> {
    let app = create_router(session_manager);

    let addr = format!("127.0.0.1:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("Failed to bind WebSocket server to {}", addr))?;

    info!("WebSocket server listening on {}", addr);

    axum::serve(listener, app)
        .await
        .context("WebSocket server error")?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_state_creation() {
        let session_manager = Arc::new(SessionManager::new());
        let state = WsState {
            session_manager: Arc::clone(&session_manager),
        };
        assert!(Arc::strong_count(&state.session_manager) > 0);
    }
}
