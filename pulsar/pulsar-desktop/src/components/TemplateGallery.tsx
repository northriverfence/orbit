/**
 * Template Gallery Component
 *
 * Browse and select from pre-built workspace templates
 */

import { useState } from 'react';
import { workspaceTemplates, type WorkspaceTemplate } from '../lib/workspaceTemplates';
import { useWorkspace } from '../lib/WorkspaceManager';

interface TemplateGalleryProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function TemplateGallery({ isOpen, onClose }: TemplateGalleryProps) {
  const { createWorkspace } = useWorkspace();
  const [selectedTemplate, setSelectedTemplate] = useState<WorkspaceTemplate | null>(null);
  const [workspaceName, setWorkspaceName] = useState('');
  const [searchQuery, setSearchQuery] = useState('');
  const [creating, setCreating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Filter templates based on search
  const filteredTemplates = searchQuery
    ? workspaceTemplates.filter(
        (t) =>
          t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
          t.tags.some((tag) => tag.toLowerCase().includes(searchQuery.toLowerCase()))
      )
    : workspaceTemplates;

  const handleTemplateSelect = (template: WorkspaceTemplate) => {
    setSelectedTemplate(template);
    setWorkspaceName(template.name);
    setError(null);
  };

  const handleCreate = async () => {
    if (!selectedTemplate || !workspaceName.trim()) {
      setError('Please select a template and enter a workspace name');
      return;
    }

    setCreating(true);
    setError(null);

    try {
      await createWorkspace({
        name: workspaceName.trim(),
        description: selectedTemplate.description,
        icon: selectedTemplate.icon,
        layout: selectedTemplate.layout,
        is_template: false,
        tags: selectedTemplate.tags,
      });

      // Reset and close
      setSelectedTemplate(null);
      setWorkspaceName('');
      setSearchQuery('');
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to create workspace');
    } finally {
      setCreating(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-4xl w-full mx-4 max-h-[90vh] flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-200 flex items-center justify-between">
          <div>
            <h2 className="text-xl font-semibold text-gray-800">Workspace Templates</h2>
            <p className="text-sm text-gray-500 mt-1">Choose a pre-built layout to get started</p>
          </div>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 text-2xl"
          >
            ×
          </button>
        </div>

        {/* Search */}
        <div className="px-6 py-4 border-b border-gray-200">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search templates..."
            className="w-full px-4 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
          />
        </div>

        {/* Content */}
        <div className="flex-1 overflow-hidden flex">
          {/* Template List */}
          <div className="w-1/2 border-r border-gray-200 overflow-y-auto p-4">
            <div className="grid gap-3">
              {filteredTemplates.map((template) => (
                <button
                  key={template.id}
                  onClick={() => handleTemplateSelect(template)}
                  className={`text-left p-4 rounded border-2 transition-all ${
                    selectedTemplate?.id === template.id
                      ? 'border-accent-primary bg-accent-primary bg-opacity-5'
                      : 'border-gray-200 hover:border-gray-300 hover:bg-gray-50'
                  }`}
                >
                  <div className="flex items-start gap-3">
                    <span className="text-3xl">{template.icon}</span>
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-gray-800">{template.name}</h3>
                      <p className="text-sm text-gray-600 mt-1">{template.description}</p>
                      <div className="flex gap-1 mt-2 flex-wrap">
                        {template.tags.map((tag) => (
                          <span
                            key={tag}
                            className="text-xs px-2 py-0.5 bg-gray-100 text-gray-600 rounded"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    </div>
                  </div>
                </button>
              ))}
            </div>

            {filteredTemplates.length === 0 && (
              <div className="text-center py-8 text-gray-500">
                No templates found matching "{searchQuery}"
              </div>
            )}
          </div>

          {/* Preview & Create */}
          <div className="w-1/2 overflow-y-auto p-6">
            {selectedTemplate ? (
              <div className="space-y-4">
                <div className="text-center">
                  <span className="text-6xl">{selectedTemplate.icon}</span>
                  <h3 className="text-2xl font-semibold text-gray-800 mt-3">
                    {selectedTemplate.name}
                  </h3>
                  <p className="text-gray-600 mt-2">{selectedTemplate.description}</p>
                </div>

                {/* Layout Preview */}
                <div className="bg-gray-50 rounded p-4">
                  <h4 className="text-sm font-medium text-gray-700 mb-2">Layout Preview</h4>
                  <pre className="font-mono text-sm text-gray-600 whitespace-pre">
                    {selectedTemplate.preview}
                  </pre>
                </div>

                {/* Tags */}
                <div>
                  <h4 className="text-sm font-medium text-gray-700 mb-2">Tags</h4>
                  <div className="flex gap-2 flex-wrap">
                    {selectedTemplate.tags.map((tag) => (
                      <span
                        key={tag}
                        className="px-3 py-1 bg-gray-100 text-gray-700 rounded text-sm"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>

                {/* Workspace Name */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    Workspace Name *
                  </label>
                  <input
                    type="text"
                    value={workspaceName}
                    onChange={(e) => setWorkspaceName(e.target.value)}
                    placeholder="Enter workspace name..."
                    className="w-full px-4 py-2 border border-gray-300 rounded focus:outline-none focus:ring-2 focus:ring-accent-primary"
                  />
                </div>

                {/* Error */}
                {error && (
                  <div className="bg-red-50 text-red-700 px-4 py-2 rounded text-sm">
                    {error}
                  </div>
                )}

                {/* Create Button */}
                <button
                  onClick={handleCreate}
                  disabled={creating || !workspaceName.trim()}
                  className="w-full px-6 py-3 bg-accent-primary text-white rounded hover:bg-accent-primary-dark transition-colors disabled:opacity-50 disabled:cursor-not-allowed font-medium"
                >
                  {creating ? 'Creating...' : 'Create Workspace'}
                </button>
              </div>
            ) : (
              <div className="flex items-center justify-center h-full text-gray-400">
                <div className="text-center">
                  <span className="text-6xl">📁</span>
                  <p className="mt-4">Select a template to preview</p>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
