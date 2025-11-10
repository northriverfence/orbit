/**
 * Workspace Manager
 *
 * React context provider for managing workspace state and operations
 */

import React, { createContext, useContext, useState, useEffect, useCallback } from 'react';
import { WorkspaceClient } from './workspaceClient';
import { SessionAutoStartService } from './sessionAutoStart';
import type { Workspace, CreateWorkspaceRequest, UpdateWorkspaceRequest } from '../types/workspace';

interface WorkspaceContextType {
  // State
  workspaces: Workspace[];
  templates: Workspace[];
  currentWorkspace: Workspace | null;
  loading: boolean;
  error: string | null;
  activeSessions: Map<string, string>; // paneId -> sessionId
  startingSessions: boolean; // Loading state for session startup

  // Actions
  loadWorkspaces: () => Promise<void>;
  loadTemplates: () => Promise<void>;
  createWorkspace: (request: CreateWorkspaceRequest) => Promise<Workspace>;
  updateWorkspace: (id: string, request: UpdateWorkspaceRequest) => Promise<Workspace | null>;
  deleteWorkspace: (id: string) => Promise<void>;
  switchWorkspace: (id: string) => Promise<void>;
  duplicateWorkspace: (id: string, newName: string) => Promise<Workspace>;
  refreshWorkspaces: () => Promise<void>;
}

const WorkspaceContext = createContext<WorkspaceContextType | undefined>(undefined);

interface WorkspaceProviderProps {
  children: React.ReactNode;
}

export function WorkspaceProvider({ children }: WorkspaceProviderProps) {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [templates, setTemplates] = useState<Workspace[]>([]);
  const [currentWorkspace, setCurrentWorkspace] = useState<Workspace | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [activeSessions, setActiveSessions] = useState<Map<string, string>>(new Map());
  const [startingSessions, setStartingSessions] = useState(false);

  // Load user workspaces
  const loadWorkspaces = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await WorkspaceClient.getUserWorkspaces();
      setWorkspaces(result);

      // If no current workspace, select the first one
      if (!currentWorkspace && result.length > 0) {
        setCurrentWorkspace(result[0]);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load workspaces');
      console.error('Failed to load workspaces:', err);
    } finally {
      setLoading(false);
    }
  }, [currentWorkspace]);

  // Load workspace templates
  const loadTemplates = useCallback(async () => {
    try {
      const result = await WorkspaceClient.getTemplates();
      setTemplates(result);
    } catch (err) {
      console.error('Failed to load templates:', err);
    }
  }, []);

  // Create a new workspace
  const createWorkspace = useCallback(async (request: CreateWorkspaceRequest): Promise<Workspace> => {
    setError(null);
    try {
      const workspace = await WorkspaceClient.create(request);
      setWorkspaces(prev => [...prev, workspace]);
      return workspace;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create workspace');
      throw err;
    }
  }, []);

  // Update a workspace
  const updateWorkspace = useCallback(async (
    id: string,
    request: UpdateWorkspaceRequest
  ): Promise<Workspace | null> => {
    setError(null);
    try {
      const updated = await WorkspaceClient.update(id, request);
      if (updated) {
        setWorkspaces(prev =>
          prev.map(w => (w.id === id ? updated : w))
        );
        if (currentWorkspace?.id === id) {
          setCurrentWorkspace(updated);
        }
      }
      return updated;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to update workspace');
      throw err;
    }
  }, [currentWorkspace]);

  // Delete a workspace
  const deleteWorkspace = useCallback(async (id: string): Promise<void> => {
    setError(null);
    try {
      const deleted = await WorkspaceClient.delete(id);
      if (deleted) {
        setWorkspaces(prev => prev.filter(w => w.id !== id));

        // If deleted workspace was current, switch to first available
        if (currentWorkspace?.id === id) {
          const remaining = workspaces.filter(w => w.id !== id);
          setCurrentWorkspace(remaining.length > 0 ? remaining[0] : null);
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete workspace');
      throw err;
    }
  }, [currentWorkspace, workspaces]);

  // Switch to a different workspace
  const switchWorkspace = useCallback(async (id: string): Promise<void> => {
    setError(null);
    try {
      const workspace = await WorkspaceClient.get(id);
      if (workspace) {
        setCurrentWorkspace(workspace);
        // Store in localStorage for persistence
        localStorage.setItem('currentWorkspaceId', id);

        // Load and execute session auto-start configuration
        const startupConfig = await SessionAutoStartService.loadStartupConfig(id);
        if (startupConfig && startupConfig.autoStart) {
          setStartingSessions(true);
          console.log('Starting workspace sessions...');

          try {
            const sessionMap = await SessionAutoStartService.startWorkspaceSessions(
              startupConfig,
              (paneId, sessionId) => {
                console.log(`Session created for pane ${paneId}: ${sessionId}`);
              },
              (paneId, error) => {
                console.error(`Failed to start session for pane ${paneId}:`, error);
              }
            );

            setActiveSessions(sessionMap);
            console.log(`Started ${sessionMap.size} sessions`);
          } finally {
            setStartingSessions(false);
          }
        }
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to switch workspace');
      throw err;
    }
  }, []);

  // Duplicate a workspace
  const duplicateWorkspace = useCallback(async (id: string, newName: string): Promise<Workspace> => {
    setError(null);
    try {
      const duplicated = await WorkspaceClient.duplicate(id, newName);
      setWorkspaces(prev => [...prev, duplicated]);
      return duplicated;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to duplicate workspace');
      throw err;
    }
  }, []);

  // Refresh workspaces
  const refreshWorkspaces = useCallback(async () => {
    await Promise.all([loadWorkspaces(), loadTemplates()]);
  }, [loadWorkspaces, loadTemplates]);

  // Load workspaces on mount
  useEffect(() => {
    loadWorkspaces();
    loadTemplates();
  }, [loadWorkspaces, loadTemplates]);

  // Restore current workspace from localStorage
  useEffect(() => {
    const savedId = localStorage.getItem('currentWorkspaceId');
    if (savedId && workspaces.length > 0) {
      const workspace = workspaces.find(w => w.id === savedId);
      if (workspace) {
        setCurrentWorkspace(workspace);
      }
    }
  }, [workspaces]);

  const value: WorkspaceContextType = {
    workspaces,
    templates,
    currentWorkspace,
    loading,
    error,
    activeSessions,
    startingSessions,
    loadWorkspaces,
    loadTemplates,
    createWorkspace,
    updateWorkspace,
    deleteWorkspace,
    switchWorkspace,
    duplicateWorkspace,
    refreshWorkspaces,
  };

  return (
    <WorkspaceContext.Provider value={value}>
      {children}
    </WorkspaceContext.Provider>
  );
}

/**
 * Hook to use workspace context
 */
export function useWorkspace() {
  const context = useContext(WorkspaceContext);
  if (!context) {
    throw new Error('useWorkspace must be used within a WorkspaceProvider');
  }
  return context;
}
