import { useState } from 'react';
import VaultClient from '../lib/vaultClient';

interface VaultSshKeyFormProps {
  onSuccess: (id: string) => void;
  onCancel: () => void;
}

export default function VaultSshKeyForm({ onSuccess, onCancel }: VaultSshKeyFormProps) {
  const [name, setName] = useState('');
  const [privateKey, setPrivateKey] = useState('');
  const [publicKey, setPublicKey] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [username, setUsername] = useState('');
  const [hostPattern, setHostPattern] = useState('');
  const [tags, setTags] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleFileUpload = async (type: 'private' | 'public') => {
    try {
      // In Tauri, we'd use the file dialog
      // For now, this is a placeholder
      const input = document.createElement('input');
      input.type = 'file';
      input.accept = type === 'private' ? '' : '.pub';
      input.onchange = async (e) => {
        const file = (e.target as HTMLInputElement).files?.[0];
        if (file) {
          const content = await file.text();
          if (type === 'private') {
            setPrivateKey(content);
          } else {
            setPublicKey(content);
          }
        }
      };
      input.click();
    } catch (err) {
      console.error('Failed to upload file:', err);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      if (!name.trim()) {
        throw new Error('Name is required');
      }

      if (!privateKey.trim()) {
        throw new Error('Private key is required');
      }

      const tagArray = tags
        .split(',')
        .map((t) => t.trim())
        .filter((t) => t.length > 0);

      const id = await VaultClient.storeSshKey(
        name.trim(),
        privateKey.trim(),
        publicKey.trim() || undefined,
        passphrase.trim() || undefined,
        tagArray,
        username.trim() || undefined,
        hostPattern.trim() || undefined
      );

      // Clear form
      setName('');
      setPrivateKey('');
      setPublicKey('');
      setPassphrase('');
      setUsername('');
      setHostPattern('');
      setTags('');

      onSuccess(id);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to store SSH key');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-lg p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-2xl font-bold text-gray-900">🔑 Add SSH Key</h2>
        <button
          onClick={onCancel}
          className="text-gray-400 hover:text-gray-600 transition-colors"
        >
          ✕
        </button>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        {/* Name */}
        <div>
          <label htmlFor="name" className="block text-sm font-medium text-gray-700 mb-1">
            Name <span className="text-red-500">*</span>
          </label>
          <input
            id="name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="My Production SSH Key"
            required
          />
        </div>

        {/* Private Key */}
        <div>
          <label htmlFor="privateKey" className="block text-sm font-medium text-gray-700 mb-1">
            Private Key <span className="text-red-500">*</span>
          </label>
          <div className="flex space-x-2 mb-2">
            <button
              type="button"
              onClick={() => handleFileUpload('private')}
              className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
            >
              📁 Upload File
            </button>
          </div>
          <textarea
            id="privateKey"
            value={privateKey}
            onChange={(e) => setPrivateKey(e.target.value)}
            rows={8}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-xs"
            placeholder="-----BEGIN OPENSSH PRIVATE KEY-----&#10;...&#10;-----END OPENSSH PRIVATE KEY-----"
            required
          />
        </div>

        {/* Public Key */}
        <div>
          <label htmlFor="publicKey" className="block text-sm font-medium text-gray-700 mb-1">
            Public Key (optional)
          </label>
          <div className="flex space-x-2 mb-2">
            <button
              type="button"
              onClick={() => handleFileUpload('public')}
              className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200 transition-colors"
            >
              📁 Upload File
            </button>
          </div>
          <textarea
            id="publicKey"
            value={publicKey}
            onChange={(e) => setPublicKey(e.target.value)}
            rows={2}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono text-xs"
            placeholder="ssh-rsa AAAA... user@host"
          />
        </div>

        {/* Passphrase */}
        <div>
          <label htmlFor="passphrase" className="block text-sm font-medium text-gray-700 mb-1">
            Passphrase (if key is encrypted)
          </label>
          <input
            id="passphrase"
            type="password"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="Key passphrase"
          />
        </div>

        {/* Username */}
        <div>
          <label htmlFor="username" className="block text-sm font-medium text-gray-700 mb-1">
            Username (optional)
          </label>
          <input
            id="username"
            type="text"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="admin"
          />
        </div>

        {/* Host Pattern */}
        <div>
          <label htmlFor="hostPattern" className="block text-sm font-medium text-gray-700 mb-1">
            Host Pattern (optional)
          </label>
          <input
            id="hostPattern"
            type="text"
            value={hostPattern}
            onChange={(e) => setHostPattern(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="*.example.com"
          />
          <p className="text-xs text-gray-500 mt-1">
            Use wildcards to match multiple hosts (e.g., *.example.com)
          </p>
        </div>

        {/* Tags */}
        <div>
          <label htmlFor="tags" className="block text-sm font-medium text-gray-700 mb-1">
            Tags (optional)
          </label>
          <input
            id="tags"
            type="text"
            value={tags}
            onChange={(e) => setTags(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            placeholder="production, aws, work"
          />
          <p className="text-xs text-gray-500 mt-1">Comma-separated tags for organization</p>
        </div>

        {/* Error */}
        {error && (
          <div className="p-3 bg-red-50 border border-red-200 rounded-md">
            <p className="text-sm text-red-800">{error}</p>
          </div>
        )}

        {/* Actions */}
        <div className="flex space-x-3 pt-4">
          <button
            type="submit"
            disabled={loading}
            className={`flex-1 px-4 py-2 rounded-md font-medium transition-colors ${
              loading
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : 'bg-blue-600 text-white hover:bg-blue-700'
            }`}
          >
            {loading ? 'Saving...' : 'Save SSH Key'}
          </button>
          <button
            type="button"
            onClick={onCancel}
            disabled={loading}
            className="px-4 py-2 border border-gray-300 rounded-md font-medium text-gray-700 hover:bg-gray-50 transition-colors"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}
