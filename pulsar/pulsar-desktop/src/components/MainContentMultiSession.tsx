import { useState, useCallback, useEffect, useRef } from 'react'
import Terminal from './Terminal'
import PulsarTerminal from './PulsarTerminal'
import ConnectionDialog, { ConnectionConfig } from './ConnectionDialog'
import SessionTabs, { Session } from './SessionTabs'
import LoadingOverlay from './LoadingOverlay'
import SessionRestoreNotification from './SessionRestoreNotification'
import SessionRestoreDialog from './SessionRestoreDialog'
import { useToast } from './ToastContainer'
import {
  loadSessions,
  SessionAutoSaver,
  PersistedSession,
} from '../lib/sessionPersistence'
import VaultClient from '../lib/vaultClient'
import { readTextFile } from '@tauri-apps/plugin-fs'

interface SessionData extends Session {
  createdAt: string
  lastActive: string
  sessionConfig?: {
    host?: string
    port?: number
    username?: string
    password?: string
    authType?: 'password' | 'publickey' | 'agent'
    keyPath?: string
    keyPassphrase?: string
    credentialId?: string | null
  }
}

export default function MainContentMultiSession() {
  const [sessions, setSessions] = useState<SessionData[]>([])
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null)
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [sessionCounter, setSessionCounter] = useState(1)
  const [isLoadingSession, setIsLoadingSession] = useState(true)
  const [showRestoreNotification, setShowRestoreNotification] = useState(false)
  const [showRestoreDialog, setShowRestoreDialog] = useState(false)
  const [persistedSessionsToRestore, setPersistedSessionsToRestore] = useState<SessionData[]>([])
  const autoSaverRef = useRef<SessionAutoSaver | null>(null)
  const toast = useToast()

  // Initialize auto-saver
  if (!autoSaverRef.current) {
    autoSaverRef.current = new SessionAutoSaver(1000) // 1 second debounce
  }

  // Load sessions from disk on mount
  useEffect(() => {
    const loadPersistedSessions = async () => {
      try {
        const persisted = await loadSessions()
        if (persisted && persisted.sessions.length > 0) {
          console.log(`Found ${persisted.sessions.length} sessions to restore`)

          // Store sessions for potential restoration
          setPersistedSessionsToRestore(persisted.sessions as SessionData[])

          // Show restore notification
          setShowRestoreNotification(true)

          // Update session counter to avoid ID conflicts
          const maxCounter = persisted.sessions.reduce((max, session) => {
            const match = session.name.match(/Local (\d+)/)
            return match ? Math.max(max, parseInt(match[1])) : max
          }, 0)
          setSessionCounter(maxCounter + 1)
        }
      } catch (error) {
        console.error('Failed to load sessions:', error)
        toast.showError('Failed to load previous sessions')
      } finally {
        setIsLoadingSession(false)
      }
    }

    loadPersistedSessions()
  }, [toast])

  // Auto-save sessions when they change
  useEffect(() => {
    if (!isLoadingSession && autoSaverRef.current) {
      autoSaverRef.current.scheduleSave(sessions as PersistedSession[], activeSessionId)
    }
  }, [sessions, activeSessionId, isLoadingSession])

  // Flush auto-save on unmount
  useEffect(() => {
    return () => {
      if (autoSaverRef.current) {
        autoSaverRef.current.flush(sessions as PersistedSession[], activeSessionId)
      }
    }
  }, [sessions, activeSessionId])

  // Create a new local terminal session
  const createLocalSession = useCallback(() => {
    const now = new Date().toISOString()
    const newSession: SessionData = {
      id: `local-${Date.now()}`,
      name: `Local ${sessionCounter}`,
      type: 'local',
      active: true,
      createdAt: now,
      lastActive: now,
    }
    setSessions((prev) => [...prev, newSession])
    setActiveSessionId(newSession.id)
    setSessionCounter((c) => c + 1)
  }, [sessionCounter])

  // Create a new SSH session
  const createSSHSession = useCallback(
    async (config: ConnectionConfig) => {
      const now = new Date().toISOString()
      const newSession: SessionData = {
        id: `ssh-${Date.now()}`,
        name: `${config.username}@${config.host}`,
        type: 'ssh',
        active: true,
        createdAt: now,
        lastActive: now,
        sessionConfig: {
          host: config.host,
          port: config.port,
          username: config.username,
          password: config.authType === 'password' ? config.password : undefined,
          authType: config.authType,
          keyPath: config.keyPath,
          keyPassphrase: config.keyPassphrase,
          credentialId: config.selectedCredentialId,
        },
      }
      setSessions((prev) => [...prev, newSession])
      setActiveSessionId(newSession.id)
      setIsDialogOpen(false)

      // Save to vault if requested (and not already from vault)
      if (config.saveToVault && !config.selectedCredentialId) {
        try {
          const isUnlocked = await VaultClient.isUnlocked()
          if (!isUnlocked) {
            console.warn('Vault is locked, cannot save credential')
            return
          }

          const credentialName = `Connection to ${config.host}`
          const tags = ['auto-saved']
          const hostPattern = config.host

          if (config.authType === 'password' && config.password) {
            // Save password credential
            await VaultClient.storePassword(
              credentialName,
              config.password,
              config.username,
              tags,
              hostPattern
            )
            console.log('Saved password credential to vault')
          } else if (config.authType === 'publickey' && config.keyPath && config.keyPath !== '<from-vault>') {
            // Read SSH key file and save to vault
            try {
              const privateKey = await readTextFile(config.keyPath)

              // Try to read public key (optional)
              let publicKey: string | undefined
              try {
                publicKey = await readTextFile(`${config.keyPath}.pub`)
              } catch {
                // Public key file not found, that's ok
                publicKey = undefined
              }

              await VaultClient.storeSshKey(
                credentialName,
                privateKey,
                publicKey,
                config.keyPassphrase,
                tags,
                config.username,
                hostPattern
              )
              console.log('Saved SSH key credential to vault')
            } catch (error) {
              console.error('Failed to read SSH key file:', error)
            }
          }
        } catch (error) {
          console.error('Failed to save credential to vault:', error)
          // Don't block connection if vault save fails
        }
      }
    },
    []
  )

  // Handle new session creation (shows dialog)
  const handleNewSession = useCallback(() => {
    setIsDialogOpen(true)
  }, [])

  // Handle session selection
  const handleSessionSelect = useCallback((sessionId: string) => {
    setActiveSessionId(sessionId)
  }, [])

  // Handle session close
  const handleSessionClose = useCallback(
    (sessionId: string) => {
      setSessions((prev) => {
        const filtered = prev.filter((s) => s.id !== sessionId)

        // If closing active session, switch to another
        if (sessionId === activeSessionId) {
          if (filtered.length > 0) {
            const index = prev.findIndex((s) => s.id === sessionId)
            const newActiveIndex = Math.min(index, filtered.length - 1)
            setActiveSessionId(filtered[newActiveIndex].id)
          } else {
            setActiveSessionId(null)
          }
        }

        return filtered
      })
    },
    [activeSessionId]
  )

  // Handle session rename
  const handleSessionRename = useCallback((sessionId: string, newName: string) => {
    setSessions((prev) =>
      prev.map((s) => (s.id === sessionId ? { ...s, name: newName } : s))
    )
  }, [])

  // Handle connection from dialog
  const handleConnect = useCallback(
    (config: ConnectionConfig) => {
      createSSHSession(config)
    },
    [createSSHSession]
  )

  // Handle restore all sessions
  const handleRestoreAll = useCallback(async () => {
    const persisted = persistedSessionsToRestore.find((_, i) => i === 0)?.id
      ? persistedSessionsToRestore
      : []

    if (persisted.length > 0) {
      setSessions(persisted)
      setActiveSessionId(persisted[0]?.id || null)
      setShowRestoreNotification(false)
      toast.showSuccess(`Restored ${persisted.length} session${persisted.length > 1 ? 's' : ''}`)
    }
  }, [persistedSessionsToRestore, toast])

  // Handle selective session restoration
  const handleRestoreSelected = useCallback(
    async (sessionIds: string[]) => {
      const sessionsToRestore = persistedSessionsToRestore.filter((s) =>
        sessionIds.includes(s.id)
      )

      if (sessionsToRestore.length > 0) {
        setSessions(sessionsToRestore)
        setActiveSessionId(sessionsToRestore[0]?.id || null)
        setShowRestoreDialog(false)
        setShowRestoreNotification(false)
        toast.showSuccess(
          `Restored ${sessionsToRestore.length} session${sessionsToRestore.length > 1 ? 's' : ''}`
        )
      }
    },
    [persistedSessionsToRestore, toast]
  )

  // Handle dismiss restore notification
  const handleDismissRestore = useCallback(() => {
    setShowRestoreNotification(false)
    setPersistedSessionsToRestore([])
    toast.showInfo('Starting with a clean slate')
  }, [toast])

  // Get active session
  const activeSession = sessions.find((s) => s.id === activeSessionId)

  return (
    <div className="flex-1 flex flex-col bg-gray-50 relative">
      {/* Loading overlay during session restoration */}
      {isLoadingSession && (
        <LoadingOverlay message="Restoring sessions..." fullScreen={true} />
      )}

      {/* Session Tabs */}
      {sessions.length > 0 && (
        <SessionTabs
          sessions={sessions}
          activeSessionId={activeSessionId}
          onSessionSelect={handleSessionSelect}
          onSessionClose={handleSessionClose}
          onSessionRename={handleSessionRename}
          onNewSession={handleNewSession}
        />
      )}

      {/* Main Terminal Area */}
      <div className="flex-1 p-4">
        {activeSession ? (
          <div className="h-full bg-[#1e1e1e] rounded-lg shadow-sm overflow-hidden">
            {activeSession.type === 'ssh' && activeSession.sessionConfig ? (
              <Terminal
                sessionId={activeSession.id}
                host={activeSession.sessionConfig.host!}
                port={activeSession.sessionConfig.port!}
                username={activeSession.sessionConfig.username!}
                password={activeSession.sessionConfig.password}
                authType={activeSession.sessionConfig.authType}
                keyPath={activeSession.sessionConfig.keyPath}
                keyPassphrase={activeSession.sessionConfig.keyPassphrase}
                credentialId={activeSession.sessionConfig.credentialId}
              />
            ) : activeSession.type === 'local' ? (
              <PulsarTerminal />
            ) : null}
          </div>
        ) : (
          // Welcome screen when no sessions
          <div className="h-full bg-white rounded-lg shadow-sm border border-gray-200 flex items-center justify-center">
            <div className="text-center text-gray-500">
              <div className="text-6xl mb-4">🚀</div>
              <h3 className="text-xl font-semibold mb-2">Welcome to Pulsar</h3>
              <p className="text-sm mb-6">Create your first terminal session</p>
              <div className="flex gap-4 justify-center">
                <button
                  onClick={createLocalSession}
                  className="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors font-medium"
                >
                  Local Terminal
                </button>
                <button
                  onClick={() => setIsDialogOpen(true)}
                  className="px-6 py-3 bg-accent-primary text-white rounded-lg hover:bg-green-600 transition-colors font-medium"
                >
                  SSH Connection
                </button>
              </div>

              {/* Keyboard shortcuts hint */}
              <div className="mt-8 text-xs text-gray-400">
                <p className="font-semibold mb-2">Keyboard Shortcuts:</p>
                <div className="flex flex-col gap-1">
                  <p><kbd className="px-2 py-1 bg-gray-100 rounded">Ctrl+T</kbd> New terminal</p>
                  <p><kbd className="px-2 py-1 bg-gray-100 rounded">Ctrl+W</kbd> Close terminal</p>
                  <p><kbd className="px-2 py-1 bg-gray-100 rounded">Ctrl+Tab</kbd> Next terminal</p>
                  <p><kbd className="px-2 py-1 bg-gray-100 rounded">Ctrl+Shift+Tab</kbd> Previous terminal</p>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Connection Dialog */}
      <ConnectionDialog
        isOpen={isDialogOpen}
        onClose={() => setIsDialogOpen(false)}
        onConnect={handleConnect}
      />

      {/* Session Restore Notification */}
      {showRestoreNotification && !showRestoreDialog && (
        <SessionRestoreNotification
          sessionCount={persistedSessionsToRestore.length}
          onRestoreAll={handleRestoreAll}
          onDismiss={handleDismissRestore}
          onManage={() => {
            setShowRestoreNotification(false)
            setShowRestoreDialog(true)
          }}
        />
      )}

      {/* Session Restore Dialog */}
      {showRestoreDialog && (
        <SessionRestoreDialog
          isOpen={showRestoreDialog}
          sessions={persistedSessionsToRestore.map((s) => ({
            id: s.id,
            name: s.name,
            type: s.type,
            host: s.sessionConfig?.host,
            username: s.sessionConfig?.username,
            lastActive: s.lastActive,
            hasVaultCredential: !!s.sessionConfig?.credentialId,
          }))}
          onRestore={handleRestoreSelected}
          onClose={() => {
            setShowRestoreDialog(false)
            setPersistedSessionsToRestore([])
          }}
        />
      )}
    </div>
  )
}
