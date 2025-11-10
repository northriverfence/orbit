import { useState, useEffect, useRef } from 'react'
import VaultClient from '../lib/vaultClient'
import VaultCredentialSelector from './VaultCredentialSelector'
import InlineLoader from './InlineLoader'
import ErrorAlert from './ErrorAlert'
import { useToast } from './ToastContainer'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { useKeyboardShortcut, SHORTCUTS } from '../hooks/useKeyboardShortcut'
import type { CredentialSummary } from '../types/vault'

interface ConnectionDialogProps {
  isOpen: boolean
  onClose: () => void
  onConnect: (config: ConnectionConfig) => void
}

export interface ConnectionConfig {
  host: string
  port: number
  username: string
  authType: 'password' | 'publickey' | 'agent'
  password?: string
  keyPath?: string
  keyPassphrase?: string
  saveToVault?: boolean
  selectedCredentialId?: string | null
}

export default function ConnectionDialog({ isOpen, onClose, onConnect }: ConnectionDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null)
  const [config, setConfig] = useState<ConnectionConfig>({
    host: '',
    port: 22,
    username: '',
    authType: 'password',
    password: '',
    keyPath: '',
    keyPassphrase: '',
  })

  const [errors, setErrors] = useState<Record<string, string>>({})
  const [showVaultSelector, setShowVaultSelector] = useState(false)
  const [saveToVault, setSaveToVault] = useState(false)
  const [vaultUnlocked, setVaultUnlocked] = useState(false)
  const [selectedCredentialId, setSelectedCredentialId] = useState<string | null>(null)
  const [agentAvailable, setAgentAvailable] = useState(false)
  const [agentIdentitiesCount, setAgentIdentitiesCount] = useState(0)
  const [isLoadingVault, setIsLoadingVault] = useState(false)
  const [isLoadingAgent, setIsLoadingAgent] = useState(false)
  const [isConnecting, setIsConnecting] = useState(false)
  const [isLoadingCredential, setIsLoadingCredential] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const toast = useToast()

  // Check vault status and agent availability when dialog opens
  useEffect(() => {
    if (isOpen) {
      setError(null)
      checkVaultStatus()
      checkAgentStatus()
    }
  }, [isOpen])

  const checkVaultStatus = async () => {
    setIsLoadingVault(true)
    try {
      const unlocked = await VaultClient.isUnlocked()
      setVaultUnlocked(unlocked)
    } catch (error) {
      console.error('Failed to check vault status:', error)
      setVaultUnlocked(false)
    } finally {
      setIsLoadingVault(false)
    }
  }

  const checkAgentStatus = async () => {
    setIsLoadingAgent(true)
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const available = await invoke<boolean>('check_ssh_agent')
      setAgentAvailable(available)

      if (available) {
        const identities = await invoke<Array<{ comment: string | null; fingerprint: string }>>('list_agent_identities')
        setAgentIdentitiesCount(identities.length)
      } else {
        setAgentIdentitiesCount(0)
      }
    } catch (error) {
      console.error('Failed to check SSH agent:', error)
      setAgentAvailable(false)
      setAgentIdentitiesCount(0)
    } finally {
      setIsLoadingAgent(false)
    }
  }

  const handleVaultCredentialSelect = async (credential: CredentialSummary) => {
    setIsLoadingCredential(true)
    try {
      // Get the full decrypted credential
      const fullCredential = await VaultClient.getCredential(credential.id)

      // Auto-fill the form based on credential type
      if (fullCredential.data.type === 'ssh_key') {
        const sshKey = fullCredential.data.ssh_key
        setConfig({
          ...config,
          host: fullCredential.host_pattern?.replace('*', '') || config.host,
          username: fullCredential.username || config.username,
          authType: 'publickey',
          keyPath: '<from-vault>',  // Marker that key comes from vault
          keyPassphrase: sshKey.passphrase || '',
        })
        setSelectedCredentialId(credential.id)
      } else if (fullCredential.data.type === 'password') {
        const passwordData = fullCredential.data.password
        setConfig({
          ...config,
          host: fullCredential.host_pattern?.replace('*', '') || config.host,
          username: passwordData.username || fullCredential.username || config.username,
          authType: 'password',
          password: passwordData.password,
        })
        setSelectedCredentialId(credential.id)
      }
    } catch (error) {
      console.error('Failed to load credential:', error)
      toast.showError(`Failed to load credential: ${error}`)
      setError(String(error))
    } finally {
      setIsLoadingCredential(false)
      setShowVaultSelector(false)
    }
  }

  if (!isOpen) return null

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {}

    // Host validation
    if (!config.host.trim()) {
      newErrors.host = 'Host is required'
    }

    // Port validation
    if (config.port < 1 || config.port > 65535) {
      newErrors.port = 'Port must be between 1 and 65535'
    }

    // Username validation
    if (!config.username.trim()) {
      newErrors.username = 'Username is required'
    }

    // Authentication validation
    if (config.authType === 'password') {
      if (!config.password) {
        newErrors.password = 'Password is required'
      }
    } else if (config.authType === 'publickey') {
      // If using vault credential, keyPath will be '<from-vault>'
      if (!config.keyPath?.trim() || (config.keyPath !== '<from-vault>' && !config.keyPath.startsWith('/'))) {
        newErrors.keyPath = 'Key path is required'
      }
    }
    // Agent auth requires no additional validation

    setErrors(newErrors)
    return Object.keys(newErrors).length === 0
  }

  const handleConnect = () => {
    if (validate()) {
      setIsConnecting(true)
      onConnect({
        ...config,
        saveToVault,
        selectedCredentialId,
      })
      onClose()
    }
  }

  // Keyboard navigation (after function definitions)
  useFocusTrap(dialogRef, isOpen)
  useKeyboardShortcut(SHORTCUTS.ESCAPE, onClose, isOpen)
  useKeyboardShortcut(
    [
      { key: 'Enter', ctrlKey: true },
      { key: 'Enter', metaKey: true },
    ],
    handleConnect,
    isOpen
  )

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 modal-backdrop">
      <div
        ref={dialogRef}
        className="bg-white rounded-lg shadow-xl w-full max-w-md p-6 modal-content"
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-gray-800">New SSH Connection</h2>
          <button
            onClick={onClose}
            className="text-gray-400 hover:text-gray-600 transition-colors"
            aria-label="Close"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Error Alert */}
        {error && (
          <ErrorAlert
            message={error}
            type="error"
            onDismiss={() => setError(null)}
          />
        )}

        {/* Form */}
        <div className="space-y-4">
          {/* Host */}
          <div>
            <label htmlFor="host" className="block text-sm font-medium text-gray-700 mb-1">
              Host <span className="text-red-500">*</span>
            </label>
            <input
              id="host"
              type="text"
              value={config.host}
              onChange={(e) => setConfig({ ...config, host: e.target.value })}
              className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-accent-primary ${
                errors.host ? 'border-red-500' : 'border-gray-300'
              }`}
              placeholder="example.com or 192.168.1.10"
              autoFocus
            />
            {errors.host && <p className="text-red-500 text-sm mt-1">{errors.host}</p>}
          </div>

          {/* Port */}
          <div>
            <label htmlFor="port" className="block text-sm font-medium text-gray-700 mb-1">
              Port <span className="text-red-500">*</span>
            </label>
            <input
              id="port"
              type="number"
              value={config.port}
              onChange={(e) => setConfig({ ...config, port: parseInt(e.target.value) || 22 })}
              className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-accent-primary ${
                errors.port ? 'border-red-500' : 'border-gray-300'
              }`}
              min="1"
              max="65535"
            />
            {errors.port && <p className="text-red-500 text-sm mt-1">{errors.port}</p>}
          </div>

          {/* Username */}
          <div>
            <label htmlFor="username" className="block text-sm font-medium text-gray-700 mb-1">
              Username <span className="text-red-500">*</span>
            </label>
            <input
              id="username"
              type="text"
              value={config.username}
              onChange={(e) => setConfig({ ...config, username: e.target.value })}
              className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-accent-primary ${
                errors.username ? 'border-red-500' : 'border-gray-300'
              }`}
              placeholder="root or user"
            />
            {errors.username && <p className="text-red-500 text-sm mt-1">{errors.username}</p>}
          </div>

          {/* Vault Quick Access */}
          {isLoadingVault ? (
            <div className="p-3 bg-gray-50 border border-gray-200 rounded-md">
              <InlineLoader message="Checking vault status..." />
            </div>
          ) : vaultUnlocked ? (
            <div className="p-3 bg-blue-50 border border-blue-200 rounded-md">
              <div className="flex items-center justify-between">
                <div className="flex items-center space-x-2">
                  <span className="text-blue-700">🗄️</span>
                  <span className="text-sm font-medium text-blue-800">
                    Use credentials from vault
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => setShowVaultSelector(true)}
                  disabled={isLoadingCredential}
                  className="px-3 py-1 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isLoadingCredential ? 'Loading...' : 'Select'}
                </button>
              </div>
            </div>
          ) : null}

          {/* Authentication Type */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Authentication Method <span className="text-red-500">*</span>
            </label>
            <div className="flex gap-4 flex-wrap">
              <label className="flex items-center">
                <input
                  type="radio"
                  value="password"
                  checked={config.authType === 'password'}
                  onChange={(e) => setConfig({ ...config, authType: e.target.value as 'password' })}
                  className="mr-2"
                />
                <span className="text-sm text-gray-700">Password</span>
              </label>
              <label className="flex items-center">
                <input
                  type="radio"
                  value="publickey"
                  checked={config.authType === 'publickey'}
                  onChange={(e) => setConfig({ ...config, authType: e.target.value as 'publickey' })}
                  className="mr-2"
                />
                <span className="text-sm text-gray-700">Public Key</span>
              </label>
              {isLoadingAgent ? (
                <div className="flex items-center">
                  <InlineLoader message="Checking SSH agent..." size="sm" />
                </div>
              ) : agentAvailable ? (
                <label className="flex items-center">
                  <input
                    type="radio"
                    value="agent"
                    checked={config.authType === 'agent'}
                    onChange={(e) => setConfig({ ...config, authType: e.target.value as 'agent' })}
                    className="mr-2"
                  />
                  <span className="text-sm text-gray-700">
                    SSH Agent ({agentIdentitiesCount} {agentIdentitiesCount === 1 ? 'key' : 'keys'})
                  </span>
                </label>
              ) : null}
            </div>
          </div>

          {/* Password Authentication */}
          {config.authType === 'password' && (
            <div>
              <label htmlFor="password" className="block text-sm font-medium text-gray-700 mb-1">
                Password <span className="text-red-500">*</span>
              </label>
              <input
                id="password"
                type="password"
                value={config.password}
                onChange={(e) => setConfig({ ...config, password: e.target.value })}
                className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-accent-primary ${
                  errors.password ? 'border-red-500' : 'border-gray-300'
                }`}
                placeholder="Enter password"
              />
              {errors.password && <p className="text-red-500 text-sm mt-1">{errors.password}</p>}
            </div>
          )}

          {/* Public Key Authentication */}
          {config.authType === 'publickey' && (
            <>
              <div>
                <label htmlFor="keyPath" className="block text-sm font-medium text-gray-700 mb-1">
                  Private Key Path <span className="text-red-500">*</span>
                </label>
                {config.keyPath === '<from-vault>' ? (
                  <div className="w-full px-3 py-2 border border-blue-300 rounded-md bg-blue-50">
                    <div className="flex items-center space-x-2">
                      <span>🗄️</span>
                      <span className="text-sm font-medium text-blue-800">Using SSH key from vault</span>
                    </div>
                    <button
                      type="button"
                      onClick={() => {
                        setConfig({ ...config, keyPath: '' })
                        setSelectedCredentialId(null)
                      }}
                      className="text-xs text-blue-600 hover:text-blue-800 mt-1"
                    >
                      Clear and enter manually
                    </button>
                  </div>
                ) : (
                  <input
                    id="keyPath"
                    type="text"
                    value={config.keyPath}
                    onChange={(e) => setConfig({ ...config, keyPath: e.target.value })}
                    className={`w-full px-3 py-2 border rounded-md focus:outline-none focus:ring-2 focus:ring-accent-primary ${
                      errors.keyPath ? 'border-red-500' : 'border-gray-300'
                    }`}
                    placeholder="~/.ssh/id_rsa or /path/to/key"
                  />
                )}
                {errors.keyPath && <p className="text-red-500 text-sm mt-1">{errors.keyPath}</p>}
              </div>

              <div>
                <label htmlFor="keyPassphrase" className="block text-sm font-medium text-gray-700 mb-1">
                  Passphrase (optional)
                </label>
                <input
                  id="keyPassphrase"
                  type="password"
                  value={config.keyPassphrase}
                  onChange={(e) => setConfig({ ...config, keyPassphrase: e.target.value })}
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-accent-primary"
                  placeholder="Enter passphrase if key is encrypted"
                />
              </div>
            </>
          )}

          {/* SSH Agent Authentication */}
          {config.authType === 'agent' && (
            <div className="p-4 bg-green-50 border border-green-200 rounded-md">
              <div className="flex items-start space-x-3">
                <div className="flex-shrink-0">
                  <span className="text-2xl">🔑</span>
                </div>
                <div className="flex-1">
                  <h4 className="text-sm font-medium text-green-800 mb-1">
                    SSH Agent Available
                  </h4>
                  <p className="text-sm text-green-700 mb-2">
                    Authentication will use keys from your SSH agent.
                    {agentIdentitiesCount > 0 && (
                      <> Found {agentIdentitiesCount} {agentIdentitiesCount === 1 ? 'key' : 'keys'}.</>
                    )}
                  </p>
                  <p className="text-xs text-green-600">
                    The agent will automatically try all available keys until one succeeds.
                  </p>
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between mt-6 pt-4 border-t border-gray-200">
          <div className="text-xs text-gray-500">
            Press <kbd className="px-1.5 py-0.5 bg-gray-100 border border-gray-300 rounded">Ctrl+Enter</kbd> to connect
          </div>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="px-4 py-2 text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 transition-colors btn-press"
            >
              Cancel
            </button>
            <button
              onClick={handleConnect}
              disabled={isConnecting}
              className="px-4 py-2 text-white bg-accent-primary rounded-md hover:bg-green-600 transition-colors disabled:opacity-50 disabled:cursor-not-allowed btn-press"
            >
              {isConnecting ? 'Connecting...' : 'Connect'}
            </button>
          </div>
        </div>

        {/* Security Notice */}
        <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-md">
          <div className="flex items-start">
            <svg className="w-5 h-5 text-blue-500 mt-0.5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
            <div className="text-xs text-blue-800">
              <p className="font-semibold mb-1">Security Features Active</p>
              <p>• Host key verification enabled</p>
              <p>• Unknown hosts will be auto-accepted (development mode)</p>
              <p>• Changed host keys will be rejected</p>
            </div>
          </div>
        </div>

        {/* Save to Vault option */}
        {vaultUnlocked && !selectedCredentialId && (
          <div className="mt-4 flex items-center">
            <input
              id="saveToVault"
              type="checkbox"
              checked={saveToVault}
              onChange={(e) => setSaveToVault(e.target.checked)}
              className="mr-2"
            />
            <label htmlFor="saveToVault" className="text-sm text-gray-700">
              💾 Save this connection to vault after connecting
            </label>
          </div>
        )}
      </div>

      {/* Vault Credential Selector Modal */}
      <VaultCredentialSelector
        isOpen={showVaultSelector}
        onClose={() => setShowVaultSelector(false)}
        onSelect={handleVaultCredentialSelect}
        hostHint={config.host}
      />
    </div>
  )
}
