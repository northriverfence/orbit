// Session types and data structures
//
// This module defines the core types for session management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session represents a terminal session (local or SSH)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub session_type: SessionType,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub status: SessionStatus,
    pub config: SessionConfig,
    pub workspace_id: Option<String>,
}

/// Type of session
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionType {
    /// Local shell session
    Local,
    /// SSH remote session
    Ssh,
}

/// Session status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    /// Session is currently active
    Active,
    /// Session is detached but still running
    Detached,
    /// Session has been terminated
    Terminated,
}

/// Configuration for creating a new session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub session_type: SessionType,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub workspace_id: Option<String>,
    pub command: Option<String>,
}

/// Workspace groups multiple sessions with a layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub layout: WorkspaceLayout,
    pub active_session_id: Option<String>,
}

/// Workspace layout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub layout_type: String, // "grid", "split-horizontal", "split-vertical", "custom"
    pub config: serde_json::Value, // Layout-specific configuration
}

/// Summary of a session (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub session_type: SessionType,
    pub status: SessionStatus,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub workspace_id: Option<String>,
}

impl From<Session> for SessionSummary {
    fn from(session: Session) -> Self {
        Self {
            id: session.id,
            session_type: session.session_type,
            status: session.status,
            created_at: session.created_at,
            last_active: session.last_active,
            workspace_id: session.workspace_id,
        }
    }
}

/// Snapshot metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub id: String,
    pub session_id: String,
    pub snapshot_at: DateTime<Utc>,
    pub buffer_size: usize,
}

/// Workspace summary (for listing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSummary {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub session_count: usize,
}

impl From<Workspace> for WorkspaceSummary {
    fn from(workspace: Workspace) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            created_at: workspace.created_at,
            session_count: 0, // Would need to query sessions to get count
        }
    }
}
