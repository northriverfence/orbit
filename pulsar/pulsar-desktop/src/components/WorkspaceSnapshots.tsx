/**
 * Workspace Snapshots Component
 *
 * Manage workspace snapshots (save/restore/list)
 */

import React, { useState, useEffect } from 'react';
import { useWorkspace } from '../lib/WorkspaceManager';
import { WorkspaceClient } from '../lib/workspaceClient';
import type { WorkspaceSnapshot } from '../types/workspace';

interface WorkspaceSnapshotsProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function WorkspaceSnapshots({ isOpen, onClose }: WorkspaceSnapshotsProps) {
  const { currentWorkspace, refreshWorkspaces } = useWorkspace();
  const [snapshots, setSnapshots] = useState<WorkspaceSnapshot[]>([]);
  const [loading, setLoading] = useState(false);
  const [creating, setCreating] = useState(false);
  const [restoring, setRestoring] = useState(false);
  const [snapshotName, setSnapshotName] = useState('');
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  // Load snapshots when dialog opens
  useEffect(() => {
    if (isOpen && currentWorkspace) {
      loadSnapshots();
    }
  }, [isOpen, currentWorkspace]);

  const loadSnapshots = async () => {
    if (!currentWorkspace) return;

    setLoading(true);
    setMessage(null);

    try {
      const result = await WorkspaceClient.listSnapshots(currentWorkspace.id);
      setSnapshots(result);
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to load snapshots',
      });
    } finally {
      setLoading(false);
    }
  };

  const handleCreateSnapshot = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!currentWorkspace || !snapshotName.trim()) {
      setMessage({ type: 'error', text: 'Snapshot name is required' });
      return;
    }

    setCreating(true);
    setMessage(null);

    try {
      await WorkspaceClient.saveSnapshot(currentWorkspace.id, snapshotName.trim());
      setMessage({ type: 'success', text: 'Snapshot created successfully' });
      setSnapshotName('');
      await loadSnapshots();
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to create snapshot',
      });
    } finally {
      setCreating(false);
    }
  };

  const handleRestoreSnapshot = async (snapshotId: string) => {
    if (!confirm('Restore workspace from this snapshot? This will replace the current layout.')) {
      return;
    }

    setRestoring(true);
    setMessage(null);

    try {
      await WorkspaceClient.restoreSnapshot(snapshotId);
      setMessage({ type: 'success', text: 'Workspace restored successfully' });
      await refreshWorkspaces();
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to restore snapshot',
      });
    } finally {
      setRestoring(false);
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-3xl w-full mx-4 max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-800">Workspace Snapshots</h2>
            <p className="text-sm text-gray-500 mt-1">
              {currentWorkspace
                ? `Snapshots for: ${currentWorkspace.name}`
                : 'No workspace selected'}
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl"
          >
            ×
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 space-y-6">
          {/* Message */}
          {message && (
            <div
              className={`px-4 py-3 rounded ${
                message.type === 'success'
                  ? 'bg-green-50 text-green-700'
                  : 'bg-red-50 text-red-700'
              }`}
            >
              {message.text}
            </div>
          )}

          {/* Create Snapshot */}
          <div>
            <h3 className="text-lg font-semibold text-gray-800 mb-3">Create New Snapshot</h3>
            <form onSubmit={handleCreateSnapshot} className="flex gap-3">
              <input
                type="text"
                value={snapshotName}
                onChange={(e) => setSnapshotName(e.target.value)}
                placeholder="Snapshot name (e.g., Before refactor)"
                className="flex-1 px-4 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                disabled={creating || !currentWorkspace}
              />
              <button
                type="submit"
                disabled={creating || !snapshotName.trim() || !currentWorkspace}
                className="px-6 py-2 bg-accent-primary text-white rounded hover:bg-accent-primary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {creating ? 'Creating...' : 'Create'}
              </button>
            </form>
          </div>

          {/* Snapshots List */}
          <div>
            <h3 className="text-lg font-semibold text-gray-800 mb-3">
              Saved Snapshots ({snapshots.length})
            </h3>

            {loading && (
              <div className="text-center py-8 text-gray-500">Loading snapshots...</div>
            )}

            {!loading && !currentWorkspace && (
              <div className="text-center py-8 text-gray-500">
                No workspace selected
              </div>
            )}

            {!loading && currentWorkspace && snapshots.length === 0 && (
              <div className="text-center py-8 text-gray-400">
                <span className="text-5xl block mb-3">📸</span>
                <p>No snapshots yet</p>
                <p className="text-sm mt-1">Create your first snapshot above</p>
              </div>
            )}

            {!loading && snapshots.length > 0 && (
              <div className="space-y-2">
                {snapshots.map((snapshot) => (
                  <div
                    key={snapshot.id}
                    className="flex items-center justify-between p-4 border-2 border-gray-200 rounded hover:border-gray-300 transition-colors"
                  >
                    <div className="flex-1 min-w-0">
                      <h4 className="font-medium text-gray-800">{snapshot.name}</h4>
                      <p className="text-sm text-gray-500 mt-1">
                        Created: {formatDate(snapshot.created_at)}
                      </p>
                      <div className="text-xs text-gray-400 mt-1">
                        Layout: {snapshot.layout.panes.length} pane(s)
                      </div>
                    </div>
                    <button
                      onClick={() => handleRestoreSnapshot(snapshot.id)}
                      disabled={restoring}
                      className="ml-4 px-4 py-2 bg-accent-secondary text-white rounded hover:bg-accent-secondary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                    >
                      {restoring ? 'Restoring...' : 'Restore'}
                    </button>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 flex justify-between items-center">
          <div className="text-sm text-gray-500">
            💡 Tip: Snapshots let you save and restore workspace layouts
          </div>
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded transition-colors"
          >
            Close
          </button>
        </div>
      </div>
    </div>
  );
}
