/**
 * Workspace API Client
 *
 * Provides a clean interface for interacting with workspace management
 * via Tauri commands to the pulsar-daemon backend
 */

import { invoke } from '@tauri-apps/api/core';
import type {
  Workspace,
  WorkspaceLayout,
  WorkspaceSnapshot,
  CreateWorkspaceRequest,
  UpdateWorkspaceRequest,
  WorkspaceFilter,
} from '../types/workspace';

export class WorkspaceClient {
  /**
   * Create a new workspace
   */
  static async create(request: CreateWorkspaceRequest): Promise<Workspace> {
    try {
      const workspace = await invoke<Workspace>('workspace_create', { request });
      return workspace;
    } catch (error) {
      console.error('Failed to create workspace:', error);
      throw new Error(`Failed to create workspace: ${error}`);
    }
  }

  /**
   * Get a workspace by ID
   */
  static async get(id: string): Promise<Workspace | null> {
    try {
      const workspace = await invoke<Workspace | null>('workspace_get', { id });
      return workspace;
    } catch (error) {
      console.error('Failed to get workspace:', error);
      throw new Error(`Failed to get workspace: ${error}`);
    }
  }

  /**
   * List all workspaces with optional filter
   */
  static async list(filter?: WorkspaceFilter): Promise<Workspace[]> {
    try {
      const workspaces = await invoke<Workspace[]>('workspace_list', { filter });
      return workspaces;
    } catch (error) {
      console.error('Failed to list workspaces:', error);
      throw new Error(`Failed to list workspaces: ${error}`);
    }
  }

  /**
   * Update an existing workspace
   */
  static async update(id: string, request: UpdateWorkspaceRequest): Promise<Workspace | null> {
    try {
      const workspace = await invoke<Workspace | null>('workspace_update', { id, request });
      return workspace;
    } catch (error) {
      console.error('Failed to update workspace:', error);
      throw new Error(`Failed to update workspace: ${error}`);
    }
  }

  /**
   * Delete a workspace
   */
  static async delete(id: string): Promise<boolean> {
    try {
      const deleted = await invoke<boolean>('workspace_delete', { id });
      return deleted;
    } catch (error) {
      console.error('Failed to delete workspace:', error);
      throw new Error(`Failed to delete workspace: ${error}`);
    }
  }

  /**
   * Get all user workspaces (non-templates)
   */
  static async getUserWorkspaces(): Promise<Workspace[]> {
    return this.list({ is_template: false });
  }

  /**
   * Get all workspace templates
   */
  static async getTemplates(): Promise<Workspace[]> {
    return this.list({ is_template: true });
  }

  /**
   * Search workspaces by name or description
   */
  static async search(query: string): Promise<Workspace[]> {
    return this.list({ search: query });
  }

  /**
   * Update workspace layout
   */
  static async updateLayout(id: string, layout: WorkspaceLayout): Promise<Workspace | null> {
    return this.update(id, { layout });
  }

  /**
   * Rename a workspace
   */
  static async rename(id: string, name: string): Promise<Workspace | null> {
    return this.update(id, { name });
  }

  /**
   * Duplicate a workspace
   */
  static async duplicate(id: string, newName: string): Promise<Workspace> {
    const original = await this.get(id);
    if (!original) {
      throw new Error('Workspace not found');
    }

    return this.create({
      name: newName,
      description: original.description,
      icon: original.icon,
      layout: original.layout,
      is_template: false,
      tags: original.tags,
    });
  }

  // ============= Snapshot Methods =============

  /**
   * Save a snapshot of a workspace
   */
  static async saveSnapshot(workspaceId: string, name: string): Promise<WorkspaceSnapshot> {
    try {
      const snapshot = await invoke<WorkspaceSnapshot>('workspace_save_snapshot', {
        workspaceId,
        name,
      });
      return snapshot;
    } catch (error) {
      console.error('Failed to save snapshot:', error);
      throw new Error(`Failed to save snapshot: ${error}`);
    }
  }

  /**
   * List snapshots for a workspace
   */
  static async listSnapshots(workspaceId: string): Promise<WorkspaceSnapshot[]> {
    try {
      const snapshots = await invoke<WorkspaceSnapshot[]>('workspace_list_snapshots', {
        workspaceId,
      });
      return snapshots;
    } catch (error) {
      console.error('Failed to list snapshots:', error);
      throw new Error(`Failed to list snapshots: ${error}`);
    }
  }

  /**
   * Restore workspace from a snapshot
   */
  static async restoreSnapshot(snapshotId: string): Promise<Workspace | null> {
    try {
      const workspace = await invoke<Workspace | null>('workspace_restore_snapshot', {
        snapshotId,
      });
      return workspace;
    } catch (error) {
      console.error('Failed to restore snapshot:', error);
      throw new Error(`Failed to restore snapshot: ${error}`);
    }
  }
}

export default WorkspaceClient;
