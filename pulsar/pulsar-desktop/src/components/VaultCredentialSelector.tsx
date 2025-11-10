import { useState, useEffect } from 'react';
import VaultClient from '../lib/vaultClient';
import type { CredentialSummary } from '../types/vault';

interface VaultCredentialSelectorProps {
  isOpen: boolean;
  onClose: () => void;
  onSelect: (credential: CredentialSummary) => void;
  hostHint?: string;
}

export default function VaultCredentialSelector({
  isOpen,
  onClose,
  onSelect,
  hostHint,
}: VaultCredentialSelectorProps) {
  const [credentials, setCredentials] = useState<CredentialSummary[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState('');

  useEffect(() => {
    if (isOpen) {
      loadCredentials();
      // Pre-fill search with host hint
      if (hostHint) {
        setSearchQuery(hostHint);
      }
    }
  }, [isOpen, hostHint]);

  const loadCredentials = async () => {
    setLoading(true);
    setError(null);

    try {
      // Check if vault is unlocked
      const isUnlocked = await VaultClient.isUnlocked();
      if (!isUnlocked) {
        setError('Vault is locked. Please unlock it first.');
        setLoading(false);
        return;
      }

      // Load SSH keys and passwords only (most relevant for SSH connections)
      const [sshKeys, passwords] = await Promise.all([
        VaultClient.getSshKeys(),
        VaultClient.getPasswords(),
      ]);

      setCredentials([...sshKeys, ...passwords]);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load credentials');
    } finally {
      setLoading(false);
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

  const getTypeIcon = (type: string) => {
    return type === 'ssh_key' ? '🔑' : '🔐';
  };

  const formatDate = (timestamp: number) => {
    return new Date(timestamp).toLocaleDateString('en-US', {
      month: 'short',
      day: 'numeric',
    });
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl w-full max-w-2xl max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h2 className="text-2xl font-bold text-gray-900">🗄️ Select from Vault</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            ✕
          </button>
        </div>

        {/* Search */}
        <div className="p-4 border-b border-gray-200">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Search credentials..."
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            autoFocus
          />
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {loading ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-gray-500">
                <div className="text-4xl mb-2">⏳</div>
                <p>Loading credentials...</p>
              </div>
            </div>
          ) : error ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-red-600">
                <div className="text-4xl mb-2">⚠️</div>
                <p>{error}</p>
                <button
                  onClick={loadCredentials}
                  className="mt-4 px-4 py-2 bg-red-600 text-white rounded-md hover:bg-red-700"
                >
                  Retry
                </button>
              </div>
            </div>
          ) : filteredCredentials.length === 0 ? (
            <div className="flex items-center justify-center h-full">
              <div className="text-center text-gray-500">
                <div className="text-6xl mb-4">🔍</div>
                <p className="text-lg font-medium mb-2">
                  {searchQuery ? 'No matching credentials' : 'No credentials found'}
                </p>
                <p className="text-sm">
                  {searchQuery
                    ? 'Try a different search query'
                    : 'Add credentials to the vault first'}
                </p>
              </div>
            </div>
          ) : (
            <div className="grid gap-3">
              {filteredCredentials.map((cred) => (
                <button
                  key={cred.id}
                  onClick={() => {
                    onSelect(cred);
                    onClose();
                  }}
                  className="bg-white border border-gray-200 rounded-lg p-4 hover:border-blue-500 hover:shadow-md transition-all text-left"
                >
                  <div className="flex items-start justify-between">
                    <div className="flex-1">
                      <div className="flex items-center space-x-2 mb-2">
                        <span className="text-2xl">{getTypeIcon(cred.credential_type)}</span>
                        <div>
                          <h3 className="font-semibold text-gray-900">{cred.name}</h3>
                          <p className="text-xs text-gray-500 capitalize">
                            {cred.credential_type.replace('_', ' ')}
                          </p>
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
                        Added {formatDate(cred.created_at)}
                      </div>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          )}
        </div>

        {/* Footer */}
        {!loading && !error && filteredCredentials.length > 0 && (
          <div className="p-4 border-t border-gray-200 bg-gray-50">
            <p className="text-sm text-gray-600">
              Click on a credential to use it for this connection
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
