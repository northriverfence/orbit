//! Client for communicating with pulsar-daemon via IPC
//!
//! Provides high-level async API for daemon communication

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::sync::Mutex;
use uuid::Uuid;

/// Session state (matches daemon)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionState {
    Running,
    Detached,
    Stopped,
}

/// Session type (matches daemon)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Local,
    Ssh { host: String, port: u16 },
    Serial { device: String },
}

/// Session info (matches daemon)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub name: String,
    pub session_type: SessionType,
    pub created_at: String,  // ISO 8601 timestamp
    pub last_active: String,
    pub state: SessionState,
    pub num_clients: usize,
}

/// Daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    pub version: String,
    pub uptime_seconds: u64,
    pub num_sessions: usize,
    pub num_clients: usize,
}

/// Workspace layout structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub version: String,
    #[serde(rename = "type")]
    pub layout_type: String,
    pub panes: Vec<PaneConfig>,
    pub active_pane: Option<String>,
}

/// Pane configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    pub id: String,
    pub session_id: Option<String>,
    pub size: f32,
    pub direction: Option<String>,
    pub children: Option<Vec<PaneConfig>>,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
}

/// Workspace info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub layout: WorkspaceLayout,
    pub created_at: String,
    pub updated_at: String,
    pub is_template: bool,
    pub tags: Option<Vec<String>>,
}

/// Create workspace request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub layout: WorkspaceLayout,
    pub is_template: bool,
    pub tags: Option<Vec<String>>,
}

/// Update workspace request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateWorkspaceRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub layout: Option<WorkspaceLayout>,
    pub tags: Option<Vec<String>>,
}

/// Workspace filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceFilter {
    pub is_template: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
}

/// Workspace snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub layout: WorkspaceLayout,
    pub created_at: String,
}

/// IPC request (matches daemon protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Request {
    id: String,
    method: String,
    params: serde_json::Value,
}

