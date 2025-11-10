/**
 * Workspace Dialog
 *
 * Modal dialog for creating and editing workspaces
 */

import React, { useState, useEffect } from 'react';
import { useWorkspace } from '../lib/WorkspaceManager';
import type { Workspace, WorkspaceLayout } from '../types/workspace';
import { createDefaultLayout, createHorizontalSplit, createVerticalSplit } from '../types/workspace';

interface WorkspaceDialogProps {
  isOpen: boolean;
  onClose: () => void;
  workspace?: Workspace; // If provided, edit mode
  template?: Workspace; // If provided, create from template
}

const layoutTemplates = [
  { id: 'single', name: 'Single Pane', icon: '□', create: createDefaultLayout },
  { id: 'hsplit', name: 'Horizontal Split', icon: '⬒', create: createHorizontalSplit },
  { id: 'vsplit', name: 'Vertical Split', icon: '⬓', create: createVerticalSplit },
];

export default function WorkspaceDialog({ isOpen, onClose, workspace, template }: WorkspaceDialogProps) {
  const { createWorkspace, updateWorkspace } = useWorkspace();

  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [icon, setIcon] = useState('📁');
  const [selectedLayout, setSelectedLayout] = useState('single');
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const isEditMode = !!workspace;
  const isFromTemplate = !!template;

  // Initialize form when dialog opens or workspace changes
  useEffect(() => {
    if (isOpen) {
      if (workspace) {
        // Edit mode
        setName(workspace.name);
        setDescription(workspace.description || '');
        setIcon(workspace.icon || '📁');
        setTags(workspace.tags || []);
        setSelectedLayout('custom');
      } else if (template) {
        // Create from template
        setName(`${template.name} Copy`);
        setDescription(template.description || '');
        setIcon(template.icon || '📁');
        setTags([]);
        setSelectedLayout('custom');
      } else {
        // Create new
        setName('');
        setDescription('');
        setIcon('📁');
        setTags([]);
        setSelectedLayout('single');
      }
      setTagInput('');
      setError(null);
    }
  }, [isOpen, workspace, template]);

  const handleAddTag = () => {
    const trimmed = tagInput.trim();
    if (trimmed && !tags.includes(trimmed)) {
      setTags([...tags, trimmed]);
      setTagInput('');
    }
  };

  const handleRemoveTag = (tag: string) => {
    setTags(tags.filter(t => t !== tag));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError('Workspace name is required');
      return;
    }

    setSaving(true);

    try {
      if (isEditMode && workspace) {
        // Update existing workspace
        await updateWorkspace(workspace.id, {
          name: name.trim(),
          description: description.trim() || undefined,
          icon: icon || undefined,
          tags: tags.length > 0 ? tags : undefined,
        });
      } else {
        // Create new workspace
        let layout: WorkspaceLayout;

        if (template) {
          // Use template layout
          layout = template.layout;
        } else if (selectedLayout === 'custom' && workspace) {
          // Keep existing layout (shouldn't happen, but safeguard)
          layout = workspace.layout;
        } else {
          // Use selected template
          const layoutTemplate = layoutTemplates.find(lt => lt.id === selectedLayout);
          layout = layoutTemplate ? layoutTemplate.create() : createDefaultLayout();
        }

        await createWorkspace({
          name: name.trim(),
          description: description.trim() || undefined,
          icon: icon || undefined,
          layout,
          is_template: false,
          tags: tags.length > 0 ? tags : undefined,
        });
      }

      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to save workspace');
    } finally {
      setSaving(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200">
          <h2 className="text-xl font-semibold text-gray-800">
            {isEditMode ? 'Edit Workspace' : isFromTemplate ? 'Create from Template' : 'Create Workspace'}
          </h2>
        </div>

        {/* Form */}
        <form onSubmit={handleSubmit} className="px-6 py-4 space-y-4">
          {/* Error message */}
          {error && (
            <div className="bg-red-50 text-red-700 px-4 py-2 rounded text-sm">
              {error}
            </div>
          )}

          {/* Name */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Name *
            </label>
            <input
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
              placeholder="My Workspace"
              required
              autoFocus
            />
          </div>

          {/* Icon */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Icon
            </label>
            <div className="flex gap-2">
              {['📁', '💼', '🚀', '⚙️', '🔧', '💻', '🌐', '🔥'].map((emoji) => (
                <button
                  key={emoji}
                  type="button"
                  onClick={() => setIcon(emoji)}
                  className={`text-2xl p-2 rounded border-2 transition-colors ${
                    icon === emoji
                      ? 'border-accent-primary bg-accent-primary bg-opacity-10'
                      : 'border-gray-200 hover:border-gray-300'
                  }`}
                >
                  {emoji}
                </button>
              ))}
            </div>
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Description
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary resize-none"
              rows={3}
              placeholder="Optional description..."
            />
          </div>

          {/* Layout Template (only for create mode) */}
          {!isEditMode && !isFromTemplate && (
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">
                Layout
              </label>
              <div className="grid grid-cols-3 gap-2">
                {layoutTemplates.map((lt) => (
                  <button
                    key={lt.id}
                    type="button"
                    onClick={() => setSelectedLayout(lt.id)}
                    className={`p-3 rounded border-2 transition-colors flex flex-col items-center gap-1 ${
                      selectedLayout === lt.id
                        ? 'border-accent-primary bg-accent-primary bg-opacity-10'
                        : 'border-gray-200 hover:border-gray-300'
                    }`}
                  >
                    <span className="text-3xl">{lt.icon}</span>
                    <span className="text-xs text-gray-600">{lt.name}</span>
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Tags */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Tags
            </label>
            <div className="flex gap-2 mb-2 flex-wrap">
              {tags.map((tag) => (
                <span
                  key={tag}
                  className="inline-flex items-center gap-1 px-2 py-1 bg-gray-100 text-gray-700 text-sm rounded"
                >
                  {tag}
                  <button
                    type="button"
                    onClick={() => handleRemoveTag(tag)}
                    className="text-gray-500 hover:text-gray-700"
                  >
                    ×
                  </button>
                </span>
              ))}
            </div>
            <div className="flex gap-2">
              <input
                type="text"
                value={tagInput}
                onChange={(e) => setTagInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && (e.preventDefault(), handleAddTag())}
                className="flex-1 px-3 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                placeholder="Add tag..."
              />
              <button
                type="button"
                onClick={handleAddTag}
                className="px-4 py-2 bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
              >
                Add
              </button>
            </div>
          </div>
        </form>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-200 flex justify-end gap-3">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-gray-700 hover:bg-gray-100 rounded transition-colors"
            disabled={saving}
          >
            Cancel
          </button>
          <button
            onClick={handleSubmit}
            disabled={saving || !name.trim()}
            className="px-4 py-2 bg-accent-primary text-white rounded hover:bg-accent-primary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {saving ? 'Saving...' : isEditMode ? 'Update' : 'Create'}
          </button>
        </div>
      </div>
    </div>
  );
}
