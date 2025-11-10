//! Workspace Models
//!
//! Data structures for workspace management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Workspace represents a collection of terminal sessions with a specific layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub layout: WorkspaceLayout,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_template: bool,
    pub tags: Option<Vec<String>>,
}

/// Workspace layout structure (mirrors frontend split-pane layout)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLayout {
    pub version: String,
    #[serde(rename = "type")]
    pub layout_type: String, // "single" | "split"
    pub panes: Vec<PaneConfig>,
    pub active_pane: Option<String>,
}

/// Pane configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneConfig {
    pub id: String,
    pub session_id: Option<String>,
    pub size: f32, // percentage (0-100)
    pub direction: Option<String>, // "horizontal" | "vertical"
    pub children: Option<Vec<PaneConfig>>,
    pub min_size: Option<u32>,
    pub max_size: Option<u32>,
}

/// Workspace session mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSession {
    pub workspace_id: String,
    pub session_id: String,
    pub pane_id: String,
    pub position: i32,
    pub session_config: Option<SessionConfig>,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(rename = "type")]
    pub session_type: String, // "local" | "ssh"
    pub name: String,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub username: Option<String>,
}

/// Workspace snapshot (version)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceSnapshot {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub layout: WorkspaceLayout,
    pub created_at: DateTime<Utc>,
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
#[derive(Debug, Clone, Default)]
pub struct WorkspaceFilter {
    pub is_template: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
}

// Helper functions for manual row parsing

impl Workspace {
    /// Create a new workspace with default layout
    pub fn new(name: String) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id,
            name,
            description: None,
            icon: None,
            layout: WorkspaceLayout::default(),
            created_at: now,
            updated_at: now,
            is_template: false,
            tags: None,
        }
    }

    /// Create from request
    pub fn from_request(req: CreateWorkspaceRequest) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        Self {
            id,
            name: req.name,
            description: req.description,
            icon: req.icon,
            layout: req.layout,
            created_at: now,
            updated_at: now,
            is_template: req.is_template,
            tags: req.tags,
        }
    }
}

impl Default for WorkspaceLayout {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            layout_type: "single".to_string(),
            panes: vec![PaneConfig::default()],
            active_pane: None,
        }
    }
}

impl Default for PaneConfig {
    fn default() -> Self {
        Self {
            id: format!("pane-{}", uuid::Uuid::new_v4()),
            session_id: None,
            size: 100.0,
            direction: None,
            children: None,
            min_size: Some(100),
            max_size: None,
        }
    }
}

impl WorkspaceSnapshot {
    /// Create snapshot from workspace
    pub fn from_workspace(workspace: &Workspace, name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            workspace_id: workspace.id.clone(),
            name,
            layout: workspace.layout.clone(),
            created_at: Utc::now(),
        }
    }
}
