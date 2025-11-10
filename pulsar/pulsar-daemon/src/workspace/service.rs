//! Workspace Service
//!
//! Provides CRUD operations for workspaces

use super::models::*;
use anyhow::{Context, Result};
use chrono::{DateTime, TimeZone, Utc};
use sqlx::{Pool, Row, Sqlite};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Workspace service for managing workspace CRUD operations
pub struct WorkspaceService {
    db: Arc<Pool<Sqlite>>,
}

impl WorkspaceService {
    /// Create a new workspace service
    pub fn new(db: Arc<Pool<Sqlite>>) -> Self {
        Self { db }
    }

    /// Initialize database (run migrations)
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing workspace database");

        // Read migration file
        let migration_sql = include_str!("../../migrations/003_workspaces.sql");

        // Execute migration
        sqlx::raw_sql(migration_sql)
            .execute(&*self.db)
            .await
            .context("Failed to run workspace migrations")?;

        info!("Workspace database initialized");
        Ok(())
    }

    /// Create a new workspace
    pub async fn create_workspace(&self, req: CreateWorkspaceRequest) -> Result<Workspace> {
        let workspace = Workspace::from_request(req);

        let layout_json = serde_json::to_string(&workspace.layout)
            .context("Failed to serialize workspace layout")?;

        let tags_json = workspace
            .tags
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()
            .context("Failed to serialize tags")?;

        sqlx::query(
            r#"
            INSERT INTO workspaces (id, name, description, icon, layout, created_at, updated_at, is_template, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&workspace.id)
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(&workspace.icon)
        .bind(&layout_json)
        .bind(workspace.created_at.timestamp())
        .bind(workspace.updated_at.timestamp())
        .bind(workspace.is_template)
        .bind(tags_json)
        .execute(&*self.db)
        .await
        .context("Failed to insert workspace")?;

        info!("Created workspace: {} ({})", workspace.name, workspace.id);
        Ok(workspace)
    }

    /// Get a workspace by ID
    pub async fn get_workspace(&self, id: &str) -> Result<Option<Workspace>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, icon, layout, created_at, updated_at, is_template, tags
            FROM workspaces
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&*self.db)
        .await
        .context("Failed to fetch workspace")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let layout_json: String = row.get("layout");
        let layout: WorkspaceLayout = serde_json::from_str(&layout_json)?;

        let tags_json: Option<String> = row.get("tags");
        let tags: Option<Vec<String>> = tags_json
            .as_ref()
            .map(|j| serde_json::from_str(j.as_str()))
            .transpose()?;

        let created_at_ts: i64 = row.get("created_at");
        let updated_at_ts: i64 = row.get("updated_at");

        Ok(Some(Workspace {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            icon: row.get("icon"),
            layout,
            created_at: Utc.timestamp_opt(created_at_ts, 0).unwrap(),
            updated_at: Utc.timestamp_opt(updated_at_ts, 0).unwrap(),
            is_template: row.get("is_template"),
            tags,
        }))
    }

    /// List all workspaces with optional filtering
    pub async fn list_workspaces(&self, filter: WorkspaceFilter) -> Result<Vec<Workspace>> {
        let mut query = String::from(
            "SELECT id, name, description, icon, layout, created_at, updated_at, is_template, tags FROM workspaces WHERE 1=1",
        );
        let mut params: Vec<String> = Vec::new();

        // Apply filters
        if let Some(is_template) = filter.is_template {
            query.push_str(" AND is_template = ?");
            params.push(if is_template { "1" } else { "0" }.to_string());
        }

        if let Some(search) = filter.search {
            query.push_str(" AND (name LIKE ? OR description LIKE ?)");
            let search_pattern = format!("%{}%", search);
            params.push(search_pattern.clone());
            params.push(search_pattern);
        }

        query.push_str(" ORDER BY updated_at DESC");

        // Build and execute query
        let mut sql_query = sqlx::query(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query
            .fetch_all(&*self.db)
            .await
            .context("Failed to list workspaces")?;

        let mut workspaces = Vec::new();
        for row in rows {
            let layout_json: String = row.get("layout");
            let layout: WorkspaceLayout = serde_json::from_str(&layout_json)?;

            let tags_json: Option<String> = row.get("tags");
            let tags: Option<Vec<String>> = tags_json
                .as_ref()
                .map(|j| serde_json::from_str(j.as_str()))
                .transpose()?;

            let created_at_ts: i64 = row.get("created_at");
            let updated_at_ts: i64 = row.get("updated_at");

            workspaces.push(Workspace {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                icon: row.get("icon"),
                layout,
                created_at: Utc.timestamp_opt(created_at_ts, 0).unwrap(),
                updated_at: Utc.timestamp_opt(updated_at_ts, 0).unwrap(),
                is_template: row.get("is_template"),
                tags,
            });
        }

        debug!("Listed {} workspaces", workspaces.len());
        Ok(workspaces)
    }

    /// Update a workspace
    pub async fn update_workspace(
        &self,
        id: &str,
        req: UpdateWorkspaceRequest,
    ) -> Result<Option<Workspace>> {
        // Fetch existing workspace
        let Some(mut workspace) = self.get_workspace(id).await? else {
            return Ok(None);
        };

        // Apply updates
        if let Some(name) = req.name {
            workspace.name = name;
        }

        if let Some(description) = req.description {
            workspace.description = Some(description);
        }

        if let Some(icon) = req.icon {
            workspace.icon = Some(icon);
        }

        if let Some(layout) = req.layout {
            workspace.layout = layout;
        }

        if let Some(tags) = req.tags {
            workspace.tags = Some(tags);
        }

        // Serialize
        let layout_json = serde_json::to_string(&workspace.layout)
            .context("Failed to serialize workspace layout")?;

        let tags_json = workspace
            .tags
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()
            .context("Failed to serialize tags")?;

        // Update in database
        sqlx::query(
            r#"
            UPDATE workspaces
            SET name = ?, description = ?, icon = ?, layout = ?, tags = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&workspace.name)
        .bind(&workspace.description)
        .bind(&workspace.icon)
        .bind(&layout_json)
        .bind(tags_json)
        .bind(Utc::now().timestamp())
        .bind(id)
        .execute(&*self.db)
        .await
        .context("Failed to update workspace")?;

        info!("Updated workspace: {} ({})", workspace.name, id);
        Ok(Some(workspace))
    }

    /// Delete a workspace
    pub async fn delete_workspace(&self, id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM workspaces WHERE id = ?")
            .bind(id)
            .execute(&*self.db)
            .await
            .context("Failed to delete workspace")?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted workspace: {}", id);
        } else {
            warn!("Workspace not found for deletion: {}", id);
        }

        Ok(deleted)
    }

    /// Save a snapshot of a workspace
    pub async fn save_snapshot(&self, workspace_id: &str, name: String) -> Result<WorkspaceSnapshot> {
        let Some(workspace) = self.get_workspace(workspace_id).await? else {
            anyhow::bail!("Workspace not found: {}", workspace_id);
        };

        let snapshot = WorkspaceSnapshot::from_workspace(&workspace, name);

        let layout_json = serde_json::to_string(&snapshot.layout)
            .context("Failed to serialize snapshot layout")?;

        sqlx::query(
            r#"
            INSERT INTO workspace_snapshots (id, workspace_id, name, layout, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&snapshot.id)
        .bind(&snapshot.workspace_id)
        .bind(&snapshot.name)
        .bind(&layout_json)
        .bind(snapshot.created_at.timestamp())
        .execute(&*self.db)
        .await
        .context("Failed to insert workspace snapshot")?;

        info!("Created snapshot: {} for workspace {}", snapshot.name, workspace_id);
        Ok(snapshot)
    }

    /// List snapshots for a workspace
    pub async fn list_snapshots(&self, workspace_id: &str) -> Result<Vec<WorkspaceSnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT id, workspace_id, name, layout, created_at
            FROM workspace_snapshots
            WHERE workspace_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&*self.db)
        .await
        .context("Failed to list workspace snapshots")?;

        let mut snapshots = Vec::new();
        for row in rows {
            let layout_json: String = row.get("layout");
            let layout: WorkspaceLayout = serde_json::from_str(&layout_json)?;

            let created_at_ts: i64 = row.get("created_at");

            snapshots.push(WorkspaceSnapshot {
                id: row.get("id"),
                workspace_id: row.get("workspace_id"),
                name: row.get("name"),
                layout,
                created_at: Utc.timestamp_opt(created_at_ts, 0).unwrap(),
            });
        }

        debug!("Listed {} snapshots for workspace {}", snapshots.len(), workspace_id);
        Ok(snapshots)
    }

    /// Restore a workspace from a snapshot
    pub async fn restore_snapshot(&self, snapshot_id: &str) -> Result<Option<Workspace>> {
        // Fetch snapshot
        let row = sqlx::query(
            r#"
            SELECT id, workspace_id, name, layout, created_at
            FROM workspace_snapshots
            WHERE id = ?
            "#,
        )
        .bind(snapshot_id)
        .fetch_optional(&*self.db)
        .await
        .context("Failed to fetch snapshot")?;

        let Some(row) = row else {
            return Ok(None);
        };

        let layout_json: String = row.get("layout");
        let layout: WorkspaceLayout = serde_json::from_str(&layout_json)?;

        let created_at_ts: i64 = row.get("created_at");

        let snapshot = WorkspaceSnapshot {
            id: row.get("id"),
            workspace_id: row.get("workspace_id"),
            name: row.get("name"),
            layout,
            created_at: Utc.timestamp_opt(created_at_ts, 0).unwrap(),
        };

        // Update workspace with snapshot layout
        let update_req = UpdateWorkspaceRequest {
            name: None,
            description: None,
            icon: None,
            layout: Some(snapshot.layout),
            tags: None,
        };

        let workspace = self.update_workspace(&snapshot.workspace_id, update_req).await?;

        info!("Restored workspace {} from snapshot {}", snapshot.workspace_id, snapshot_id);
        Ok(workspace)
    }

    /// Add session to workspace
    pub async fn add_session(
        &self,
        workspace_id: &str,
        session_id: &str,
        pane_id: &str,
        position: i32,
        session_config: Option<SessionConfig>,
    ) -> Result<()> {
        let config_json = session_config
            .map(|c| serde_json::to_string(&c))
            .transpose()
            .context("Failed to serialize session config")?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO workspace_sessions (workspace_id, session_id, pane_id, position, session_config)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(workspace_id)
        .bind(session_id)
        .bind(pane_id)
        .bind(position)
        .bind(config_json)
        .execute(&*self.db)
        .await
        .context("Failed to add session to workspace")?;

        debug!("Added session {} to workspace {} at pane {}", session_id, workspace_id, pane_id);
        Ok(())
    }

    /// Get sessions for workspace
    pub async fn get_workspace_sessions(&self, workspace_id: &str) -> Result<Vec<WorkspaceSession>> {
        let rows = sqlx::query(
            r#"
            SELECT workspace_id, session_id, pane_id, position, session_config
            FROM workspace_sessions
            WHERE workspace_id = ?
            ORDER BY position ASC
            "#,
        )
        .bind(workspace_id)
        .fetch_all(&*self.db)
        .await
        .context("Failed to fetch workspace sessions")?;

        let mut sessions = Vec::new();
        for row in rows {
            let config_json: Option<String> = row.get("session_config");
            let session_config: Option<SessionConfig> = config_json
                .as_ref()
                .map(|j| serde_json::from_str(j.as_str()))
                .transpose()?;

            sessions.push(WorkspaceSession {
                workspace_id: row.get("workspace_id"),
                session_id: row.get("session_id"),
                pane_id: row.get("pane_id"),
                position: row.get("position"),
                session_config,
            });
        }

        debug!("Fetched {} sessions for workspace {}", sessions.len(), workspace_id);
        Ok(sessions)
    }

    /// Remove session from workspace
    pub async fn remove_session(&self, workspace_id: &str, session_id: &str) -> Result<bool> {
        let result = sqlx::query(
            "DELETE FROM workspace_sessions WHERE workspace_id = ? AND session_id = ?",
        )
        .bind(workspace_id)
        .bind(session_id)
        .execute(&*self.db)
        .await
        .context("Failed to remove session from workspace")?;

        let removed = result.rows_affected() > 0;
        if removed {
            debug!("Removed session {} from workspace {}", session_id, workspace_id);
        }

        Ok(removed)
    }

    /// Get workspace count
    pub async fn count_workspaces(&self, is_template: Option<bool>) -> Result<i64> {
        let count: (i64,) = if let Some(template) = is_template {
            sqlx::query_as("SELECT COUNT(*) FROM workspaces WHERE is_template = ?")
                .bind(template)
                .fetch_one(&*self.db)
                .await?
        } else {
            sqlx::query_as("SELECT COUNT(*) FROM workspaces")
                .fetch_one(&*self.db)
                .await?
        };

        Ok(count.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_test_db() -> Arc<Pool<Sqlite>> {
        let pool = SqlitePoolOptions::new()
            .connect(":memory:")
            .await
            .expect("Failed to create test database");

        Arc::new(pool)
    }

    #[tokio::test]
    async fn test_create_and_get_workspace() {
        let db = setup_test_db().await;
        let service = WorkspaceService::new(db);
        service.initialize().await.expect("Failed to initialize");

        let req = CreateWorkspaceRequest {
            name: "Test Workspace".to_string(),
            description: Some("A test workspace".to_string()),
            icon: Some("🚀".to_string()),
            layout: WorkspaceLayout::default(),
            is_template: false,
            tags: Some(vec!["test".to_string()]),
        };

        let created = service.create_workspace(req).await.expect("Failed to create workspace");
        assert_eq!(created.name, "Test Workspace");

        let fetched = service
            .get_workspace(&created.id)
            .await
            .expect("Failed to fetch workspace")
            .expect("Workspace not found");

        assert_eq!(fetched.id, created.id);
        assert_eq!(fetched.name, created.name);
    }

    #[tokio::test]
    async fn test_update_workspace() {
        let db = setup_test_db().await;
        let service = WorkspaceService::new(db);
        service.initialize().await.expect("Failed to initialize");

        let req = CreateWorkspaceRequest {
            name: "Original Name".to_string(),
            description: None,
            icon: None,
            layout: WorkspaceLayout::default(),
            is_template: false,
            tags: None,
        };

        let created = service.create_workspace(req).await.expect("Failed to create workspace");

        let update_req = UpdateWorkspaceRequest {
            name: Some("Updated Name".to_string()),
            description: Some("New description".to_string()),
            icon: None,
            layout: None,
            tags: None,
        };

        let updated = service
            .update_workspace(&created.id, update_req)
            .await
            .expect("Failed to update workspace")
            .expect("Workspace not found");

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.description, Some("New description".to_string()));
    }

    #[tokio::test]
    async fn test_delete_workspace() {
        let db = setup_test_db().await;
        let service = WorkspaceService::new(db);
        service.initialize().await.expect("Failed to initialize");

        let req = CreateWorkspaceRequest {
            name: "To Delete".to_string(),
            description: None,
            icon: None,
            layout: WorkspaceLayout::default(),
            is_template: false,
            tags: None,
        };

        let created = service.create_workspace(req).await.expect("Failed to create workspace");

        let deleted = service
            .delete_workspace(&created.id)
            .await
            .expect("Failed to delete workspace");

        assert!(deleted);

        let fetched = service
            .get_workspace(&created.id)
            .await
            .expect("Failed to fetch workspace");

        assert!(fetched.is_none());
    }

    #[tokio::test]
    async fn test_list_workspaces() {
        let db = setup_test_db().await;
        let service = WorkspaceService::new(db);
        service.initialize().await.expect("Failed to initialize");

        // Create multiple workspaces
        for i in 1..=3 {
            let req = CreateWorkspaceRequest {
                name: format!("Workspace {}", i),
                description: None,
                icon: None,
                layout: WorkspaceLayout::default(),
                is_template: i == 1, // First one is a template
                tags: None,
            };

            service.create_workspace(req).await.expect("Failed to create workspace");
        }

        let all_workspaces = service
            .list_workspaces(WorkspaceFilter::default())
            .await
            .expect("Failed to list workspaces");

        assert_eq!(all_workspaces.len(), 3);

        let templates = service
            .list_workspaces(WorkspaceFilter {
                is_template: Some(true),
                ..Default::default()
            })
            .await
            .expect("Failed to list templates");

        assert_eq!(templates.len(), 1);
    }

    #[tokio::test]
    async fn test_snapshots() {
        let db = setup_test_db().await;
        let service = WorkspaceService::new(db);
        service.initialize().await.expect("Failed to initialize");

        let req = CreateWorkspaceRequest {
            name: "Workspace with Snapshots".to_string(),
            description: None,
            icon: None,
            layout: WorkspaceLayout::default(),
            is_template: false,
            tags: None,
        };

        let workspace = service.create_workspace(req).await.expect("Failed to create workspace");

        // Create snapshots
        let snap1 = service
            .save_snapshot(&workspace.id, "Snapshot 1".to_string())
            .await
            .expect("Failed to create snapshot");

        let snap2 = service
            .save_snapshot(&workspace.id, "Snapshot 2".to_string())
            .await
            .expect("Failed to create snapshot");

        let snapshots = service
            .list_snapshots(&workspace.id)
            .await
            .expect("Failed to list snapshots");

        assert_eq!(snapshots.len(), 2);
        assert!(snapshots.iter().any(|s| s.id == snap1.id));
        assert!(snapshots.iter().any(|s| s.id == snap2.id));
    }
}
