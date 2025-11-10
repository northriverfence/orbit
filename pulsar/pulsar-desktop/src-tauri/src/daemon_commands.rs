//! Tauri commands for daemon interaction

use crate::daemon_client::{
    CreateWorkspaceRequest, DaemonClient, SessionInfo, SessionType, UpdateWorkspaceRequest,
    Workspace, WorkspaceFilter, WorkspaceSnapshot,
};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

/// Create a new local terminal session via daemon
#[tauri::command]
pub async fn daemon_create_local_session(
    name: String,
    cols: u16,
    rows: u16,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<String, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    // Create session
    let session_id = daemon
        .create_session(name, SessionType::Local, Some(cols), Some(rows))
        .await
        .map_err(|e| format!("Failed to create session: {}", e))?;

    Ok(session_id.to_string())
}

/// Create a new SSH session via daemon
#[tauri::command]
pub async fn daemon_create_ssh_session(
    name: String,
    host: String,
    port: u16,
    cols: u16,
    rows: u16,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<String, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    // Create session
    let session_id = daemon
        .create_session(
            name,
            SessionType::Ssh { host, port },
            Some(cols),
            Some(rows),
        )
        .await
        .map_err(|e| format!("Failed to create session: {}", e))?;

    Ok(session_id.to_string())
}

/// List all sessions from daemon
#[tauri::command]
pub async fn daemon_list_sessions(
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Vec<SessionInfo>, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .list_sessions()
        .await
        .map_err(|e| format!("Failed to list sessions: {}", e))
}

/// Attach to a session
#[tauri::command]
pub async fn daemon_attach_session(
    session_id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<(), String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let client_id = Uuid::new_v4(); // Generate unique client ID

    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .attach_session(session_uuid, client_id)
        .await
        .map_err(|e| format!("Failed to attach to session: {}", e))
}

/// Detach from a session
#[tauri::command]
pub async fn daemon_detach_session(
    session_id: String,
    client_id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<(), String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    let client_uuid = Uuid::parse_str(&client_id)
        .map_err(|e| format!("Invalid client ID: {}", e))?;

    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .detach_session(session_uuid, client_uuid)
        .await
        .map_err(|e| format!("Failed to detach from session: {}", e))
}

/// Terminate a session
#[tauri::command]
pub async fn daemon_terminate_session(
    session_id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<(), String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .terminate_session(session_uuid)
        .await
        .map_err(|e| format!("Failed to terminate session: {}", e))
}

/// Resize terminal in session
#[tauri::command]
pub async fn daemon_resize_terminal(
    session_id: String,
    cols: u16,
    rows: u16,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<(), String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .resize_terminal(session_uuid, cols, rows)
        .await
        .map_err(|e| format!("Failed to resize terminal: {}", e))
}

/// Get daemon status
#[tauri::command]
pub async fn daemon_get_status(
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<serde_json::Value, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    let status = daemon
        .get_status()
        .await
        .map_err(|e| format!("Failed to get status: {}", e))?;

    Ok(serde_json::json!({
        "version": status.version,
        "uptime_seconds": status.uptime_seconds,
        "num_sessions": status.num_sessions,
        "num_clients": status.num_clients,
    }))
}

/// Send input to session PTY
#[tauri::command]
pub async fn daemon_send_input(
    session_id: String,
    data: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<usize, String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .send_input(session_uuid, data)
        .await
        .map_err(|e| format!("Failed to send input: {}", e))
}

/// Receive output from session PTY
#[tauri::command]
pub async fn daemon_receive_output(
    session_id: String,
    timeout_ms: Option<u64>,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<serde_json::Value, String> {
    let session_uuid = Uuid::parse_str(&session_id)
        .map_err(|e| format!("Invalid session ID: {}", e))?;

    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    let (data, bytes_read) = daemon
        .receive_output(session_uuid, timeout_ms)
        .await
        .map_err(|e| format!("Failed to receive output: {}", e))?;

    Ok(serde_json::json!({
        "data": data,
        "bytes_read": bytes_read,
    }))
}

/// Check if daemon is reachable
#[tauri::command]
pub async fn daemon_check_connection(
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<bool, String> {
    if daemon.is_connected().await {
        return Ok(true);
    }

    // Try to connect
    match daemon.connect().await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

// ============= Workspace Commands =============

/// Create a new workspace
#[tauri::command]
pub async fn workspace_create(
    request: CreateWorkspaceRequest,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Workspace, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .create_workspace(request)
        .await
        .map_err(|e| format!("Failed to create workspace: {}", e))
}

/// Get workspace by ID
#[tauri::command]
pub async fn workspace_get(
    id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Option<Workspace>, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .get_workspace(id)
        .await
        .map_err(|e| format!("Failed to get workspace: {}", e))
}

/// List workspaces with optional filter
#[tauri::command]
pub async fn workspace_list(
    filter: Option<WorkspaceFilter>,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Vec<Workspace>, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    let filter = filter.unwrap_or(WorkspaceFilter {
        is_template: None,
        tags: None,
        search: None,
    });

    daemon
        .list_workspaces(filter)
        .await
        .map_err(|e| format!("Failed to list workspaces: {}", e))
}

/// Update a workspace
#[tauri::command]
pub async fn workspace_update(
    id: String,
    request: UpdateWorkspaceRequest,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Option<Workspace>, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .update_workspace(id, request)
        .await
        .map_err(|e| format!("Failed to update workspace: {}", e))
}

/// Delete a workspace
#[tauri::command]
pub async fn workspace_delete(
    id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<bool, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .delete_workspace(id)
        .await
        .map_err(|e| format!("Failed to delete workspace: {}", e))
}

/// Save a workspace snapshot
#[tauri::command]
pub async fn workspace_save_snapshot(
    workspace_id: String,
    name: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<WorkspaceSnapshot, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .save_workspace_snapshot(workspace_id, name)
        .await
        .map_err(|e| format!("Failed to save snapshot: {}", e))
}

/// List workspace snapshots
#[tauri::command]
pub async fn workspace_list_snapshots(
    workspace_id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Vec<WorkspaceSnapshot>, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .list_workspace_snapshots(workspace_id)
        .await
        .map_err(|e| format!("Failed to list snapshots: {}", e))
}

/// Restore workspace from snapshot
#[tauri::command]
pub async fn workspace_restore_snapshot(
    snapshot_id: String,
    daemon: State<'_, Arc<DaemonClient>>,
) -> Result<Option<Workspace>, String> {
    // Ensure connected
    if !daemon.is_connected().await {
        daemon
            .connect()
            .await
            .map_err(|e| format!("Failed to connect to daemon: {}", e))?;
    }

    daemon
        .restore_workspace_snapshot(snapshot_id)
        .await
        .map_err(|e| format!("Failed to restore snapshot: {}", e))
}
