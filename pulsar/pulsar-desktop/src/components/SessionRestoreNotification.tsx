import { useState } from 'react'

interface SessionRestoreNotificationProps {
  sessionCount: number
  onRestoreAll: () => void
  onDismiss: () => void
  onManage: () => void
}

export default function SessionRestoreNotification({
  sessionCount,
  onRestoreAll,
  onDismiss,
  onManage,
}: SessionRestoreNotificationProps) {
  const [isRestoring, setIsRestoring] = useState(false)

  const handleRestoreAll = async () => {
    setIsRestoring(true)
    try {
      await onRestoreAll()
    } finally {
      setIsRestoring(false)
    }
  }

  return (
    <div className="fixed top-4 right-4 z-50 animate-slideInFromRight">
      <div className="bg-white rounded-lg shadow-2xl border border-blue-200 w-96 overflow-hidden">
        {/* Header */}
        <div className="bg-gradient-to-r from-blue-500 to-blue-600 px-4 py-3">
          <div className="flex items-center space-x-2">
            <svg
              className="w-5 h-5 text-white"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
            <h3 className="text-white font-semibold">Sessions Found</h3>
          </div>
        </div>

        {/* Content */}
        <div className="p-4">
          <p className="text-gray-700 text-sm mb-3">
            {sessionCount === 1
              ? 'Found 1 previous session.'
              : `Found ${sessionCount} previous sessions.`}{' '}
            Would you like to restore them?
          </p>

          {/* Session Benefits */}
          <div className="bg-blue-50 border border-blue-100 rounded-md p-3 mb-4 text-xs text-blue-800">
            <div className="flex items-start space-x-2">
              <svg
                className="w-4 h-4 mt-0.5 flex-shrink-0"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2v-3a1 1 0 00-1-1H9z"
                  clipRule="evenodd"
                />
              </svg>
              <div>
                <strong>Note:</strong> Sessions with saved credentials in your vault will reconnect
                automatically. Others will prompt for credentials.
              </div>
            </div>
          </div>

          {/* Actions */}
          <div className="flex space-x-2">
            <button
              onClick={handleRestoreAll}
              disabled={isRestoring}
              className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed btn-press"
            >
              {isRestoring ? (
                <span className="flex items-center justify-center">
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
                'Restore All'
              )}
            </button>

            <button
              onClick={onManage}
              disabled={isRestoring}
              className="flex-1 px-4 py-2 bg-gray-100 text-gray-700 rounded-md hover:bg-gray-200 transition-colors font-medium disabled:opacity-50 btn-press"
            >
              Choose Sessions
            </button>
          </div>

          {/* Dismiss */}
          <button
            onClick={onDismiss}
            disabled={isRestoring}
            className="w-full mt-2 text-xs text-gray-500 hover:text-gray-700 transition-colors disabled:opacity-50"
          >
            Dismiss (start fresh)
          </button>
        </div>
      </div>
    </div>
  )
}
