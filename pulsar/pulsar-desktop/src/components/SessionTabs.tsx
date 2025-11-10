import React, { useState, useRef, useEffect } from 'react'

export interface Session {
  id: string
  name: string
  type: 'ssh' | 'local' | 'serial'
  config?: any
  active: boolean
}

interface SessionTabsProps {
  sessions: Session[]
  activeSessionId: string | null
  onSessionSelect: (sessionId: string) => void
  onSessionClose: (sessionId: string) => void
  onSessionRename: (sessionId: string, newName: string) => void
  onNewSession: () => void
}

export default function SessionTabs({
  sessions,
  activeSessionId,
  onSessionSelect,
  onSessionClose,
  onSessionRename,
  onNewSession,
}: SessionTabsProps) {
  const [contextMenu, setContextMenu] = useState<{
    x: number
    y: number
    sessionId: string
  } | null>(null)
  const [renamingId, setRenamingId] = useState<string | null>(null)
  const [renameValue, setRenameValue] = useState('')
  const renameInputRef = useRef<HTMLInputElement>(null)

  // Close context menu when clicking outside
  useEffect(() => {
    const handleClick = () => setContextMenu(null)
    document.addEventListener('click', handleClick)
    return () => document.removeEventListener('click', handleClick)
  }, [])

  // Focus rename input when renaming starts
  useEffect(() => {
    if (renamingId && renameInputRef.current) {
      renameInputRef.current.focus()
      renameInputRef.current.select()
    }
  }, [renamingId])

  // Handle keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl+Tab or Ctrl+Shift+Tab for tab switching
      if (e.ctrlKey && e.key === 'Tab') {
        e.preventDefault()
        const currentIndex = sessions.findIndex((s) => s.id === activeSessionId)
        if (currentIndex === -1) return

        let nextIndex
        if (e.shiftKey) {
          // Previous tab
          nextIndex = currentIndex === 0 ? sessions.length - 1 : currentIndex - 1
        } else {
          // Next tab
          nextIndex = (currentIndex + 1) % sessions.length
        }

        onSessionSelect(sessions[nextIndex].id)
      }

      // Ctrl+W to close current tab
      if (e.ctrlKey && e.key === 'w' && activeSessionId) {
        e.preventDefault()
        onSessionClose(activeSessionId)
      }

      // Ctrl+T to create new tab
      if (e.ctrlKey && e.key === 't') {
        e.preventDefault()
        onNewSession()
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [sessions, activeSessionId, onSessionSelect, onSessionClose, onNewSession])

  const handleContextMenu = (e: React.MouseEvent, sessionId: string) => {
    e.preventDefault()
    setContextMenu({ x: e.clientX, y: e.clientY, sessionId })
  }

  const handleRename = (sessionId: string) => {
    const session = sessions.find((s) => s.id === sessionId)
    if (session) {
      setRenamingId(sessionId)
      setRenameValue(session.name)
      setContextMenu(null)
    }
  }

  const handleRenameSubmit = () => {
    if (renamingId && renameValue.trim()) {
      onSessionRename(renamingId, renameValue.trim())
    }
    setRenamingId(null)
    setRenameValue('')
  }

  const handleRenameKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleRenameSubmit()
    } else if (e.key === 'Escape') {
      setRenamingId(null)
      setRenameValue('')
    }
  }

  const getSessionIcon = (type: Session['type']) => {
    switch (type) {
      case 'ssh':
        return '🌐'
      case 'local':
        return '💻'
      case 'serial':
        return '🔌'
      default:
        return '📟'
    }
  }

  const getStatusColor = (session: Session) => {
    if (session.active) {
      return 'bg-green-500'
    }
    return 'bg-gray-400'
  }

  return (
    <div className="h-10 bg-gray-100 border-b border-gray-300 flex items-center px-2 gap-1">
      {/* Session Tabs */}
      <div className="flex-1 flex items-center gap-1 overflow-x-auto scrollbar-thin">
        {sessions.map((session) => (
          <div
            key={session.id}
            className={`
              group relative flex items-center gap-2 px-3 py-1.5 rounded-t-md cursor-pointer
              transition-all duration-150 min-w-[120px] max-w-[200px]
              ${
                session.id === activeSessionId
                  ? 'bg-white border-t-2 border-blue-500 shadow-sm'
                  : 'bg-gray-200 hover:bg-gray-250 border-t-2 border-transparent'
              }
            `}
            onClick={() => onSessionSelect(session.id)}
            onContextMenu={(e) => handleContextMenu(e, session.id)}
          >
            {/* Icon */}
            <span className="text-sm">{getSessionIcon(session.type)}</span>

            {/* Status indicator */}
            <div className={`w-2 h-2 rounded-full ${getStatusColor(session)}`} />

            {/* Session name */}
            {renamingId === session.id ? (
              <input
                ref={renameInputRef}
                type="text"
                value={renameValue}
                onChange={(e) => setRenameValue(e.target.value)}
                onBlur={handleRenameSubmit}
                onKeyDown={handleRenameKeyDown}
                className="flex-1 px-1 py-0.5 text-sm border border-blue-500 rounded focus:outline-none"
                onClick={(e) => e.stopPropagation()}
              />
            ) : (
              <span className="flex-1 text-sm font-medium truncate">
                {session.name}
              </span>
            )}

            {/* Close button */}
            <button
              onClick={(e) => {
                e.stopPropagation()
                onSessionClose(session.id)
              }}
              className="opacity-0 group-hover:opacity-100 transition-opacity p-0.5 hover:bg-gray-300 rounded"
            >
              <svg
                className="w-3.5 h-3.5 text-gray-600"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M6 18L18 6M6 6l12 12"
                />
              </svg>
            </button>
          </div>
        ))}
      </div>

      {/* New tab button */}
      <button
        onClick={onNewSession}
        className="p-1.5 hover:bg-gray-200 rounded transition-colors"
        title="New terminal (Ctrl+T)"
      >
        <svg
          className="w-4 h-4 text-gray-600"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M12 4v16m8-8H4"
          />
        </svg>
      </button>

      {/* Context Menu */}
      {contextMenu && (
        <div
          className="fixed bg-white rounded-lg shadow-lg border border-gray-200 py-1 z-50 min-w-[150px]"
          style={{ top: contextMenu.y, left: contextMenu.x }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            onClick={() => handleRename(contextMenu.sessionId)}
            className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 transition-colors"
          >
            Rename
          </button>
          <button
            onClick={() => {
              // TODO: Implement duplicate
              setContextMenu(null)
            }}
            className="w-full text-left px-4 py-2 text-sm hover:bg-gray-100 transition-colors"
          >
            Duplicate
          </button>
          <div className="h-px bg-gray-200 my-1" />
          <button
            onClick={() => {
              onSessionClose(contextMenu.sessionId)
              setContextMenu(null)
            }}
            className="w-full text-left px-4 py-2 text-sm hover:bg-red-50 text-red-600 transition-colors"
          >
            Close
          </button>
        </div>
      )}
    </div>
  )
}
