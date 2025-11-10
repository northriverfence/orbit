import { useState, useEffect, useCallback, useRef } from 'react';
import VaultClient from '../lib/vaultClient';
import LoadingSpinner from './LoadingSpinner';
import ErrorAlert from './ErrorAlert';
import { useToast } from './ToastContainer';
import { useArrowNavigation } from '../hooks/useArrowNavigation';
import type { CredentialSummary, CredentialType } from '../types/vault';

interface VaultCredentialListProps {
  onSelect?: (credential: CredentialSummary) => void;
  onAdd?: () => void;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

export default function VaultCredentialList({
  onSelect,
  onAdd,
  onView,
  onEdit,
  onDelete,
}: VaultCredentialListProps) {
  const listRef = useRef<HTMLDivElement>(null);
  const [credentials, setCredentials] = useState<CredentialSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filterType, setFilterType] = useState<CredentialType | 'all'>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const toast = useToast();

  const loadCredentials = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      let result: CredentialSummary[];

      if (filterType === 'all') {
        result = await VaultClient.listCredentials();
      } else {
        result = await VaultClient.listCredentialsByType(filterType);
      }

      setCredentials(result);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Failed to load credentials';
      setError(errorMessage);
      toast.showError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [filterType, toast]);

  useEffect(() => {
    loadCredentials();
  }, [loadCredentials]);

  const handleDelete = async (id: string, name: string) => {
    if (!confirm(`Are you sure you want to delete "${name}"? This cannot be undone.`)) {
      return;
    }

    try {
      await VaultClient.deleteCredential(id);
      await loadCredentials();
      if (onDelete) onDelete(id);
      toast.showSuccess(`Deleted "${name}" successfully`);
    } catch (err) {
      const errorMessage = `Failed to delete credential: ${err}`;
      toast.showError(errorMessage);
      setError(errorMessage);
    }
  };

  const filteredCredentials = credentials.filter((cred) => {
    if (!searchQuery) return true;

    const query = searchQuery.toLowerCase();
    return (
      cred.name.toLowerCase().includes(query) ||
      cred.tags.some((tag) => tag.toLowerCase().includes(query)) ||
      (cred.username && cred.username.toLowerCase().includes(query)) ||
      (cred.host_pattern && cred.host_pattern.toLowerCase().includes(query))
    );
  });

  // Arrow navigation for credential list
  const { activeIndex } = useArrowNavigation({
    containerRef: listRef,
    enabled: !loading && !error && filteredCredentials.length > 0,
    onSelect: (index) => {
      if (onSelect && filteredCredentials[index]) {
        onSelect(filteredCredentials[index]);
      }
    },
    loop: true,
  });

  const getTypeIcon = (type: CredentialType) => {
    switch (type) {
      case 'ssh_key':
        return '🔑';
      case 'password':
        return '🔐';
      case 'certificate':
        return '📜';
      default:
        return '📄';
    }
  };

  const getTypeLabel = (type: CredentialType) => {
    switch (type) {
      case 'ssh_key':
        return 'SSH Key';
      case 'password':
        return 'Password';
      case 'certificate':
        return 'Certificate';
      default:
        return type;
    }
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  return (
    <div className="h-full flex flex-col bg-gray-50">
      {/* Header with filters */}
      <div className="flex-shrink-0 bg-white border-b border-gray-200 p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold text-gray-900">Credentials</h2>
          {onAdd && (
            <button
              onClick={onAdd}
              className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium"
            >
              + Add Credential
            </button>
          )}
        </div>

        {/* Search */}
        <div className="mb-3">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search credentials..."
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>

        {/* Type filter */}
        <div className="flex space-x-2">
          <button
            onClick={() => setFilterType('all')}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              filterType === 'all'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            All
          </button>
          <button
            onClick={() => setFilterType('ssh_key')}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              filterType === 'ssh_key'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            🔑 SSH Keys
          </button>
          <button
            onClick={() => setFilterType('password')}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              filterType === 'password'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            🔐 Passwords
          </button>
          <button
            onClick={() => setFilterType('certificate')}
            className={`px-3 py-1.5 text-sm font-medium rounded-md transition-colors ${
              filterType === 'certificate'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
            }`}
          >
            📜 Certificates
          </button>
        </div>
      </div>

      {/* Credential list */}
      <div className="flex-1 overflow-y-auto p-4">
        {loading ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <LoadingSpinner size="lg" />
              <p className="mt-4 text-sm font-medium text-gray-600">Loading credentials...</p>
            </div>
          </div>
        ) : error ? (
          <div className="flex items-center justify-center h-full p-4">
            <ErrorAlert
              title="Failed to load credentials"
              message={error}
              type="error"
              onRetry={loadCredentials}
              onDismiss={() => setError(null)}
            />
          </div>
        ) : filteredCredentials.length === 0 ? (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-500">
              <div className="text-6xl mb-4">
                {searchQuery ? '🔍' : filterType === 'all' ? '🗄️' : getTypeIcon(filterType as CredentialType)}
              </div>
              <p className="text-lg font-medium mb-2">
                {searchQuery ? 'No matching credentials' : 'No credentials yet'}
              </p>
              <p className="text-sm">
                {searchQuery
                  ? 'Try a different search query'
                  : onAdd
                  ? 'Click "Add Credential" to create your first credential'
                  : 'No credentials found'}
              </p>
            </div>
          </div>
        ) : (
          <div ref={listRef} className="grid gap-3" role="listbox">
            {filteredCredentials.map((cred, index) => (
              <div
                key={cred.id}
                role="option"
                aria-selected={index === activeIndex}
                tabIndex={0}
                className={`bg-white rounded-lg border border-gray-200 p-4 hover:shadow-md transition-all hover-lift animate-fadeIn ${
                  index === activeIndex ? 'ring-2 ring-blue-500' : ''
                }`}
              >
                <div className="flex items-start justify-between">
                  <div
                    className={`flex-1 ${onSelect ? 'cursor-pointer' : ''}`}
                    onClick={() => onSelect && onSelect(cred)}
                  >
                    <div className="flex items-center space-x-2 mb-2">
                      <span className="text-2xl">{getTypeIcon(cred.credential_type)}</span>
                      <div>
                        <h3 className="font-semibold text-gray-900">{cred.name}</h3>
                        <p className="text-xs text-gray-500">{getTypeLabel(cred.credential_type)}</p>
                      </div>
                    </div>

                    {cred.username && (
                      <div className="text-sm text-gray-600 mb-1">
                        <span className="font-medium">Username:</span> {cred.username}
                      </div>
                    )}

                    {cred.host_pattern && (
                      <div className="text-sm text-gray-600 mb-1">
                        <span className="font-medium">Host:</span> {cred.host_pattern}
                      </div>
                    )}

                    {cred.tags.length > 0 && (
                      <div className="flex flex-wrap gap-1 mt-2">
                        {cred.tags.map((tag) => (
                          <span
                            key={tag}
                            className="px-2 py-0.5 text-xs bg-blue-100 text-blue-700 rounded-full"
                          >
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}

                    <div className="text-xs text-gray-400 mt-2">
                      Created {formatDate(cred.created_at)}
                    </div>
                  </div>

                  <div className="flex space-x-1 ml-4">
                    {onView && (
                      <button
                        onClick={() => onView(cred.id)}
                        className="px-2 py-1 text-sm text-blue-600 hover:bg-blue-50 rounded transition-colors"
                        title="View"
                      >
                        👁️
                      </button>
                    )}
                    {onEdit && (
                      <button
                        onClick={() => onEdit(cred.id)}
                        className="px-2 py-1 text-sm text-gray-600 hover:bg-gray-100 rounded transition-colors"
                        title="Edit"
                      >
                        ✏️
                      </button>
                    )}
                    {onDelete && (
                      <button
                        onClick={() => handleDelete(cred.id, cred.name)}
                        className="px-2 py-1 text-sm text-red-600 hover:bg-red-50 rounded transition-colors"
                        title="Delete"
                      >
                        🗑️
                      </button>
                    )}
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Footer stats */}
      {!loading && !error && filteredCredentials.length > 0 && (
        <div className="flex-shrink-0 bg-white border-t border-gray-200 px-4 py-2">
          <p className="text-sm text-gray-600">
            Showing {filteredCredentials.length} of {credentials.length} credential
            {credentials.length !== 1 ? 's' : ''}
          </p>
        </div>
      )}
    </div>
  );
}
