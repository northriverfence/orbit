import { useState } from 'react'
import Terminal from './Terminal'
import PulsarTerminal from './PulsarTerminal'
import ConnectionDialog, { ConnectionConfig } from './ConnectionDialog'

interface SessionConfig {
  id: string
  host: string
  port: number
  username: string
  password?: string
}

type TerminalMode = 'ssh' | 'local' | null

export default function MainContent() {
  const [activeSession, setActiveSession] = useState<SessionConfig | null>(null)
  const [isDialogOpen, setIsDialogOpen] = useState(false)
  const [terminalMode, setTerminalMode] = useState<TerminalMode>(null)

  const handleConnect = (config: ConnectionConfig) => {
    const sessionId = `${config.username}@${config.host}:${config.port}`
    setActiveSession({
      id: sessionId,
      host: config.host,
      port: config.port,
      username: config.username,
      password: config.authType === 'password' ? config.password : undefined,
    })
    setTerminalMode('ssh')
  }

  const handleDisconnect = () => {
    setActiveSession(null)
    setTerminalMode(null)
  }

  const handleOpenLocal = () => {
    setTerminalMode('local')
    setActiveSession(null)
  }

  return (
    <div className="flex-1 flex flex-col bg-gray-50">
      {/* Header */}
      <div className="h-14 bg-white border-b border-gray-200 flex items-center justify-between px-6">
        <h2 className="text-lg font-semibold text-gray-800">
          {terminalMode === 'ssh' && activeSession ? activeSession.id :
           terminalMode === 'local' ? 'Local Terminal' : 'Default Workspace'}
        </h2>
        {terminalMode && (
          <button
            onClick={handleDisconnect}
            className="px-3 py-1.5 text-sm text-red-600 hover:text-red-700 hover:bg-red-50 rounded-md transition-colors"
          >
            Close Terminal
          </button>
        )}
      </div>

      {/* Main Area */}
      <div className="flex-1 p-4">
        {terminalMode === 'ssh' && activeSession ? (
          <Terminal
            sessionId={activeSession.id}
            host={activeSession.host}
            port={activeSession.port}
            username={activeSession.username}
            password={activeSession.password}
          />
        ) : terminalMode === 'local' ? (
          <div className="h-full bg-[#1e1e1e] rounded-lg shadow-sm overflow-hidden">
            <PulsarTerminal />
          </div>
        ) : (
          <div className="h-full bg-white rounded-lg shadow-sm border border-gray-200 flex items-center justify-center">
            <div className="text-center text-gray-500">
              <div className="text-6xl mb-4">🚀</div>
              <h3 className="text-xl font-semibold mb-2">Welcome to Pulsar</h3>
              <p className="text-sm mb-6">Choose your terminal type</p>
              <div className="flex gap-4 justify-center">
                <button
                  onClick={handleOpenLocal}
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
    </div>
  )
}
