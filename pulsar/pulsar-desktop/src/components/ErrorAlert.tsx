interface ErrorAlertProps {
  title?: string
  message: string
  details?: string
  onRetry?: () => void
  onDismiss?: () => void
  type?: 'error' | 'warning'
}

export default function ErrorAlert({
  title,
  message,
  details,
  onRetry,
  onDismiss,
  type = 'error',
}: ErrorAlertProps) {
  const isError = type === 'error'

  const bgColor = isError ? 'bg-red-50' : 'bg-yellow-50'
  const borderColor = isError ? 'border-red-200' : 'border-yellow-200'
  const textColor = isError ? 'text-red-800' : 'text-yellow-800'
  const iconColor = isError ? 'text-red-600' : 'text-yellow-600'
  const buttonColor = isError
    ? 'bg-red-600 hover:bg-red-700'
    : 'bg-yellow-600 hover:bg-yellow-700'

  return (
    <div className={`${bgColor} border ${borderColor} rounded-lg p-4 animate-slideInFromTop`}>
      <div className="flex items-start">
        {/* Icon */}
        <div className="flex-shrink-0">
          <svg
            className={`w-5 h-5 ${iconColor}`}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            {isError ? (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            ) : (
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
              />
            )}
          </svg>
        </div>

        {/* Content */}
        <div className="ml-3 flex-1">
          {title && (
            <h3 className={`text-sm font-semibold ${textColor} mb-1`}>
              {title}
            </h3>
          )}
          <p className={`text-sm ${textColor}`}>{message}</p>

          {details && (
            <details className="mt-2">
              <summary className={`text-xs font-medium ${textColor} cursor-pointer hover:underline`}>
                Show details
              </summary>
              <pre className="mt-1 text-xs bg-white bg-opacity-50 rounded p-2 overflow-auto max-h-32 font-mono">
                {details}
              </pre>
            </details>
          )}

          {/* Actions */}
          {(onRetry || onDismiss) && (
            <div className="mt-3 flex gap-2">
              {onRetry && (
                <button
                  onClick={onRetry}
                  className={`px-3 py-1.5 text-xs font-medium text-white ${buttonColor} rounded transition-colors btn-press`}
                >
                  Try Again
                </button>
              )}
              {onDismiss && (
                <button
                  onClick={onDismiss}
                  className="px-3 py-1.5 text-xs font-medium text-gray-700 bg-white border border-gray-300 rounded hover:bg-gray-50 transition-colors btn-press"
                >
                  Dismiss
                </button>
              )}
            </div>
          )}
        </div>

        {/* Close button */}
        {onDismiss && (
          <div className="ml-3 flex-shrink-0">
            <button
              onClick={onDismiss}
              className={`${textColor} hover:opacity-75 transition-opacity`}
            >
              <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
