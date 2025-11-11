// Session persistence module for Orbit daemon
//
// This module provides:
// - Session lifecycle management
// - Terminal session persistence
// - Workspace management
// - Session snapshots for restore

pub mod database;
pub mod types;

use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub use database::{SessionDatabase, SessionSnapshot};
pub use types::{Session, SessionConfig, SessionStatus, SessionType, Workspace, WorkspaceLayout};

/// Session manager coordinates all active sessions and their persistence
pub struct SessionManager {
    db: SessionDatabase,
    active_sessions: Arc<RwLock<HashMap<String, Session>>>,
    active_workspaces: Arc<RwLock<HashMap<String, Workspace>>>,
}

impl SessionManager {
    /// Create a new session manager
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database for persistence
    pub async fn new(db_path: &str) -> Result<Self> {
        let db = SessionDatabase::new(db_path).await?;

        // Initialize schema
        db.initialize_schema().await?;

        // Load active sessions from database
        let sessions = db.list_active_sessions().await?;
        let mut active_sessions = HashMap::new();
        for session in sessions {
            active_sessions.insert(session.id.clone(), session);
        }

        // Load workspaces
        let workspaces = db.list_workspaces().await?;
        let mut active_workspaces = HashMap::new();
        for workspace in workspaces {
            active_workspaces.insert(workspace.id.clone(), workspace);
        }

        Ok(Self {
            db,
            active_sessions: Arc::new(RwLock::new(active_sessions)),
            active_workspaces: Arc::new(RwLock::new(active_workspaces)),
        })
    }

    /// Create a new session
    pub async fn create_session(&self, config: SessionConfig) -> Result<Session> {
        let session = Session {
            id: Uuid::new_v4().to_string(),
            session_type: config.session_type.clone(),
            created_at: Utc::now(),
            last_active: Utc::now(),
            status: SessionStatus::Active,
            config: config.clone(),
            workspace_id: config.workspace_id.clone(),
        };

        // Save to database
        self.db.save_session(&session).await?;

        // Add to active sessions
        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());

        tracing::info!(
            session_id = %session.id,
            session_type = ?session.session_type,
            "Created new session"
        );

        Ok(session)
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let sessions = self.active_sessions.read().await;
        if let Some(session) = sessions.get(id) {
            return Ok(Some(session.clone()));
        }

