-- Workspaces Migration
-- Manages workspace layouts and configurations

-- Workspaces table
CREATE TABLE IF NOT EXISTS workspaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    icon TEXT,
    layout JSONB NOT NULL,  -- Split-pane layout structure
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_template BOOLEAN NOT NULL DEFAULT FALSE,
    tags TEXT  -- JSON array of tags
);

-- Workspace sessions mapping
CREATE TABLE IF NOT EXISTS workspace_sessions (
    workspace_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    pane_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    session_config JSONB,  -- SSH/local session configuration
    PRIMARY KEY (workspace_id, session_id),
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

-- Workspace snapshots (versioning)
CREATE TABLE IF NOT EXISTS workspace_snapshots (
    id TEXT PRIMARY KEY,
    workspace_id TEXT NOT NULL,
    name TEXT NOT NULL,
    layout JSONB NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (workspace_id) REFERENCES workspaces(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_workspaces_updated_at ON workspaces(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_workspaces_is_template ON workspaces(is_template);
CREATE INDEX IF NOT EXISTS idx_workspace_sessions_workspace_id ON workspace_sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_workspace_snapshots_workspace_id ON workspace_snapshots(workspace_id);
CREATE INDEX IF NOT EXISTS idx_workspace_snapshots_created_at ON workspace_snapshots(created_at DESC);

-- Trigger to update updated_at timestamp
CREATE TRIGGER IF NOT EXISTS update_workspace_timestamp
AFTER UPDATE ON workspaces
BEGIN
    UPDATE workspaces SET updated_at = CURRENT_TIMESTAMP WHERE id = NEW.id;
END;
