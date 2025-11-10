/**
 * Session Configuration Dialog
 *
 * Configure automatic session startup for workspace panes
 */

import { useState, useEffect } from 'react';
import { useWorkspace } from '../lib/WorkspaceManager';
import { SessionAutoStartService } from '../lib/sessionAutoStart';
import type {
  SessionType,
  SessionStartupConfig,
  WorkspaceStartupConfig,
} from '../types/sessionAutoStart';
import { sessionExamples } from '../types/sessionAutoStart';

interface SessionConfigDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function SessionConfigDialog({ isOpen, onClose }: SessionConfigDialogProps) {
  const { currentWorkspace } = useWorkspace();

  // State for the entire workspace configuration
  const [config, setConfig] = useState<WorkspaceStartupConfig>({
    autoStart: false,
    panes: {},
    globalEnv: {},
    startupOrder: [],
  });

  const [selectedPaneId, setSelectedPaneId] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [showExamples, setShowExamples] = useState(false);

  // Load configuration when dialog opens
  useEffect(() => {
    if (isOpen && currentWorkspace) {
      loadConfiguration();
    }
  }, [isOpen, currentWorkspace]);

  const loadConfiguration = async () => {
    if (!currentWorkspace) return;

    // Load saved config or create default
    const saved = await SessionAutoStartService.loadStartupConfig(currentWorkspace.id);

    if (saved) {
      setConfig(saved);
    } else {
      // Create default config with panes from workspace
      const paneIds = currentWorkspace.layout.panes.map((p) => p.id);
      const defaultPanes: Record<string, SessionStartupConfig> = {};

      paneIds.forEach((id, index) => {
        defaultPanes[id] = {
          enabled: false,
          type: 'local',
          name: `Terminal ${index + 1}`,
          autoReconnect: false,
          startupDelay: 0,
          order: index,
        };
      });

      setConfig({
        autoStart: false,
        panes: defaultPanes,
        globalEnv: {},
        startupOrder: paneIds,
      });
    }

    // Select first pane by default
    if (currentWorkspace.layout.panes.length > 0) {
      setSelectedPaneId(currentWorkspace.layout.panes[0].id);
    }
  };

