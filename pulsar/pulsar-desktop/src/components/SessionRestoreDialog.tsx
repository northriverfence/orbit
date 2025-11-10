import { useState, useRef } from 'react'
import { useFocusTrap } from '../hooks/useFocusTrap'
import { useKeyboardShortcut, SHORTCUTS } from '../hooks/useKeyboardShortcut'

interface SessionInfo {
  id: string
  name: string
  type: 'local' | 'ssh' | 'serial'
  host?: string
  username?: string
  lastActive: string
  hasVaultCredential: boolean
}

interface SessionRestoreDialogProps {
  isOpen: boolean
  sessions: SessionInfo[]
  onRestore: (sessionIds: string[]) => void
  onClose: () => void
}

export default function SessionRestoreDialog({
  isOpen,
  sessions,
  onRestore,
  onClose,
}: SessionRestoreDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null)
  const [selectedIds, setSelectedIds] = useState<Set<string>>(
    new Set(sessions.map((s) => s.id))
  )
  const [isRestoring, setIsRestoring] = useState(false)

  // Keyboard navigation
  useFocusTrap(dialogRef, isOpen)
  useKeyboardShortcut(SHORTCUTS.ESCAPE, onClose, isOpen && !isRestoring)

  const toggleSession = (id: string) => {
    setSelectedIds((prev) => {
      const next = new Set(prev)
      if (next.has(id)) {
        next.delete(id)
      } else {
        next.add(id)
      }
      return next
    })
  }

  const selectAll = () => {
    setSelectedIds(new Set(sessions.map((s) => s.id)))
  }

  const selectNone = () => {
    setSelectedIds(new Set())
  }

  const handleRestore = async () => {
    setIsRestoring(true)
    try {
      await onRestore(Array.from(selectedIds))
      onClose()
    } finally {
      setIsRestoring(false)
    }
  }

  const formatTime = (isoString: string) => {
    const date = new Date(isoString)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMs / 3600000)
    const diffDays = Math.floor(diffMs / 86400000)

    if (diffMins < 1) return 'Just now'
    if (diffMins < 60) return `${diffMins}m ago`
    if (diffHours < 24) return `${diffHours}h ago`
    if (diffDays < 7) return `${diffDays}d ago`
    return date.toLocaleDateString()
  }

  if (!isOpen) return null

  return (
    <div className="fixed inset-0 bg-black bg-opacity-60 flex items-center justify-center z-50 modal-backdrop">
      <div
        ref={dialogRef}
        className="bg-white rounded-lg shadow-2xl w-full max-w-2xl max-h-[80vh] flex flex-col modal-content"
      >
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div>
            <h2 className="text-2xl font-bold text-gray-900 flex items-center space-x-2">
              <span>⏮️</span>
              <span>Restore Sessions</span>
            </h2>
            <p className="text-sm text-gray-600 mt-1">
              Select which sessions to restore
            </p>
          </div>
          <button
            onClick={onClose}
            disabled={isRestoring}
            className="text-gray-400 hover:text-gray-600 transition-colors disabled:opacity-50"
            aria-label="Close"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Bulk Actions */}
        <div className="px-6 py-3 bg-gray-50 border-b border-gray-200 flex items-center justify-between">
          <div className="text-sm text-gray-600">
            {selectedIds.size} of {sessions.length} selected
          </div>
          <div className="flex space-x-2">
            <button
              onClick={selectAll}
              disabled={isRestoring}
              className="text-xs text-blue-600 hover:text-blue-700 font-medium transition-colors disabled:opacity-50"
            >
              Select All
            </button>
            <span className="text-gray-300">|</span>
            <button
              onClick={selectNone}
              disabled={isRestoring}
              className="text-xs text-blue-600 hover:text-blue-700 font-medium transition-colors disabled:opacity-50"
            >
              Select None
            </button>
          </div>
        </div>

        {/* Session List */}
        <div className="flex-1 overflow-y-auto p-6">
          <div className="space-y-3">
            {sessions.map((session) => {
              const isSelected = selectedIds.has(session.id)

              return (
                <div
                  key={session.id}
                  onClick={() => !isRestoring && toggleSession(session.id)}
                  className={`
                    flex items-center justify-between p-4 rounded-lg border-2 cursor-pointer
                    transition-all hover-lift
                    ${
                      isSelected
                        ? 'border-blue-500 bg-blue-50'
                        : 'border-gray-200 bg-white hover:border-gray-300'
                    }
                    ${isRestoring ? 'opacity-50 cursor-not-allowed' : ''}
                  `}
                >
                  {/* Checkbox */}
                  <div className="flex items-center space-x-4 flex-1">
                    <div
                      className={`
                        w-5 h-5 rounded border-2 flex items-center justify-center transition-colors
                        ${
                          isSelected
                            ? 'bg-blue-600 border-blue-600'
                            : 'bg-white border-gray-300'
                        }
                      `}
                    >
                      {isSelected && (
                        <svg
                          className="w-3 h-3 text-white"
                          fill="none"
                          stroke="currentColor"
                          viewBox="0 0 24 24"
                        >
                          <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={3}
                            d="M5 13l4 4L19 7"
                          />
                        </svg>
                      )}
                    </div>

                    {/* Session Info */}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center space-x-2">
                        <span className="text-lg">
                          {session.type === 'local' ? '💻' : '🖥️'}
                        </span>
                        <div className="flex-1 min-w-0">
                          <div className="font-semibold text-gray-900 truncate">
                            {session.name}
                          </div>
                          {session.type === 'ssh' && session.host && (
                            <div className="text-sm text-gray-600 truncate">
                              {session.username ? `${session.username}@` : ''}
                              {session.host}
                            </div>
                          )}
                        </div>
                      </div>
                    </div>

                    {/* Status Badges */}
                    <div className="flex items-center space-x-2">
                      {session.hasVaultCredential && (
                        <span className="px-2 py-1 text-xs bg-green-100 text-green-700 rounded-full font-medium">
                          🔐 Has Credentials
                        </span>
                      )}
                      <span className="text-xs text-gray-500">
                        {formatTime(session.lastActive)}
                      </span>
                    </div>
                  </div>
                </div>
              )
            })}
          </div>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 bg-gray-50 border-t border-gray-200">
          <div className="flex items-center justify-between">
            <div className="text-sm text-gray-600">
              {selectedIds.size === 0 ? (
                'Select sessions to restore'
              ) : selectedIds.size === sessions.length ? (
                'All sessions will be restored'
              ) : (
                `${selectedIds.size} session${selectedIds.size > 1 ? 's' : ''} will be restored`
              )}
            </div>
            <div className="flex space-x-3">
              <button
                onClick={onClose}
                disabled={isRestoring}
                className="px-4 py-2 text-gray-700 bg-gray-100 rounded-md hover:bg-gray-200 transition-colors btn-press disabled:opacity-50"
              >
                Cancel
              </button>
              <button
                onClick={handleRestore}
                disabled={selectedIds.size === 0 || isRestoring}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed btn-press"
              >
                {isRestoring ? (
                  <span className="flex items-center">
                    <svg
                      className="animate-spin -ml-1 mr-2 h-4 w-4 text-white"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        className="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        strokeWidth="4"
                      />
                      <path
                        className="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                      />
                    </svg>
                    Restoring...
                  </span>
                ) : (
                  `Restore ${selectedIds.size > 0 ? selectedIds.size : ''} Session${
                    selectedIds.size !== 1 ? 's' : ''
                  }`
                )}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
