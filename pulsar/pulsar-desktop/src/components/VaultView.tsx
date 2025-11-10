import { useState, useEffect, useCallback } from 'react';
import VaultClient from '../lib/vaultClient';
import VaultUnlockDialog from './VaultUnlockDialog';
import VaultCredentialList from './VaultCredentialList';
import VaultSshKeyForm from './VaultSshKeyForm';
import LoadingSpinner from './LoadingSpinner';
import ErrorAlert from './ErrorAlert';
import { useToast } from './ToastContainer';
import type { VaultState, DecryptedCredential } from '../types/vault';

export default function VaultView() {
  const [vaultState, setVaultState] = useState<VaultState>('locked');
  const [isInitialized, setIsInitialized] = useState(false);
  const [loading, setLoading] = useState(true);
  const [showUnlockDialog, setShowUnlockDialog] = useState(false);
  const [showAddForm, setShowAddForm] = useState(false);
  const [selectedCredential, setSelectedCredential] = useState<DecryptedCredential | null>(null);
  const [error, setError] = useState<string | null>(null);
  const toast = useToast();

  const checkVaultStatus = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const [state, initialized] = await Promise.all([
        VaultClient.getState(),
        VaultClient.isInitialized(),
      ]);
      setVaultState(state);
      setIsInitialized(initialized);

      // Show unlock dialog if vault is locked or uninitialized
      if (state !== 'unlocked') {
        setShowUnlockDialog(true);
      }
    } catch (error) {
      console.error('Failed to check vault status:', error);
      const errorMessage = error instanceof Error ? error.message : 'Failed to check vault status';
      setError(errorMessage);
      toast.showError(errorMessage);
    } finally {
      setLoading(false);
    }
  }, [toast]);

  useEffect(() => {
    checkVaultStatus();
  }, [checkVaultStatus]);

  const handleUnlock = () => {
    setShowUnlockDialog(false);
    setVaultState('unlocked');
  };

  const handleLock = async () => {
    try {
      await VaultClient.lock();
      setVaultState('locked');
      setShowUnlockDialog(true);
      setSelectedCredential(null);
      toast.showSuccess('Vault locked successfully');
    } catch (error) {
      const errorMessage = `Failed to lock vault: ${error}`;
      toast.showError(errorMessage);
      setError(errorMessage);
    }
  };

  const handleViewCredential = async (id: string) => {
    try {
      const credential = await VaultClient.getCredential(id);
      setSelectedCredential(credential);
    } catch (error) {
      const errorMessage = `Failed to load credential: ${error}`;
      toast.showError(errorMessage);
      setError(errorMessage);
    }
  };

  const handleCloseCredentialView = () => {
    setSelectedCredential(null);
  };

  const handleAddSuccess = () => {
    setShowAddForm(false);
    // Trigger a refresh of the credential list
    checkVaultStatus();
  };

  if (loading) {
    return (
      <div className="w-full h-full flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <LoadingSpinner size="xl" />
          <p className="mt-6 text-lg font-medium text-gray-700">Loading vault...</p>
          <p className="mt-2 text-sm text-gray-500">Initializing secure storage</p>
        </div>
      </div>
    );
  }

  return (
    <div className="w-full h-full flex flex-col bg-gray-50">
      {/* Unlock dialog */}
      {showUnlockDialog && vaultState !== 'unlocked' && (
        <VaultUnlockDialog
          isInitialized={isInitialized}
          onUnlock={handleUnlock}
        />
      )}

      {/* Add credential form overlay */}
      {showAddForm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-40 p-4 overflow-y-auto modal-backdrop">
          <div className="max-w-2xl w-full modal-content">
            <VaultSshKeyForm
              onSuccess={handleAddSuccess}
              onCancel={() => setShowAddForm(false)}
            />
          </div>
        </div>
      )}

      {/* View credential overlay */}
      {selectedCredential && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-40 p-4 overflow-y-auto modal-backdrop">
          <div className="bg-white rounded-lg shadow-xl p-6 max-w-2xl w-full max-h-[80vh] overflow-y-auto modal-content">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-2xl font-bold text-gray-900">
                {selectedCredential.data.type === 'ssh_key' && '🔑'}
                {selectedCredential.data.type === 'password' && '🔐'}
                {selectedCredential.data.type === 'certificate' && '📜'}
                {' '}
                {selectedCredential.name}
              </h2>
              <button
                onClick={handleCloseCredentialView}
                className="text-gray-400 hover:text-gray-600 transition-colors"
              >
                ✕
              </button>
            </div>

            <div className="space-y-4">
              {/* Type */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">Type</label>
                <p className="text-gray-900 capitalize">{selectedCredential.data.type.replace('_', ' ')}</p>
              </div>

              {/* SSH Key Data */}
              {selectedCredential.data.type === 'ssh_key' && (
                <>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Private Key</label>
                    <textarea
                      readOnly
                      value={selectedCredential.data.ssh_key.private_key}
                      rows={8}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md font-mono text-xs bg-gray-50"
                    />
                  </div>
                  {selectedCredential.data.ssh_key.public_key && (
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">Public Key</label>
                      <textarea
                        readOnly
                        value={selectedCredential.data.ssh_key.public_key}
                        rows={2}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md font-mono text-xs bg-gray-50"
                      />
                    </div>
                  )}
                  {selectedCredential.data.ssh_key.passphrase && (
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">Passphrase</label>
                      <p className="text-gray-900 font-mono">••••••••</p>
                    </div>
                  )}
                </>
              )}

              {/* Password Data */}
              {selectedCredential.data.type === 'password' && (
                <>
                  {selectedCredential.data.password.username && (
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">Username</label>
                      <p className="text-gray-900">{selectedCredential.data.password.username}</p>
                    </div>
                  )}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Password</label>
                    <p className="text-gray-900 font-mono">{selectedCredential.data.password.password}</p>
                  </div>
                </>
              )}

              {/* Certificate Data */}
              {selectedCredential.data.type === 'certificate' && (
                <>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">Certificate</label>
                    <textarea
                      readOnly
                      value={selectedCredential.data.certificate.certificate}
                      rows={8}
                      className="w-full px-3 py-2 border border-gray-300 rounded-md font-mono text-xs bg-gray-50"
                    />
                  </div>
                  {selectedCredential.data.certificate.private_key && (
                    <div>
                      <label className="block text-sm font-medium text-gray-700 mb-1">Private Key</label>
                      <textarea
                        readOnly
                        value={selectedCredential.data.certificate.private_key}
                        rows={6}
                        className="w-full px-3 py-2 border border-gray-300 rounded-md font-mono text-xs bg-gray-50"
                      />
                    </div>
                  )}
                </>
              )}

              {/* Metadata */}
              {selectedCredential.username && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Username</label>
                  <p className="text-gray-900">{selectedCredential.username}</p>
                </div>
              )}

              {selectedCredential.host_pattern && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Host Pattern</label>
                  <p className="text-gray-900">{selectedCredential.host_pattern}</p>
                </div>
              )}

              {selectedCredential.tags.length > 0 && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">Tags</label>
                  <div className="flex flex-wrap gap-2">
                    {selectedCredential.tags.map((tag) => (
                      <span
                        key={tag}
                        className="px-2 py-1 text-sm bg-blue-100 text-blue-700 rounded-full"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              <div className="pt-4 border-t border-gray-200">
                <button
                  onClick={handleCloseCredentialView}
                  className="w-full px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors font-medium"
                >
                  Close
                </button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Error Alert */}
      {error && (
        <div className="p-4">
          <ErrorAlert
            message={error}
            type="error"
            onDismiss={() => setError(null)}
            onRetry={checkVaultStatus}
          />
        </div>
      )}

      {/* Header */}
      <div className="flex-shrink-0 bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">🗄️ Vault</h1>
          <div className="flex items-center space-x-3">
            {vaultState === 'unlocked' && (
              <div className="flex items-center space-x-2 px-3 py-1.5 bg-green-50 border border-green-200 rounded-md">
                <span className="w-2 h-2 bg-green-500 rounded-full"></span>
                <span className="text-sm font-medium text-green-700">Unlocked</span>
              </div>
            )}
            {vaultState === 'unlocked' && (
              <button
                onClick={handleLock}
                className="px-4 py-2 border border-gray-300 rounded-md font-medium text-gray-700 hover:bg-gray-50 transition-colors"
              >
                🔒 Lock Vault
              </button>
            )}
          </div>
        </div>

        {vaultState === 'unlocked' && (
          <p className="text-sm text-gray-600 mt-2">
            Securely store and manage your SSH keys, passwords, and certificates
          </p>
        )}
      </div>

      {/* Main content */}
      <div className="flex-1 overflow-hidden">
        {vaultState === 'unlocked' ? (
          <VaultCredentialList
            onAdd={() => setShowAddForm(true)}
            onView={handleViewCredential}
            onDelete={() => checkVaultStatus()}
          />
        ) : (
          <div className="h-full flex items-center justify-center">
            <div className="text-center text-gray-500">
              <div className="text-6xl mb-4">🔒</div>
              <p className="text-lg font-medium mb-2">Vault is locked</p>
              <p className="text-sm">Enter your master password to access your credentials</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
