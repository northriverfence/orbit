// Session database persistence layer
//
// This module provides database operations for session persistence

use super::types::*;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite, Row};
use std::path::Path;
use uuid::Uuid;

/// Session snapshot data
#[derive(Debug, Clone)]
pub struct SessionSnapshot {
    pub id: String,
    pub session_id: String,
    pub snapshot_at: DateTime<Utc>,
    pub terminal_buffer: Vec<u8>,
    pub scrollback: Option<Vec<u8>>,
}

/// Database for session persistence
pub struct SessionDatabase {
    pool: Pool<Sqlite>,
}

impl SessionDatabase {
    /// Create a new session database connection
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .pragma("cache_size", "10000")
            .pragma("foreign_keys", "ON")
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect_with(options)
            .await
            .context("Failed to create session database pool")?;

        Ok(Self { pool })
    }

    /// Initialize database schema
    pub async fn initialize_schema(&self) -> Result<()> {
        // Create sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                session_type TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                last_active INTEGER NOT NULL,
                status TEXT NOT NULL,
                config TEXT NOT NULL,
                workspace_id TEXT,
                FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE SET NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create session_snapshots table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS session_snapshots (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                snapshot_at INTEGER NOT NULL,
                terminal_buffer BLOB NOT NULL,
                scrollback BLOB,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create workspaces table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS workspaces (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                layout TEXT NOT NULL,
                active_session_id TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_sessions_workspace_id ON sessions(workspace_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_session_id ON session_snapshots(session_id)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_snapshots_snapshot_at ON session_snapshots(snapshot_at DESC)",
        )
        .execute(&self.pool)
        .await?;

        tracing::info!("Session database schema initialized");
        Ok(())
    }

    /// Save a session
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        let config_json = serde_json::to_string(&session.config)?;

        sqlx::query(
            r#"
            INSERT INTO sessions (id, session_type, created_at, last_active, status, config, workspace_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                last_active = excluded.last_active,
                status = excluded.status,
                config = excluded.config,
                workspace_id = excluded.workspace_id
            "#,
        )
        .bind(&session.id)
        .bind(session.session_type.to_string())
        .bind(session.created_at.timestamp())
        .bind(session.last_active.timestamp())
        .bind(session.status.to_string())
        .bind(&config_json)
        .bind(&session.workspace_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load a session by ID
    pub async fn load_session(&self, id: &str) -> Result<Option<Session>> {
        let row = sqlx::query(
            "SELECT id, session_type, created_at, last_active, status, config, workspace_id FROM sessions WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let session = Self::row_to_session(row)?;
            Ok(Some(session))
        } else {
            Ok(None)
        }
    }

    /// List all active sessions
    pub async fn list_active_sessions(&self) -> Result<Vec<Session>> {
        let rows = sqlx::query(
            "SELECT id, session_type, created_at, last_active, status, config, workspace_id FROM sessions WHERE status IN ('active', 'detached')"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut sessions = Vec::new();
        for row in rows {
            sessions.push(Self::row_to_session(row)?);
        }

        Ok(sessions)
    }

    /// Update session status
    pub async fn update_session_status(&self, id: &str, status: SessionStatus) -> Result<()> {
        sqlx::query("UPDATE sessions SET status = ?, last_active = ? WHERE id = ?")
            .bind(status.to_string())
            .bind(Utc::now().timestamp())
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update session last_active timestamp
    pub async fn update_session_last_active(&self, id: &str) -> Result<()> {
        sqlx::query("UPDATE sessions SET last_active = ? WHERE id = ?")
            .bind(Utc::now().timestamp())
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete a session
    pub async fn delete_session(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Save a session snapshot
    pub async fn save_snapshot(&self, session_id: &str, buffer: Vec<u8>) -> Result<()> {
        let id = Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT INTO session_snapshots (id, session_id, snapshot_at, terminal_buffer, scrollback) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&id)
        .bind(session_id)
        .bind(Utc::now().timestamp())
        .bind(&buffer)
        .bind(Option::<Vec<u8>>::None) // scrollback can be null for now
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load the latest snapshot for a session
    pub async fn load_latest_snapshot(&self, session_id: &str) -> Result<Option<Vec<u8>>> {
        let row = sqlx::query(
            "SELECT terminal_buffer FROM session_snapshots WHERE session_id = ? ORDER BY snapshot_at DESC LIMIT 1"
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let buffer: Vec<u8> = row.try_get("terminal_buffer")?;
            Ok(Some(buffer))
        } else {
            Ok(None)
        }
    }

    /// List snapshots for a session
    pub async fn list_snapshots(&self, session_id: &str) -> Result<Vec<SnapshotInfo>> {
        let rows = sqlx::query(
            "SELECT id, session_id, snapshot_at, length(terminal_buffer) as buffer_size FROM session_snapshots WHERE session_id = ? ORDER BY snapshot_at DESC"
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        let mut snapshots = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            let session_id: String = row.try_get("session_id")?;
            let snapshot_at: i64 = row.try_get("snapshot_at")?;
            let buffer_size: i64 = row.try_get("buffer_size")?;

            snapshots.push(SnapshotInfo {
                id,
                session_id,
                snapshot_at: DateTime::from_timestamp(snapshot_at, 0).unwrap_or(Utc::now()),
                buffer_size: buffer_size as usize,
            });
        }

        Ok(snapshots)
    }

    /// Clean up old snapshots, keeping only the last N
    pub async fn cleanup_old_snapshots(&self, session_id: &str, keep_last_n: usize) -> Result<()> {
        // Get snapshot IDs to delete
        let rows = sqlx::query(
            "SELECT id FROM session_snapshots WHERE session_id = ? ORDER BY snapshot_at DESC"
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        if rows.len() > keep_last_n {
            let to_delete = &rows[keep_last_n..];
            for row in to_delete {
                let id: String = row.try_get("id")?;
                sqlx::query("DELETE FROM session_snapshots WHERE id = ?")
                    .bind(&id)
                    .execute(&self.pool)
                    .await?;
            }

            tracing::debug!(
                session_id = %session_id,
                deleted = to_delete.len(),
                "Cleaned up old snapshots"
            );
        }

        Ok(())
    }

    /// Save a workspace
    pub async fn save_workspace(&self, workspace: &Workspace) -> Result<()> {
        let layout_json = serde_json::to_string(&workspace.layout)?;

        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, created_at, layout, active_session_id)
            VALUES (?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                layout = excluded.layout,
                active_session_id = excluded.active_session_id
            "#,
        )
        .bind(&workspace.id)
        .bind(&workspace.name)
        .bind(workspace.created_at.timestamp())
        .bind(&layout_json)
        .bind(&workspace.active_session_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Load a workspace by ID
    pub async fn load_workspace(&self, id: &str) -> Result<Option<Workspace>> {
        let row = sqlx::query(
            "SELECT id, name, created_at, layout, active_session_id FROM workspaces WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let workspace = Self::row_to_workspace(row)?;
            Ok(Some(workspace))
        } else {
            Ok(None)
        }
    }

    /// List all workspaces
    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        let rows = sqlx::query(
            "SELECT id, name, created_at, layout, active_session_id FROM workspaces"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut workspaces = Vec::new();
        for row in rows {
            workspaces.push(Self::row_to_workspace(row)?);
        }

        Ok(workspaces)
    }

    /// Update workspace layout
    pub async fn update_workspace_layout(
        &self,
        id: &str,
        layout: WorkspaceLayout,
    ) -> Result<()> {
        let layout_json = serde_json::to_string(&layout)?;

        sqlx::query("UPDATE workspaces SET layout = ? WHERE id = ?")
            .bind(&layout_json)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Delete a workspace
    pub async fn delete_workspace(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // Helper functions

    fn row_to_session(row: sqlx::sqlite::SqliteRow) -> Result<Session> {
        let id: String = row.try_get("id")?;
        let session_type_str: String = row.try_get("session_type")?;
        let created_at: i64 = row.try_get("created_at")?;
        let last_active: i64 = row.try_get("last_active")?;
        let status_str: String = row.try_get("status")?;
        let config_json: String = row.try_get("config")?;
        let workspace_id: Option<String> = row.try_get("workspace_id")?;

        let session_type = match session_type_str.as_str() {
            "local" => SessionType::Local,
            "ssh" => SessionType::Ssh,
            _ => SessionType::Local,
        };

        let status = match status_str.as_str() {
            "active" => SessionStatus::Active,
            "detached" => SessionStatus::Detached,
            "terminated" => SessionStatus::Terminated,
            _ => SessionStatus::Active,
        };

        let config: SessionConfig = serde_json::from_str(&config_json)?;

        Ok(Session {
            id,
            session_type,
            created_at: DateTime::from_timestamp(created_at, 0).unwrap_or(Utc::now()),
            last_active: DateTime::from_timestamp(last_active, 0).unwrap_or(Utc::now()),
            status,
            config,
            workspace_id,
        })
    }

    fn row_to_workspace(row: sqlx::sqlite::SqliteRow) -> Result<Workspace> {
        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let created_at: i64 = row.try_get("created_at")?;
        let layout_json: String = row.try_get("layout")?;
        let active_session_id: Option<String> = row.try_get("active_session_id")?;

        let layout: WorkspaceLayout = serde_json::from_str(&layout_json)?;

        Ok(Workspace {
            id,
            name,
            created_at: DateTime::from_timestamp(created_at, 0).unwrap_or(Utc::now()),
            layout,
            active_session_id,
        })
    }
}

// Helper trait implementations for string serialization
impl ToString for SessionType {
    fn to_string(&self) -> String {
        match self {
            SessionType::Local => "local".to_string(),
            SessionType::Ssh => "ssh".to_string(),
        }
    }
}

impl ToString for SessionStatus {
    fn to_string(&self) -> String {
        match self {
            SessionStatus::Active => "active".to_string(),
            SessionStatus::Detached => "detached".to_string(),
            SessionStatus::Terminated => "terminated".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_initialization() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = SessionDatabase::new(&db_path).await.unwrap();
        db.initialize_schema().await.unwrap();
    }

    #[tokio::test]
    async fn test_session_persistence() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = SessionDatabase::new(&db_path).await.unwrap();
        db.initialize_schema().await.unwrap();

        let session = Session {
            id: Uuid::new_v4().to_string(),
            session_type: SessionType::Local,
            created_at: Utc::now(),
            last_active: Utc::now(),
            status: SessionStatus::Active,
            config: SessionConfig {
                session_type: SessionType::Local,
                host: None,
                port: None,
                username: None,
                workspace_id: None,
                command: Some("/bin/bash".to_string()),
            },
            workspace_id: None,
        };

        // Save session
        db.save_session(&session).await.unwrap();

        // Load session
        let loaded = db.load_session(&session.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().id, session.id);

        // Update status
        db.update_session_status(&session.id, SessionStatus::Detached)
            .await
            .unwrap();

        let updated = db.load_session(&session.id).await.unwrap().unwrap();
        assert_eq!(updated.status, SessionStatus::Detached);

        // Delete session
        db.delete_session(&session.id).await.unwrap();
        let deleted = db.load_session(&session.id).await.unwrap();
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_snapshot_persistence() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = SessionDatabase::new(&db_path).await.unwrap();
        db.initialize_schema().await.unwrap();

        let session = Session {
            id: Uuid::new_v4().to_string(),
            session_type: SessionType::Local,
            created_at: Utc::now(),
            last_active: Utc::now(),
            status: SessionStatus::Active,
            config: SessionConfig {
                session_type: SessionType::Local,
                host: None,
                port: None,
                username: None,
                workspace_id: None,
                command: Some("/bin/bash".to_string()),
            },
            workspace_id: None,
        };

        db.save_session(&session).await.unwrap();

        // Save snapshot
        let buffer = vec![1, 2, 3, 4, 5];
        db.save_snapshot(&session.id, buffer.clone()).await.unwrap();

        // Load snapshot
        let loaded = db.load_latest_snapshot(&session.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap(), buffer);

        // List snapshots
        let snapshots = db.list_snapshots(&session.id).await.unwrap();
        assert_eq!(snapshots.len(), 1);
    }

    #[tokio::test]
    async fn test_workspace_persistence() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let db = SessionDatabase::new(&db_path).await.unwrap();
        db.initialize_schema().await.unwrap();

        let workspace = Workspace {
            id: Uuid::new_v4().to_string(),
            name: "Test Workspace".to_string(),
            created_at: Utc::now(),
            layout: WorkspaceLayout {
                layout_type: "grid".to_string(),
                config: serde_json::json!({"rows": 2, "cols": 2}),
            },
            active_session_id: None,
        };

        // Save workspace
        db.save_workspace(&workspace).await.unwrap();

        // Load workspace
        let loaded = db.load_workspace(&workspace.id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().name, workspace.name);

        // Update layout
        let new_layout = WorkspaceLayout {
            layout_type: "split-horizontal".to_string(),
            config: serde_json::json!({"ratio": 0.5}),
        };
        db.update_workspace_layout(&workspace.id, new_layout.clone())
            .await
            .unwrap();

        let updated = db.load_workspace(&workspace.id).await.unwrap().unwrap();
        assert_eq!(updated.layout.layout_type, "split-horizontal");

        // Delete workspace
        db.delete_workspace(&workspace.id).await.unwrap();
        let deleted = db.load_workspace(&workspace.id).await.unwrap();
        assert!(deleted.is_none());
    }
}
