/**
 * Visual Layout Editor
 *
 * Drag-and-drop workspace layout editor with split/merge controls
 */

import { useState, useCallback, useEffect, type ReactNode, memo } from 'react';
import { useWorkspace } from '../lib/WorkspaceManager';
import type { WorkspaceLayout, PaneConfig } from '../types/workspace';
import { useAccessibleModal } from '../lib/accessibility';
import { deepClone } from '../lib/utils';

interface VisualLayoutEditorProps {
  isOpen: boolean;
  onClose: () => void;
}

function VisualLayoutEditor({ isOpen, onClose }: VisualLayoutEditorProps) {
  const { currentWorkspace, updateWorkspace } = useWorkspace();

  const [layout, setLayout] = useState<WorkspaceLayout | null>(null);
  const [selectedPaneId, setSelectedPaneId] = useState<string | null>(null);
  const [hoveredPaneId, setHoveredPaneId] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [history, setHistory] = useState<WorkspaceLayout[]>([]);
  const [historyIndex, setHistoryIndex] = useState(-1);

  // Accessibility features
  const { dialogRef, titleId, descriptionId, ariaProps } = useAccessibleModal({
    isOpen,
    onClose,
    title: 'Visual Layout Editor',
    description: 'Edit workspace pane layout with visual controls',
  });

  // Load current workspace layout
  useEffect(() => {
    if (isOpen && currentWorkspace) {
      setLayout(deepClone(currentWorkspace.layout));
      setHistory([deepClone(currentWorkspace.layout)]);
      setHistoryIndex(0);
    }
  }, [isOpen, currentWorkspace]);

  // Add to history for undo/redo
  const addToHistory = useCallback((newLayout: WorkspaceLayout) => {
    setHistory((prev) => {
      const newHistory = prev.slice(0, historyIndex + 1);
      newHistory.push(deepClone(newLayout));
      return newHistory;
    });
    setHistoryIndex((prev) => prev + 1);
  }, [historyIndex]);

  // Undo
  const handleUndo = useCallback(() => {
    if (historyIndex > 0) {
      setHistoryIndex((prev) => prev - 1);
      setLayout(deepClone(history[historyIndex - 1]));
    }
  }, [historyIndex, history]);

  // Redo
  const handleRedo = useCallback(() => {
    if (historyIndex < history.length - 1) {
      setHistoryIndex((prev) => prev + 1);
      setLayout(deepClone(history[historyIndex + 1]));
    }
  }, [historyIndex, history]);

  // Find pane by ID
  const findPane = useCallback((paneId: string, panes: PaneConfig[]): PaneConfig | null => {
    for (const pane of panes) {
      if (pane.id === paneId) {
        return pane;
      }
      if (pane.children) {
        const found = findPane(paneId, pane.children);
        if (found) return found;
      }
    }
    return null;
  }, []);

  // Split pane
  const handleSplitPane = useCallback((paneId: string, direction: 'horizontal' | 'vertical') => {
    if (!layout) return;

    const newLayout = deepClone(layout);
    const pane = findPane(paneId, newLayout.panes);

    if (pane) {
      // Create two new child panes
      const child1: PaneConfig = {
        id: crypto.randomUUID(),
        size: 50,
        direction,
        min_size: 20,
      };

      const child2: PaneConfig = {
        id: crypto.randomUUID(),
        size: 50,
        direction,
        min_size: 20,
      };

      pane.children = [child1, child2];
      pane.direction = direction;

      setLayout(newLayout);
      addToHistory(newLayout);
      setSelectedPaneId(child1.id);
    }
  }, [layout, findPane, addToHistory]);

  // Remove pane (merge with sibling)
  const handleRemovePane = useCallback((paneId: string) => {
    if (!layout || layout.panes.length <= 1) {
      setMessage({ type: 'error', text: 'Cannot remove the last pane' });
      return;
    }

    const newLayout = deepClone(layout);

    // Find parent pane
    const findParent = (panes: PaneConfig[], targetId: string): PaneConfig | null => {
      for (const pane of panes) {
        if (pane.children) {
          const found = pane.children.find((child: PaneConfig) => child.id === targetId);
          if (found) return pane;

          const foundInChildren = findParent(pane.children, targetId);
          if (foundInChildren) return foundInChildren;
        }
      }
      return null;
    };

    const parent = findParent(newLayout.panes, paneId);

    if (parent && parent.children) {
      // Remove the pane from children
      parent.children = parent.children.filter((child) => child.id !== paneId);

      // If only one child remains, collapse it
      if (parent.children.length === 1) {
        const remainingChild = parent.children[0];
        Object.assign(parent, remainingChild);
      }
    } else {
      // Top-level pane - remove from layout
      newLayout.panes = newLayout.panes.filter((pane) => pane.id !== paneId);
    }

    setLayout(newLayout);
    addToHistory(newLayout);
    setSelectedPaneId(null);
  }, [layout, addToHistory]);

  // Resize pane
  const handleResizePane = useCallback((paneId: string, newSize: number) => {
    if (!layout) return;

    const newLayout = deepClone(layout);
    const pane = findPane(paneId, newLayout.panes);

    if (pane) {
      pane.size = Math.max(pane.min_size || 10, Math.min(90, newSize));
      setLayout(newLayout);
    }
  }, [layout, findPane]);

  // Commit resize (add to history)
  const handleResizeCommit = useCallback(() => {
    if (layout) {
      addToHistory(layout);
    }
  }, [layout, addToHistory]);

  // Save layout
  const handleSave = async () => {
    if (!currentWorkspace || !layout) return;

    setSaving(true);
    setMessage(null);

    try {
      await updateWorkspace(currentWorkspace.id, { layout });
      setMessage({ type: 'success', text: 'Layout saved successfully' });

      // Close after short delay
      setTimeout(() => {
        onClose();
      }, 1500);
    } catch (error) {
      setMessage({
        type: 'error',
        text: error instanceof Error ? error.message : 'Failed to save layout',
      });
    } finally {
      setSaving(false);
    }
  };

  // Render pane in preview
  const renderPane = (pane: PaneConfig, depth: number = 0): ReactNode => {
    const isSelected = selectedPaneId === pane.id;
    const isHovered = hoveredPaneId === pane.id;
    const hasChildren = pane.children && pane.children.length > 0;

    return (
      <div
        key={pane.id}
        className={`relative border-2 transition-all ${
          isSelected
            ? 'border-accent-primary bg-accent-primary bg-opacity-10'
            : isHovered
            ? 'border-accent-secondary bg-accent-secondary bg-opacity-5'
            : 'border-gray-300 bg-white'
        }`}
        style={{
          flex: `${pane.size} 1 0%`,
          minHeight: hasChildren ? 'auto' : '60px',
          display: 'flex',
          flexDirection: pane.direction === 'horizontal' ? 'column' : 'row',
        }}
        onClick={(e) => {
          e.stopPropagation();
          setSelectedPaneId(pane.id);
        }}
        onMouseEnter={() => setHoveredPaneId(pane.id)}
        onMouseLeave={() => setHoveredPaneId(null)}
      >
        {/* Pane Controls */}
        {!hasChildren && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center">
              <div className="text-xs text-gray-500 mb-2">Pane {pane.id.slice(0, 8)}</div>
              <div className="flex gap-1 justify-center">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleSplitPane(pane.id, 'horizontal');
                  }}
                  className="px-2 py-1 text-xs bg-accent-primary text-white rounded hover:bg-accent-primary-dark"
                  title="Split Horizontally"
                >
                  ⬌
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    handleSplitPane(pane.id, 'vertical');
                  }}
                  className="px-2 py-1 text-xs bg-accent-primary text-white rounded hover:bg-accent-primary-dark"
                  title="Split Vertically"
                >
                  ⬍
                </button>
                {depth > 0 && (
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRemovePane(pane.id);
                    }}
                    className="px-2 py-1 text-xs bg-red-500 text-white rounded hover:bg-red-600"
                    title="Remove Pane"
                  >
                    ×
                  </button>
                )}
              </div>
            </div>
          </div>
        )}

        {/* Children */}
        {hasChildren && (
          <>
            {pane.children!.map((child: PaneConfig) => renderPane(child, depth + 1))}
          </>
        )}

        {/* Resize Handle */}
        {isSelected && !hasChildren && (
          <div className="absolute bottom-0 right-0 p-1">
            <div className="text-xs text-gray-400">Size: {pane.size}%</div>
            <input
              type="range"
              min={pane.min_size || 10}
              max="90"
              value={pane.size}
              onChange={(e) => handleResizePane(pane.id, parseInt(e.target.value))}
              onMouseUp={handleResizeCommit}
              className="w-24"
              onClick={(e) => e.stopPropagation()}
            />
          </div>
        )}
      </div>
    );
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div
        ref={dialogRef}
        {...ariaProps}
        className="bg-white rounded-lg shadow-xl max-w-7xl w-full mx-4 max-h-[90vh] flex flex-col"
      >
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <div>
            <h2 id={titleId} className="text-xl font-semibold text-gray-800">
              Visual Layout Editor
            </h2>
            <p id={descriptionId} className="text-sm text-gray-500 mt-1">
              {currentWorkspace
                ? `Editing: ${currentWorkspace.name}`
                : 'No workspace selected'}
            </p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl"
            aria-label="Close dialog"
          >
            ×
          </button>
        </div>

        {/* Toolbar */}
        <div className="px-6 py-3 border-b border-gray-200 flex items-center justify-between">
          <div className="flex gap-2">
            <button
              onClick={handleUndo}
              disabled={historyIndex <= 0}
              className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              title="Undo (Ctrl+Z)"
            >
              ↶ Undo
            </button>
            <button
              onClick={handleRedo}
              disabled={historyIndex >= history.length - 1}
              className="px-3 py-1 text-sm border border-gray-300 rounded hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              title="Redo (Ctrl+Y)"
            >
              ↷ Redo
            </button>
          </div>

          <div className="text-sm text-gray-600">
            {selectedPaneId && (
              <span>Selected: {selectedPaneId.slice(0, 8)}</span>
            )}
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-hidden p-6">
          {message && (
            <div
              className={`mb-4 px-4 py-3 rounded ${
                message.type === 'success'
                  ? 'bg-green-50 text-green-700'
                  : 'bg-red-50 text-red-700'
              }`}
            >
              {message.text}
            </div>
          )}

          {!layout && (
            <div className="text-center py-12 text-gray-400">
              No layout loaded
            </div>
          )}

          {layout && (
            <div className="h-full border-2 border-gray-200 rounded bg-gray-50 p-4">
              <div className="h-full flex flex-col gap-2">
                {layout.panes.map((pane) => renderPane(pane))}
              </div>
            </div>
          )}
        </div>

        {/* Instructions */}
        <div className="px-6 py-3 bg-gray-50 border-t border-gray-200">
          <div className="text-xs text-gray-600">
            <strong>Instructions:</strong> Click a pane to select it. Use the split buttons (⬌ horizontal, ⬍ vertical) to divide a pane. Use the × button to remove a pane. Drag the slider to resize.
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 flex justify-between items-center">
          <div className="text-sm text-gray-500">
            {layout && `${layout.panes.length} top-level pane(s)`}
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
              disabled={saving || !layout}
              className="px-6 py-2 bg-accent-primary text-white rounded hover:bg-accent-primary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {saving ? 'Saving...' : 'Save Layout'}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

// Export memoized component for performance
export default memo(VisualLayoutEditor);
