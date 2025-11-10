/**
 * Workspace Type Definitions
 *
 * These types mirror the Rust workspace models in pulsar-daemon
 */

/** Workspace represents a collection of terminal sessions with a specific layout */
export interface Workspace {
  id: string;
  name: string;
  description?: string;
  icon?: string;
  layout: WorkspaceLayout;
  created_at: string; // ISO 8601 timestamp
  updated_at: string; // ISO 8601 timestamp
  is_template: boolean;
  tags?: string[];
}

/** Workspace layout structure (mirrors frontend split-pane layout) */
export interface WorkspaceLayout {
  version: string;
  type: 'single' | 'split';
  panes: PaneConfig[];
  active_pane?: string;
}

/** Pane configuration */
export interface PaneConfig {
  id: string;
  session_id?: string;
  size: number; // percentage (0-100)
  direction?: 'horizontal' | 'vertical';
  children?: PaneConfig[];
  min_size?: number;
  max_size?: number;
}

/** Workspace session mapping */
export interface WorkspaceSession {
  workspace_id: string;
  session_id: string;
  pane_id: string;
  position: number;
  session_config?: SessionConfig;
}

/** Session configuration */
export interface SessionConfig {
  type: 'local' | 'ssh';
  name: string;
  host?: string;
  port?: number;
  username?: string;
}

/** Workspace snapshot (version) */
export interface WorkspaceSnapshot {
  id: string;
  workspace_id: string;
  name: string;
  layout: WorkspaceLayout;
  created_at: string;
}

/** Create workspace request */
export interface CreateWorkspaceRequest {
  name: string;
  description?: string;
  icon?: string;
  layout: WorkspaceLayout;
  is_template: boolean;
  tags?: string[];
}

/** Update workspace request */
export interface UpdateWorkspaceRequest {
  name?: string;
  description?: string;
  icon?: string;
  layout?: WorkspaceLayout;
  tags?: string[];
}

/** Workspace filter */
export interface WorkspaceFilter {
  is_template?: boolean;
  tags?: string[];
  search?: string;
}

/** Default single-pane layout */
export const createDefaultLayout = (): WorkspaceLayout => ({
  version: '1.0.0',
  type: 'single',
  panes: [
    {
      id: `pane-${crypto.randomUUID()}`,
      size: 100,
      min_size: 100,
    },
  ],
});

/** Create a horizontal split layout */
export const createHorizontalSplit = (): WorkspaceLayout => ({
  version: '1.0.0',
  type: 'split',
  panes: [
    {
      id: `pane-${crypto.randomUUID()}`,
      size: 50,
      direction: 'horizontal',
      min_size: 10,
    },
    {
      id: `pane-${crypto.randomUUID()}`,
      size: 50,
      direction: 'horizontal',
      min_size: 10,
    },
  ],
});

/** Create a vertical split layout */
export const createVerticalSplit = (): WorkspaceLayout => ({
  version: '1.0.0',
  type: 'split',
  panes: [
    {
      id: `pane-${crypto.randomUUID()}`,
      size: 50,
      direction: 'vertical',
      min_size: 10,
    },
    {
      id: `pane-${crypto.randomUUID()}`,
      size: 50,
      direction: 'vertical',
      min_size: 10,
    },
  ],
});
