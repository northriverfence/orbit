/**
 * Workspace Import/Export
 *
 * Functions for importing and exporting workspace configurations as JSON files
 */

import type { Workspace, CreateWorkspaceRequest } from '../types/workspace';

export interface WorkspaceExport {
  version: string;
  exported_at: string;
  workspace: {
    name: string;
    description?: string;
    icon?: string;
    layout: Workspace['layout'];
    tags?: string[];
  };
}

/**
 * Export workspace to JSON file
 */
export async function exportWorkspace(workspace: Workspace): Promise<void> {
  const exportData: WorkspaceExport = {
    version: '1.0.0',
    exported_at: new Date().toISOString(),
    workspace: {
      name: workspace.name,
      description: workspace.description,
      icon: workspace.icon,
      layout: workspace.layout,
      tags: workspace.tags,
    },
  };

  const json = JSON.stringify(exportData, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);

  // Create download link
  const a = document.createElement('a');
  a.href = url;
  a.download = `${workspace.name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}_workspace.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Import workspace from JSON file
 */
export async function importWorkspace(file: File): Promise<CreateWorkspaceRequest> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();

    reader.onload = (e) => {
      try {
        const text = e.target?.result as string;
        const data = JSON.parse(text) as WorkspaceExport;

        // Validate structure
        if (!data.workspace || !data.workspace.name || !data.workspace.layout) {
          throw new Error('Invalid workspace file format');
        }

        // Validate layout structure
        if (
          !data.workspace.layout.version ||
          !data.workspace.layout.type ||
          !Array.isArray(data.workspace.layout.panes)
        ) {
          throw new Error('Invalid layout format');
        }

        // Create request object
        const request: CreateWorkspaceRequest = {
          name: data.workspace.name,
          description: data.workspace.description,
          icon: data.workspace.icon,
          layout: data.workspace.layout,
          is_template: false,
          tags: data.workspace.tags,
        };

        resolve(request);
      } catch (error) {
        reject(new Error(`Failed to parse workspace file: ${error}`));
      }
    };

    reader.onerror = () => {
      reject(new Error('Failed to read file'));
    };

    reader.readAsText(file);
  });
}

/**
 * Export multiple workspaces as a collection
 */
export async function exportWorkspaceCollection(workspaces: Workspace[]): Promise<void> {
  const collectionData = {
    version: '1.0.0',
    exported_at: new Date().toISOString(),
    workspaces: workspaces.map((w) => ({
      name: w.name,
      description: w.description,
      icon: w.icon,
      layout: w.layout,
      tags: w.tags,
    })),
  };

  const json = JSON.stringify(collectionData, null, 2);
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);

  const a = document.createElement('a');
  a.href = url;
  a.download = `workspace_collection_${Date.now()}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

/**
 * Validate workspace JSON before import
 */
export function validateWorkspaceJSON(json: string): { valid: boolean; error?: string } {
  try {
    const data = JSON.parse(json);

    if (!data.workspace) {
      return { valid: false, error: 'Missing workspace object' };
    }

    if (!data.workspace.name) {
      return { valid: false, error: 'Missing workspace name' };
    }

    if (!data.workspace.layout) {
      return { valid: false, error: 'Missing workspace layout' };
    }

    if (!data.workspace.layout.panes || !Array.isArray(data.workspace.layout.panes)) {
      return { valid: false, error: 'Invalid layout panes' };
    }

    return { valid: true };
  } catch (error) {
    return { valid: false, error: 'Invalid JSON format' };
  }
}

/**
 * Share workspace via clipboard (JSON)
 */
export async function shareWorkspaceToClipboard(workspace: Workspace): Promise<void> {
  const exportData: WorkspaceExport = {
    version: '1.0.0',
    exported_at: new Date().toISOString(),
    workspace: {
      name: workspace.name,
      description: workspace.description,
      icon: workspace.icon,
      layout: workspace.layout,
      tags: workspace.tags,
    },
  };

  const json = JSON.stringify(exportData, null, 2);
  await navigator.clipboard.writeText(json);
}

/**
 * Import workspace from clipboard
 */
export async function importWorkspaceFromClipboard(): Promise<CreateWorkspaceRequest> {
  const text = await navigator.clipboard.readText();

  try {
    const data = JSON.parse(text) as WorkspaceExport;

    if (!data.workspace || !data.workspace.name || !data.workspace.layout) {
      throw new Error('Invalid workspace format in clipboard');
    }

    return {
      name: data.workspace.name,
      description: data.workspace.description,
      icon: data.workspace.icon,
      layout: data.workspace.layout,
      is_template: false,
      tags: data.workspace.tags,
    };
  } catch (error) {
    throw new Error(`Failed to import from clipboard: ${error}`);
  }
}
