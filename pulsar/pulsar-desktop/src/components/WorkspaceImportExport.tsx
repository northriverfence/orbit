/**
 * Workspace Import/Export Component
 *
 * UI for importing and exporting workspace configurations
 */

import React, { useRef, useState } from 'react';
import { useWorkspace } from '../lib/WorkspaceManager';
import {
  exportWorkspace,
  importWorkspace,
  exportWorkspaceCollection,
  shareWorkspaceToClipboard,
  importWorkspaceFromClipboard,
} from '../lib/workspaceIO';

interface WorkspaceImportExportProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function WorkspaceImportExport({ isOpen, onClose }: WorkspaceImportExportProps) {
  const { workspaces, createWorkspace, currentWorkspace } = useWorkspace();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [importing, setImporting] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const handleExportCurrent = async () => {
    if (!currentWorkspace) {
      setMessage({ type: 'error', text: 'No workspace selected' });
      return;
    }

    setExporting(true);
    try {
      await exportWorkspace(currentWorkspace);
      setMessage({ type: 'success', text: 'Workspace exported successfully' });
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to export workspace' });
    } finally {
      setExporting(false);
    }
  };

  const handleExportAll = async () => {
    if (workspaces.length === 0) {
      setMessage({ type: 'error', text: 'No workspaces to export' });
      return;
    }

    setExporting(true);
    try {
      await exportWorkspaceCollection(workspaces);
      setMessage({ type: 'success', text: `Exported ${workspaces.length} workspaces` });
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to export workspaces' });
    } finally {
      setExporting(false);
    }
  };

  const handleShareToClipboard = async () => {
    if (!currentWorkspace) {
      setMessage({ type: 'error', text: 'No workspace selected' });
      return;
    }

    try {
      await shareWorkspaceToClipboard(currentWorkspace);
      setMessage({ type: 'success', text: 'Copied to clipboard' });
    } catch (error) {
      setMessage({ type: 'error', text: 'Failed to copy to clipboard' });
    }
  };

  const handleImportFromFile = async (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    setImporting(true);
    setMessage(null);

    try {
      const workspaceRequest = await importWorkspace(file);
      await createWorkspace(workspaceRequest);
      setMessage({ type: 'success', text: `Imported workspace: ${workspaceRequest.name}` });

      // Reset file input
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to import workspace',
      });
    } finally {
      setImporting(false);
    }
  };

  const handleImportFromClipboard = async () => {
    setImporting(true);
    setMessage(null);

    try {
      const workspaceRequest = await importWorkspaceFromClipboard();
      await createWorkspace(workspaceRequest);
      setMessage({ type: 'success', text: `Imported workspace: ${workspaceRequest.name}` });
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to import from clipboard',
      });
    } finally {
      setImporting(false);
    }
  };

  const triggerFileInput = () => {
    fileInputRef.current?.click();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-800">Import / Export</h2>
            <p className="text-sm text-gray-500 mt-1">
              Share workspace configurations as files
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
        <div className="px-6 py-4 space-y-6">
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

          {/* Export Section */}
          <div>
            <h3 className="text-lg font-semibold text-gray-800 mb-3">Export</h3>
            <div className="space-y-3">
              <button
                onClick={handleExportCurrent}
                disabled={!currentWorkspace || exporting}
                className="w-full px-4 py-3 bg-white border-2 border-gray-200 rounded hover:border-gray-300 transition-colors text-left flex items-center justify-between disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div>
                  <div className="font-medium text-gray-800">Export Current Workspace</div>
                  <div className="text-sm text-gray-500">
                    {currentWorkspace
                      ? `${currentWorkspace.name} → JSON file`
                      : 'No workspace selected'}
                  </div>
                </div>
                <span className="text-2xl">📥</span>
              </button>

              <button
                onClick={handleExportAll}
                disabled={workspaces.length === 0 || exporting}
                className="w-full px-4 py-3 bg-white border-2 border-gray-200 rounded hover:border-gray-300 transition-colors text-left flex items-center justify-between disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div>
                  <div className="font-medium text-gray-800">Export All Workspaces</div>
                  <div className="text-sm text-gray-500">
                    {workspaces.length} workspace(s) → JSON collection
                  </div>
                </div>
                <span className="text-2xl">📦</span>
              </button>

              <button
                onClick={handleShareToClipboard}
                disabled={!currentWorkspace}
                className="w-full px-4 py-3 bg-white border-2 border-gray-200 rounded hover:border-gray-300 transition-colors text-left flex items-center justify-between disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div>
                  <div className="font-medium text-gray-800">Copy to Clipboard</div>
                  <div className="text-sm text-gray-500">
                    Share workspace configuration as JSON
                  </div>
                </div>
                <span className="text-2xl">📋</span>
              </button>
            </div>
          </div>

          {/* Import Section */}
          <div>
            <h3 className="text-lg font-semibold text-gray-800 mb-3">Import</h3>
            <div className="space-y-3">
              <button
                onClick={triggerFileInput}
                disabled={importing}
                className="w-full px-4 py-3 bg-white border-2 border-gray-200 rounded hover:border-gray-300 transition-colors text-left flex items-center justify-between disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div>
                  <div className="font-medium text-gray-800">Import from File</div>
                  <div className="text-sm text-gray-500">
                    Load workspace from JSON file
                  </div>
                </div>
                <span className="text-2xl">📂</span>
              </button>

              <button
                onClick={handleImportFromClipboard}
                disabled={importing}
                className="w-full px-4 py-3 bg-white border-2 border-gray-200 rounded hover:border-gray-300 transition-colors text-left flex items-center justify-between disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div>
                  <div className="font-medium text-gray-800">Import from Clipboard</div>
                  <div className="text-sm text-gray-500">
                    Paste workspace JSON from clipboard
                  </div>
                </div>
                <span className="text-2xl">📄</span>
              </button>
            </div>
          </div>

          {/* Hidden file input */}
          <input
            ref={fileInputRef}
            type="file"
            accept=".json,application/json"
            onChange={handleImportFromFile}
            className="hidden"
          />
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 flex justify-end">
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