        // Try loading from database
        self.db.load_session(id).await
    }

    /// Update session status
    pub async fn update_session_status(
        &self,
        id: &str,
        status: SessionStatus,
    ) -> Result<()> {
        // Update in memory
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(session) = sessions.get_mut(id) {
                session.status = status.clone();
                session.last_active = Utc::now();
            }
        }

        // Update in database
        self.db.update_session_status(id, status.clone()).await?;

        tracing::debug!(session_id = %id, status = ?status, "Updated session status");
        Ok(())
    }

    /// Save a session snapshot (terminal buffer state)
    pub async fn save_snapshot(&self, session_id: &str, buffer: Vec<u8>) -> Result<()> {
        let buffer_size = buffer.len();
        self.db.save_snapshot(session_id, buffer).await?;

        // Update last active time
        {
            let mut sessions = self.active_sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.last_active = Utc::now();
            }
        }

        tracing::debug!(
            session_id = %session_id,
            buffer_size = buffer_size,
            "Saved session snapshot"
        );

        Ok(())
    }

    /// Load the latest snapshot for a session
    pub async fn load_latest_snapshot(&self, session_id: &str) -> Result<Option<Vec<u8>>> {
        self.db.load_latest_snapshot(session_id).await
    }

    /// List all active sessions
    pub async fn list_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.active_sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    /// Detach a session (mark as detached but keep alive)
    pub async fn detach_session(&self, id: &str) -> Result<()> {
        // Save snapshot before detaching
        if let Some(session) = self.get_session(id).await? {
            if session.status == SessionStatus::Active {
                // In a real implementation, we'd capture the terminal buffer here
                let buffer = vec![]; // Placeholder
                self.save_snapshot(id, buffer).await?;
            }
        }

        self.update_session_status(id, SessionStatus::Detached).await?;

        tracing::info!(session_id = %id, "Detached session");
        Ok(())
    }

    /// Terminate a session (mark as terminated and clean up)
    pub async fn terminate_session(&self, id: &str) -> Result<()> {
        self.update_session_status(id, SessionStatus::Terminated).await?;

        // Remove from active sessions
        let mut sessions = self.active_sessions.write().await;
        sessions.remove(id);

        tracing::info!(session_id = %id, "Terminated session");
        Ok(())
    }

    /// Delete a session and all its data
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        // Remove from active sessions
        {
            let mut sessions = self.active_sessions.write().await;
            sessions.remove(id);
        }

        // Delete from database
        self.db.delete_session(id).await?;

        tracing::info!(session_id = %id, "Deleted session");
        Ok(())
    }

    /// Create a new workspace
    pub async fn create_workspace(&self, name: String, layout: WorkspaceLayout) -> Result<Workspace> {
        let workspace = Workspace {
            id: Uuid::new_v4().to_string(),
            name,
            created_at: Utc::now(),
            layout,
            active_session_id: None,
        };

        // Save to database
        self.db.save_workspace(&workspace).await?;

        // Add to active workspaces
        let mut workspaces = self.active_workspaces.write().await;
        workspaces.insert(workspace.id.clone(), workspace.clone());

        tracing::info!(workspace_id = %workspace.id, "Created new workspace");

        Ok(workspace)
    }

    /// Get a workspace by ID
    pub async fn get_workspace(&self, id: &str) -> Result<Option<Workspace>> {
        let workspaces = self.active_workspaces.read().await;
        if let Some(workspace) = workspaces.get(id) {
            return Ok(Some(workspace.clone()));
        }

        // Try loading from database
        self.db.load_workspace(id).await
    }

    /// List all workspaces
    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let workspaces = self.active_workspaces.read().await;
        Ok(workspaces.values().cloned().collect())
    }

    /// Update workspace layout
    pub async fn update_workspace_layout(
        &self,
        id: &str,
        layout: WorkspaceLayout,
    ) -> Result<()> {
        // Update in memory
        {
            let mut workspaces = self.active_workspaces.write().await;
            if let Some(workspace) = workspaces.get_mut(id) {
                workspace.layout = layout.clone();
            }
        }

        // Update in database
        self.db.update_workspace_layout(id, layout).await?;

        tracing::debug!(workspace_id = %id, "Updated workspace layout");
        Ok(())
    }

    /// Delete a workspace
    pub async fn delete_workspace(&self, id: &str) -> Result<()> {
        // Remove from active workspaces
        {
            let mut workspaces = self.active_workspaces.write().await;
            workspaces.remove(id);
        }

        // Delete from database
        self.db.delete_workspace(id).await?;

        tracing::info!(workspace_id = %id, "Deleted workspace");
        Ok(())
    }

    /// Periodic maintenance - clean up old snapshots
    pub async fn cleanup_old_snapshots(&self, keep_last_n: usize) -> Result<()> {
        let sessions = self.list_active_sessions().await?;

        for session in sessions {
            self.db
                .cleanup_old_snapshots(&session.id, keep_last_n)
                .await?;
        }

        tracing::info!("Cleaned up old session snapshots");
        Ok(())
    }

    /// Auto-save all active sessions
    ///
    /// This should be called periodically (e.g., every 30 seconds)
    pub async fn auto_save_sessions(&self) -> Result<()> {
        let sessions = self.active_sessions.read().await;

        for (id, session) in sessions.iter() {
            if session.status == SessionStatus::Active {
                // In a real implementation, we'd capture actual terminal buffers
                // For now, just update the last_active timestamp
                self.db.update_session_last_active(id).await?;
            }
        }

        tracing::debug!(count = sessions.len(), "Auto-saved active sessions");
        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> SessionStats {
        let sessions = self.active_sessions.read().await;
        let workspaces = self.active_workspaces.read().await;

        let active_count = sessions
            .values()
            .filter(|s| s.status == SessionStatus::Active)
            .count();
        let detached_count = sessions
            .values()
            .filter(|s| s.status == SessionStatus::Detached)
            .count();

        SessionStats {
            total_sessions: sessions.len(),
            active_sessions: active_count,
            detached_sessions: detached_count,
            total_workspaces: workspaces.len(),
        }
    }
}

/// Session statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub detached_sessions: usize,
    pub total_workspaces: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let manager = SessionManager::new(db_path.to_str().unwrap()).await;
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_create_and_get_session() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = SessionManager::new(db_path.to_str().unwrap())
            .await
            .unwrap();

        let config = SessionConfig {
            session_type: SessionType::Local,
            host: None,
            port: None,
            username: None,
            workspace_id: None,
            command: Some("/bin/bash".to_string()),
        };

        let session = manager.create_session(config).await.unwrap();
        assert_eq!(session.status, SessionStatus::Active);

        let retrieved = manager.get_session(&session.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, session.id);
    }

    #[tokio::test]
    async fn test_session_status_updates() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = SessionManager::new(db_path.to_str().unwrap())
            .await
            .unwrap();

        let config = SessionConfig {
            session_type: SessionType::Local,
            host: None,
            port: None,
            username: None,
            workspace_id: None,
            command: Some("/bin/bash".to_string()),
        };

        let session = manager.create_session(config).await.unwrap();

        // Detach session
        manager.detach_session(&session.id).await.unwrap();
        let detached = manager.get_session(&session.id).await.unwrap().unwrap();
        assert_eq!(detached.status, SessionStatus::Detached);

        // Terminate session
        manager.terminate_session(&session.id).await.unwrap();
        let terminated = manager.get_session(&session.id).await.unwrap().unwrap();
        assert_eq!(terminated.status, SessionStatus::Terminated);
    }

    #[tokio::test]
    async fn test_workspace_management() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let manager = SessionManager::new(db_path.to_str().unwrap())
            .await
            .unwrap();

        let layout = WorkspaceLayout {
            layout_type: "grid".to_string(),
            config: serde_json::json!({"rows": 2, "cols": 2}),
        };

        let workspace = manager
            .create_workspace("Test Workspace".to_string(), layout)
            .await
            .unwrap();

        let retrieved = manager.get_workspace(&workspace.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Workspace");

        let all_workspaces = manager.list_workspaces().await.unwrap();
        assert_eq!(all_workspaces.len(), 1);
    }
}
