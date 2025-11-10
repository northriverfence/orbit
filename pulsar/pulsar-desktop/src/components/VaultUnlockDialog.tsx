import { useState } from 'react';
import VaultClient from '../lib/vaultClient';

interface VaultUnlockDialogProps {
  isInitialized: boolean;
  onUnlock: () => void;
  onCancel?: () => void;
}

export default function VaultUnlockDialog({
  isInitialized,
  onUnlock,
  onCancel,
}: VaultUnlockDialogProps) {
  const [masterPassword, setMasterPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (isInitialized) {
        // Unlock existing vault
        await VaultClient.unlock(masterPassword);
      } else {
        // Initialize new vault
        if (masterPassword !== confirmPassword) {
          setError('Passwords do not match');
          setLoading(false);
          return;
        }

        if (masterPassword.length < 8) {
          setError('Password must be at least 8 characters');
          setLoading(false);
          return;
        }

        await VaultClient.initialize(masterPassword);
      }

      // Clear sensitive data
      setMasterPassword('');
      setConfirmPassword('');

      onUnlock();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to access vault');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl p-6 w-full max-w-md">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-900">
            {isInitialized ? '🔒 Unlock Vault' : '🔐 Initialize Vault'}
          </h2>
          {onCancel && (
            <button
              onClick={onCancel}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              ✕
            </button>
          )}
        </div>

        <form onSubmit={handleSubmit}>
          {!isInitialized && (
            <div className="mb-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
              <p className="text-sm text-blue-800">
                <strong>First time setup:</strong> Create a master password to encrypt your credentials.
                This password cannot be recovered, so choose wisely!
              </p>
            </div>
          )}

          <div className="mb-4">
            <label htmlFor="masterPassword" className="block text-sm font-medium text-gray-700 mb-2">
              Master Password
            </label>
            <input
              id="masterPassword"
              type="password"
              value={masterPassword}
              onChange={(e) => setMasterPassword(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
              placeholder="Enter master password"
              autoFocus
              required
            />
          </div>

          {!isInitialized && (
            <div className="mb-4">
              <label htmlFor="confirmPassword" className="block text-sm font-medium text-gray-700 mb-2">
                Confirm Password
              </label>
              <input
                id="confirmPassword"
                type="password"
                value={confirmPassword}
                onChange={(e) => setConfirmPassword(e.target.value)}
                className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                placeholder="Confirm master password"
                required
              />
            </div>
          )}

          {error && (
            <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-md">
              <p className="text-sm text-red-800">{error}</p>
            </div>
          )}

          <div className="flex space-x-3">
            <button
              type="submit"
              disabled={loading || !masterPassword}
              className={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
                loading || !masterPassword
                  ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                  : 'bg-blue-600 text-white hover:bg-blue-700'
              }`}
            >
              {loading ? 'Processing...' : isInitialized ? 'Unlock' : 'Initialize'}
            </button>
            {onCancel && (
              <button
                type="button"
                onClick={onCancel}
                disabled={loading}
                className="px-4 py-2 border border-gray-300 rounded-md font-medium text-gray-700 hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
            )}
          </div>
        </form>

        {!isInitialized && (
          <div className="mt-4 pt-4 border-t border-gray-200">
            <p className="text-xs text-gray-500">
              💡 <strong>Tip:</strong> Use a strong, unique password. Consider using a passphrase
              with multiple words for better security.
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