/// IPC response (matches daemon protocol)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Response {
    id: String,
    #[serde(flatten)]
    result: ResponseResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ResponseResult {
    Success { result: serde_json::Value },
    Error { error: ErrorInfo },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorInfo {
    code: i32,
    message: String,
}

/// Client for communicating with pulsar-daemon
pub struct DaemonClient {
    socket_path: PathBuf,
    connection: Mutex<Option<Connection>>,
    request_id_counter: Mutex<u64>,
}

struct Connection {
    reader: BufReader<tokio::net::unix::OwnedReadHalf>,
    writer: tokio::net::unix::OwnedWriteHalf,
}

impl DaemonClient {
    /// Create a new daemon client
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            connection: Mutex::new(None),
            request_id_counter: Mutex::new(0),
        }
    }

    /// Connect to the daemon
    pub async fn connect(&self) -> Result<()> {
        let stream = UnixStream::connect(&self.socket_path)
            .await
            .with_context(|| format!("Failed to connect to daemon at {:?}", self.socket_path))?;

        let (reader, writer) = stream.into_split();
        let connection = Connection {
            reader: BufReader::new(reader),
            writer,
        };

        *self.connection.lock().await = Some(connection);
        Ok(())
    }

    /// Check if connected to daemon
    pub async fn is_connected(&self) -> bool {
        self.connection.lock().await.is_some()
    }

    /// Send request and receive response
    async fn send_request(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        // Get request ID
        let mut counter = self.request_id_counter.lock().await;
        *counter += 1;
        let request_id = counter.to_string();
        drop(counter);

        // Build request
        let request = Request {
            id: request_id.clone(),
            method: method.to_string(),
            params,
        };

        // Serialize request
        let mut request_json = serde_json::to_string(&request)?;
        request_json.push('\n');

        // Get connection
        let mut conn_guard = self.connection.lock().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow!("Not connected to daemon"))?;

        // Send request
        conn.writer.write_all(request_json.as_bytes()).await?;
        conn.writer.flush().await?;

        // Read response
        let mut response_line = String::new();
        conn.reader.read_line(&mut response_line).await?;

        // Parse response
        let response: Response = serde_json::from_str(&response_line)
            .with_context(|| format!("Failed to parse response: {}", response_line))?;

        // Check response ID matches
        if response.id != request_id {
            return Err(anyhow!(
                "Response ID mismatch: expected {}, got {}",
                request_id,
                response.id
            ));
        }

        // Check for errors
        match response.result {
            ResponseResult::Success { result } => Ok(result),
            ResponseResult::Error { error } => Err(anyhow!(
                "Daemon error {}: {}",
                error.code,
                error.message
            )),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        name: String,
        session_type: SessionType,
        cols: Option<u16>,
        rows: Option<u16>,
    ) -> Result<Uuid> {
        let mut params = serde_json::json!({
            "name": name,
            "type": session_type,
        });

        if let Some(cols) = cols {
            params["cols"] = serde_json::json!(cols);
        }
        if let Some(rows) = rows {
            params["rows"] = serde_json::json!(rows);
        }

        let result = self.send_request("create_session", params).await?;
        let session_id = result["session_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid session_id in response"))?;

        Uuid::parse_str(session_id).context("Failed to parse session UUID")
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let result = self.send_request("list_sessions", serde_json::json!({})).await?;
        let sessions: Vec<SessionInfo> = serde_json::from_value(result["sessions"].clone())
            .context("Failed to parse sessions")?;
        Ok(sessions)
    }

    /// Attach client to session
    pub async fn attach_session(&self, session_id: Uuid, client_id: Uuid) -> Result<()> {
        let params = serde_json::json!({
            "session_id": session_id,
            "client_id": client_id,
        });

        self.send_request("attach_session", params).await?;
        Ok(())
    }

    /// Detach client from session
    pub async fn detach_session(&self, session_id: Uuid, client_id: Uuid) -> Result<()> {
        let params = serde_json::json!({
            "session_id": session_id,
            "client_id": client_id,
        });

        self.send_request("detach_session", params).await?;
        Ok(())
    }

    /// Terminate a session
    pub async fn terminate_session(&self, session_id: Uuid) -> Result<()> {
        let params = serde_json::json!({
            "session_id": session_id,
        });

        self.send_request("terminate_session", params).await?;
        Ok(())
    }

    /// Resize terminal
    pub async fn resize_terminal(&self, session_id: Uuid, cols: u16, rows: u16) -> Result<()> {
        let params = serde_json::json!({
            "session_id": session_id,
            "cols": cols,
            "rows": rows,
        });

        self.send_request("resize_terminal", params).await?;
        Ok(())
    }

    /// Send input to session PTY (data should be base64-encoded)
    pub async fn send_input(&self, session_id: Uuid, data: String) -> Result<usize> {
        let params = serde_json::json!({
            "session_id": session_id,
            "data": data,
        });

        let result = self.send_request("send_input", params).await?;
        let bytes_written = result["bytes_written"]
            .as_u64()
            .ok_or_else(|| anyhow!("Invalid bytes_written in response"))?;

        Ok(bytes_written as usize)
    }

    /// Receive output from session PTY (returns base64-encoded data)
    pub async fn receive_output(&self, session_id: Uuid, timeout_ms: Option<u64>) -> Result<(String, usize)> {
        let mut params = serde_json::json!({
            "session_id": session_id,
        });

        if let Some(timeout) = timeout_ms {
            params["timeout_ms"] = serde_json::json!(timeout);
        }

        let result = self.send_request("receive_output", params).await?;
        let data = result["data"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid data in response"))?
            .to_string();
        let bytes_read = result["bytes_read"]
            .as_u64()
            .ok_or_else(|| anyhow!("Invalid bytes_read in response"))?;

        Ok((data, bytes_read as usize))
    }

    /// Get daemon status
    pub async fn get_status(&self) -> Result<DaemonStatus> {
        let result = self.send_request("get_status", serde_json::json!({})).await?;
        let status: DaemonStatus = serde_json::from_value(result)
            .context("Failed to parse daemon status")?;
        Ok(status)
    }

    // ============= Workspace Methods =============

    /// Create a new workspace
    pub async fn create_workspace(&self, request: CreateWorkspaceRequest) -> Result<Workspace> {
        let result = self.send_request("workspace_create", serde_json::to_value(&request)?).await?;
        let workspace: Workspace = serde_json::from_value(result)
            .context("Failed to parse workspace")?;
        Ok(workspace)
    }

    /// Get workspace by ID
    pub async fn get_workspace(&self, id: String) -> Result<Option<Workspace>> {
        let result = self.send_request("workspace_get", serde_json::json!({ "id": id })).await?;
        if result.is_null() {
            Ok(None)
        } else {
            let workspace: Workspace = serde_json::from_value(result)
                .context("Failed to parse workspace")?;
            Ok(Some(workspace))
        }
    }

    /// List workspaces with optional filter
    pub async fn list_workspaces(&self, filter: WorkspaceFilter) -> Result<Vec<Workspace>> {
        let result = self.send_request("workspace_list", serde_json::to_value(&filter)?).await?;
        let workspaces: Vec<Workspace> = serde_json::from_value(result)
            .context("Failed to parse workspaces")?;
        Ok(workspaces)
    }

    /// Update a workspace
    pub async fn update_workspace(&self, id: String, request: UpdateWorkspaceRequest) -> Result<Option<Workspace>> {
        let result = self.send_request("workspace_update", serde_json::json!({
            "id": id,
            "request": request
        })).await?;
        if result.is_null() {
            Ok(None)
        } else {
            let workspace: Workspace = serde_json::from_value(result)
                .context("Failed to parse workspace")?;
            Ok(Some(workspace))
        }
    }

    /// Delete a workspace
    pub async fn delete_workspace(&self, id: String) -> Result<bool> {
        let result = self.send_request("workspace_delete", serde_json::json!({ "id": id })).await?;
        let deleted: bool = serde_json::from_value(result)
            .context("Failed to parse delete result")?;
        Ok(deleted)
    }

    /// Save a workspace snapshot
    pub async fn save_workspace_snapshot(&self, workspace_id: String, name: String) -> Result<WorkspaceSnapshot> {
        let result = self.send_request("workspace_save_snapshot", serde_json::json!({
            "workspace_id": workspace_id,
            "name": name
        })).await?;
        let snapshot: WorkspaceSnapshot = serde_json::from_value(result)
            .context("Failed to parse snapshot")?;
        Ok(snapshot)
    }

    /// List workspace snapshots
    pub async fn list_workspace_snapshots(&self, workspace_id: String) -> Result<Vec<WorkspaceSnapshot>> {
        let result = self.send_request("workspace_list_snapshots", serde_json::json!({
            "workspace_id": workspace_id
        })).await?;
        let snapshots: Vec<WorkspaceSnapshot> = serde_json::from_value(result)
            .context("Failed to parse snapshots")?;
        Ok(snapshots)
    }

    /// Restore workspace from snapshot
    pub async fn restore_workspace_snapshot(&self, snapshot_id: String) -> Result<Option<Workspace>> {
        let result = self.send_request("workspace_restore_snapshot", serde_json::json!({
            "snapshot_id": snapshot_id
        })).await?;
        if result.is_null() {
            Ok(None)
        } else {
            let workspace: Workspace = serde_json::from_value(result)
                .context("Failed to parse workspace")?;
            Ok(Some(workspace))
        }
    }

    /// Disconnect from daemon
    pub async fn disconnect(&self) {
        *self.connection.lock().await = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DaemonClient::new(PathBuf::from("/tmp/test.sock"));
        assert!(!client.socket_path.as_os_str().is_empty());
    }

    #[tokio::test]
    async fn test_client_not_connected_initially() {
        let client = DaemonClient::new(PathBuf::from("/tmp/test.sock"));
        assert!(!client.is_connected().await);
    }
}