  const handleSave = async () => {
    if (!currentWorkspace) return;

    setSaving(true);
    setMessage(null);

    try {
      // Validate all pane configs
      const errors: string[] = [];
      Object.entries(config.panes).forEach(([paneId, paneConfig]) => {
        if (paneConfig.enabled) {
          const validation = SessionAutoStartService.validateConfig(paneConfig);
          if (!validation.valid) {
            errors.push(`Pane ${paneId}: ${validation.errors.join(', ')}`);
          }
        }
      });

      if (errors.length > 0) {
        setMessage({ type: 'error', text: errors.join('\n') });
        return;
      }

      // Save configuration
      await SessionAutoStartService.saveStartupConfig(currentWorkspace.id, config);
      setMessage({ type: 'success', text: 'Configuration saved successfully' });

      // Close after short delay
      setTimeout(() => {
        onClose();
      }, 1500);
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to save configuration',
      });
    } finally {
      setSaving(false);
    }
  };

  const updatePaneConfig = (paneId: string, updates: Partial<SessionStartupConfig>) => {
    setConfig((prev) => ({
      ...prev,
      panes: {
        ...prev.panes,
        [paneId]: {
          ...prev.panes[paneId],
          ...updates,
        },
      },
    }));
  };

  const applyExample = (exampleKey: keyof typeof sessionExamples) => {
    if (!selectedPaneId) return;

    const example = sessionExamples[exampleKey];
    updatePaneConfig(selectedPaneId, example);
    setShowExamples(false);
  };

  const addEnvVar = () => {
    if (!selectedPaneId) return;

    const key = prompt('Environment variable name:');
    if (!key) return;

    const value = prompt('Environment variable value:');
    if (value === null) return;

    updatePaneConfig(selectedPaneId, {
      env: {
        ...config.panes[selectedPaneId]?.env,
        [key]: value,
      },
    });
  };

  const removeEnvVar = (key: string) => {
    if (!selectedPaneId) return;

    const { [key]: removed, ...rest } = config.panes[selectedPaneId]?.env || {};
    updatePaneConfig(selectedPaneId, { env: rest });
  };

  const selectedConfig = selectedPaneId ? config.panes[selectedPaneId] : null;

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-6xl w-full mx-4 max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-800">Session Auto-Start Configuration</h2>
            <p className="text-sm text-gray-500 mt-1">
              {currentWorkspace
                ? `Configure sessions for: ${currentWorkspace.name}`
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
        <div className="flex-1 overflow-hidden flex">
          {/* Left: Pane List */}
          <div className="w-64 border-r border-gray-200 overflow-y-auto">
            <div className="p-4 border-b border-gray-200">
              <label className="flex items-center space-x-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={config.autoStart}
                  onChange={(e) => setConfig({ ...config, autoStart: e.target.checked })}
                  className="rounded"
                />
                <span className="text-sm font-medium text-gray-700">Enable Auto-Start</span>
              </label>
            </div>

            <div className="p-4">
              <h3 className="text-sm font-semibold text-gray-700 mb-2">Panes</h3>
              <div className="space-y-2">
                {currentWorkspace?.layout.panes.map((pane, index) => {
                  const paneConfig = config.panes[pane.id];
                  const isSelected = selectedPaneId === pane.id;

                  return (
                    <button
                      key={pane.id}
                      onClick={() => setSelectedPaneId(pane.id)}
                      className={`w-full text-left px-3 py-2 rounded transition-colors ${
                        isSelected
                          ? 'bg-accent-primary text-white'
                          : 'bg-gray-50 text-gray-700 hover:bg-gray-100'
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className="text-sm font-medium">
                          {paneConfig?.name || `Pane ${index + 1}`}
                        </span>
                        {paneConfig?.enabled && (
                          <span className="text-xs">⚡</span>
                        )}
                      </div>
                      <div className="text-xs opacity-75 mt-1">
                        {paneConfig?.type || 'local'}
                      </div>
                    </button>
                  );
                })}
              </div>
            </div>
          </div>

          {/* Right: Configuration Form */}
          <div className="flex-1 overflow-y-auto p-6">
            {!selectedPaneId && (
              <div className="text-center py-12 text-gray-400">
                Select a pane to configure
              </div>
            )}

            {selectedPaneId && selectedConfig && (
              <div className="space-y-6">
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

                {/* Basic Settings */}
                <div>
                  <h3 className="text-lg font-semibold text-gray-800 mb-3">Basic Settings</h3>

                  <div className="space-y-4">
                    {/* Enabled */}
                    <label className="flex items-center space-x-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={selectedConfig.enabled}
                        onChange={(e) =>
                          updatePaneConfig(selectedPaneId, { enabled: e.target.checked })
                        }
                        className="rounded"
                      />
                      <span className="text-sm font-medium text-gray-700">
                        Enable auto-start for this pane
                      </span>
                    </label>

                    {/* Name */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Session Name
                      </label>
                      <input
                        type="text"
                        value={selectedConfig.name || ''}
                        onChange={(e) => updatePaneConfig(selectedPaneId, { name: e.target.value })}
                        placeholder="e.g., Dev Server"
                        className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                      />
                    </div>

                    {/* Session Type */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Session Type
                      </label>
                      <select
                        value={selectedConfig.type}
                        onChange={(e) =>
                          updatePaneConfig(selectedPaneId, { type: e.target.value as SessionType })
                        }
                        className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                      >
                        <option value="local">Local Terminal</option>
                        <option value="ssh">SSH Connection</option>
                      </select>
                    </div>
                  </div>
                </div>

                {/* Local Session Settings */}
                {selectedConfig.type === 'local' && (
                  <div>
                    <h3 className="text-lg font-semibold text-gray-800 mb-3">Local Session</h3>

                    <div className="space-y-4">
                      {/* Command */}
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Startup Command
                        </label>
                        <input
                          type="text"
                          value={selectedConfig.command || ''}
                          onChange={(e) =>
                            updatePaneConfig(selectedPaneId, { command: e.target.value })
                          }
                          placeholder="e.g., npm run dev"
                          className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                        />
                        <p className="text-xs text-gray-500 mt-1">
                          Command to run when session starts (optional)
                        </p>
                      </div>

                      {/* Working Directory */}
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Working Directory
                        </label>
                        <input
                          type="text"
                          value={selectedConfig.cwd || ''}
                          onChange={(e) => updatePaneConfig(selectedPaneId, { cwd: e.target.value })}
                          placeholder="e.g., ~/projects/myapp"
                          className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                        />
                      </div>

                      {/* Environment Variables */}
                      <div>
                        <div className="flex items-center justify-between mb-2">
                          <label className="text-sm font-medium text-gray-700">
                            Environment Variables
                          </label>
                          <button
                            onClick={addEnvVar}
                            className="text-xs px-2 py-1 bg-gray-100 hover:bg-gray-200 rounded transition-colors"
                          >
                            + Add
                          </button>
                        </div>

                        {selectedConfig.env && Object.keys(selectedConfig.env).length > 0 ? (
                          <div className="space-y-2">
                            {Object.entries(selectedConfig.env).map(([key, value]) => (
                              <div
                                key={key}
                                className="flex items-center gap-2 px-3 py-2 bg-gray-50 rounded"
                              >
                                <code className="flex-1 text-sm font-mono">
                                  {key}={value}
                                </code>
                                <button
                                  onClick={() => removeEnvVar(key)}
                                  className="text-red-500 hover:text-red-700 text-sm"
                                >
                                  ×
                                </button>
                              </div>
                            ))}
                          </div>
                        ) : (
                          <p className="text-xs text-gray-500 italic">No environment variables</p>
                        )}
                      </div>
                    </div>
                  </div>
                )}

                {/* SSH Session Settings */}
                {selectedConfig.type === 'ssh' && (
                  <div>
                    <h3 className="text-lg font-semibold text-gray-800 mb-3">SSH Connection</h3>

                    <div className="space-y-4">
                      {/* Host */}
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Host <span className="text-red-500">*</span>
                        </label>
                        <input
                          type="text"
                          value={selectedConfig.host || ''}
                          onChange={(e) => updatePaneConfig(selectedPaneId, { host: e.target.value })}
                          placeholder="e.g., example.com"
                          className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                        />
                      </div>

                      {/* Port */}
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">Port</label>
                        <input
                          type="number"
                          value={selectedConfig.port || 22}
                          onChange={(e) =>
                            updatePaneConfig(selectedPaneId, { port: parseInt(e.target.value) })
                          }
                          min="1"
                          max="65535"
                          className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                        />
                      </div>

                      {/* Username */}
                      <div>
                        <label className="block text-sm font-medium text-gray-700 mb-1">
                          Username
                        </label>
                        <input
                          type="text"
                          value={selectedConfig.username || ''}
                          onChange={(e) =>
                            updatePaneConfig(selectedPaneId, { username: e.target.value })
                          }
                          placeholder="e.g., user"
                          className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                        />
                      </div>
                    </div>
                  </div>
                )}

                {/* Advanced Options */}
                <div>
                  <h3 className="text-lg font-semibold text-gray-800 mb-3">Advanced Options</h3>

                  <div className="space-y-4">
                    {/* Startup Delay */}
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">
                        Startup Delay (ms)
                      </label>
                      <input
                        type="number"
                        value={selectedConfig.startupDelay || 0}
                        onChange={(e) =>
                          updatePaneConfig(selectedPaneId, {
                            startupDelay: parseInt(e.target.value),
                          })
                        }
                        min="0"
                        className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                      />
                      <p className="text-xs text-gray-500 mt-1">
                        Delay before starting this session (in milliseconds)
                      </p>
                    </div>

                    {/* Auto Reconnect */}
                    <label className="flex items-center space-x-2 cursor-pointer">
                      <input
                        type="checkbox"
                        checked={selectedConfig.autoReconnect || false}
                        onChange={(e) =>
                          updatePaneConfig(selectedPaneId, { autoReconnect: e.target.checked })
                        }
                        className="rounded"
                      />
                      <span className="text-sm text-gray-700">Auto-reconnect on disconnect</span>
                    </label>
                  </div>
                </div>

                {/* Example Configurations */}
                <div>
                  <button
                    onClick={() => setShowExamples(!showExamples)}
                    className="text-sm text-accent-primary hover:text-accent-primary-dark"
                  >
                    {showExamples ? '▼' : '▶'} Show Example Configurations
                  </button>

                  {showExamples && (
                    <div className="mt-3 space-y-2">
                      {Object.entries(sessionExamples).map(([key, example]) => (
                        <button
                          key={key}
                          onClick={() => applyExample(key as keyof typeof sessionExamples)}
                          className="w-full text-left px-3 py-2 border border-gray-200 rounded hover:border-accent-primary hover:bg-accent-primary hover:bg-opacity-5 transition-colors"
                        >
                          <div className="text-sm font-medium text-gray-700">
                            {example.name}
                          </div>
                          {'command' in example && example.command && (
                            <code className="text-xs text-gray-500">{example.command}</code>
                          )}
                        </button>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 flex justify-between items-center">
          <div className="text-sm text-gray-500">
            💡 Sessions will start automatically when you load this workspace
          </div>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded transition-colors"
            >
              Cancel
            </button>
            <button
              onClick={handleSave}
              disabled={saving || !currentWorkspace}
              className="px-6 py-2 bg-accent-primary text-white rounded hover:bg-accent-primary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {saving ? 'Saving...' : 'Save Configuration'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
