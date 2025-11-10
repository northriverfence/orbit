/**
 * Sidebar Component (Updated with Workspace Integration)
 *
 * Displays workspaces, servers, and other navigation sections
 */

import { useState } from 'react';
import { SidebarSection } from '../App';
import { useWorkspace } from '../lib/WorkspaceManager';
import WorkspaceDialog from './WorkspaceDialog';

interface SidebarProps {
  expandedSection: SidebarSection;
  onSectionToggle: (section: SidebarSection) => void;
}

interface SidebarSectionData {
  id: SidebarSection;
  label: string;
  icon: string;
  badge?: number;
}

export default function SidebarNew({ expandedSection, onSectionToggle }: SidebarProps) {
  const {
    workspaces,
    currentWorkspace,
    switchWorkspace,
    deleteWorkspace,
    loading: workspacesLoading,
  } = useWorkspace();

  const [dialogOpen, setDialogOpen] = useState(false);
  const [contextMenuWorkspaceId, setContextMenuWorkspaceId] = useState<string | null>(null);

  const sidebarSections: SidebarSectionData[] = [
    {
      id: 'workspaces',
      label: 'Workspaces',
      icon: '📁',
      badge: workspaces.length,
    },
    {
      id: 'servers',
      label: 'Servers',
      icon: '🖥️',
      badge: 2,
    },
    {
      id: 'file-transfer',
      label: 'File Transfer',
      icon: '📤',
    },
    {
      id: 'vaults',
      label: 'Vaults',
      icon: '🔐',
    },
    {
      id: 'settings',
      label: 'Settings',
      icon: '⚙️',
    },
  ];

  const handleToggle = (section: SidebarSection) => {
    onSectionToggle(expandedSection === section ? null : section);
  };

  const handleWorkspaceClick = async (workspaceId: string) => {
    try {
      await switchWorkspace(workspaceId);
    } catch (error) {
      console.error('Failed to switch workspace:', error);
    }
  };

  const handleDeleteWorkspace = async (workspaceId: string) => {
    if (confirm('Are you sure you want to delete this workspace?')) {
      try {
        await deleteWorkspace(workspaceId);
        setContextMenuWorkspaceId(null);
      } catch (error) {
        console.error('Failed to delete workspace:', error);
        alert('Failed to delete workspace');
      }
    }
  };

  const renderWorkspaceSection = () => {
    if (expandedSection !== 'workspaces') return null;

    return (
      <div className="bg-white">
        {/* Loading state */}
        {workspacesLoading && (
          <div className="px-6 py-4 text-sm text-gray-500 text-center">
            Loading workspaces...
          </div>
        )}

        {/* Empty state */}
        {!workspacesLoading && workspaces.length === 0 && (
          <div className="px-6 py-4 text-sm text-gray-500 text-center">
            No workspaces yet
          </div>
        )}

        {/* Workspace list */}
        {!workspacesLoading &&
          workspaces.map((workspace) => (
            <button
              key={workspace.id}
              onClick={() => handleWorkspaceClick(workspace.id)}
              className={`w-full px-6 py-2.5 text-left text-sm transition-colors flex items-center justify-between group ${
                currentWorkspace?.id === workspace.id
                  ? 'bg-accent-primary bg-opacity-10 text-accent-primary font-medium'
                  : 'text-gray-600 hover:bg-gray-50'
              }`}
            >
              <div className="flex items-center gap-2 flex-1 min-w-0">
                <span className="text-base">{workspace.icon || '📁'}</span>
                <span className="truncate">{workspace.name}</span>
              </div>

              {/* Context menu trigger */}
              <button
                onClick={(e) => {
                  e.stopPropagation();
                  setContextMenuWorkspaceId(
                    contextMenuWorkspaceId === workspace.id ? null : workspace.id
                  );
                }}
                className="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-gray-600 px-1"
              >
                ⋮
              </button>

              {/* Context menu */}
              {contextMenuWorkspaceId === workspace.id && (
                <div className="absolute right-6 mt-8 bg-white border border-gray-200 rounded shadow-lg py-1 z-10">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      // TODO: Open edit dialog
                      setContextMenuWorkspaceId(null);
                    }}
                    className="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-50"
                  >
                    Edit
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      // TODO: Duplicate workspace
                      setContextMenuWorkspaceId(null);
                    }}
                    className="w-full px-4 py-2 text-left text-sm text-gray-700 hover:bg-gray-50"
                  >
                    Duplicate
                  </button>
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleDeleteWorkspace(workspace.id);
                    }}
                    className="w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50"
                  >
                    Delete
                  </button>
                </div>
              )}
            </button>
          ))}

        {/* Create new workspace button */}
        <button
          onClick={() => setDialogOpen(true)}
          className="w-full px-6 py-2.5 text-left text-sm text-accent-primary hover:bg-accent-primary hover:bg-opacity-5 transition-colors flex items-center gap-2"
        >
          <span>+</span>
          <span>New Workspace</span>
        </button>
      </div>
    );
  };

  const renderServersSection = () => {
    if (expandedSection !== 'servers') return null;

    return (
      <div className="bg-white">
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Production (prod.example.com)
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          AWS Instance (ec2-54-123...)
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-accent-primary hover:bg-accent-primary hover:bg-opacity-5 transition-colors flex items-center gap-2">
          <span>+</span>
          <span>Add Server</span>
        </button>
      </div>
    );
  };

  const renderFileTransferSection = () => {
    if (expandedSection !== 'file-transfer') return null;

    return (
      <div className="bg-white">
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Quick Transfer
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Recent Files
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Scheduled
        </button>
      </div>
    );
  };

  const renderVaultsSection = () => {
    if (expandedSection !== 'vaults') return null;

    return (
      <div className="bg-white">
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Credentials
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          SSH Keys
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Certificates
        </button>
      </div>
    );
  };

  const renderSettingsSection = () => {
    if (expandedSection !== 'settings') return null;

    return (
      <div className="bg-white">
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Appearance
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Connections
        </button>
        <button className="w-full px-6 py-2.5 text-left text-sm text-gray-600 hover:bg-gray-50 transition-colors">
          Security
        </button>
      </div>
    );
  };

  return (
    <>
      <div className="w-64 bg-sidebar-bg border-r border-gray-200 flex flex-col">
        {/* Header */}
        <div className="h-14 flex items-center px-4 border-b border-gray-200">
          <h1 className="text-xl font-bold text-gray-800">Pulsar</h1>
        </div>

        {/* Sections */}
        <div className="flex-1 overflow-y-auto">
          {sidebarSections.map((section) => (
            <div key={section.id} className="border-b border-gray-200">
              {/* Section Header */}
              <button
                onClick={() => handleToggle(section.id)}
                className="w-full px-4 py-3 flex items-center justify-between hover:bg-sidebar-hover transition-colors"
              >
                <div className="flex items-center gap-2">
                  <span className="text-lg">{section.icon}</span>
                  <span className="font-medium text-gray-700">{section.label}</span>
                </div>
                <div className="flex items-center gap-2">
                  {section.badge !== undefined && section.badge > 0 && (
                    <span className="bg-accent-secondary text-white text-xs font-semibold px-2 py-0.5 rounded-full">
                      {section.badge}
                    </span>
                  )}
                  <span
                    className={`text-gray-500 transition-transform ${
                      expandedSection === section.id ? 'rotate-90' : ''
                    }`}
                  >
                    ›
                  </span>
                </div>
              </button>

              {/* Section Content */}
              {section.id === 'workspaces' && renderWorkspaceSection()}
              {section.id === 'servers' && renderServersSection()}
              {section.id === 'file-transfer' && renderFileTransferSection()}
              {section.id === 'vaults' && renderVaultsSection()}
              {section.id === 'settings' && renderSettingsSection()}
            </div>
          ))}
        </div>

        {/* Footer */}
        <div className="h-12 border-t border-gray-200 flex items-center px-4 text-xs text-gray-500">
          v0.1.0
        </div>
      </div>

      {/* Workspace Dialog */}
      <WorkspaceDialog isOpen={dialogOpen} onClose={() => setDialogOpen(false)} />
    </>
  );
}
