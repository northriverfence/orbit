import LoadingSpinner from './LoadingSpinner'

interface LoadingOverlayProps {
  message?: string
  fullScreen?: boolean
  transparent?: boolean
}

export default function LoadingOverlay({
  message = 'Loading...',
  fullScreen = false,
  transparent = false,
}: LoadingOverlayProps) {
  const containerClasses = fullScreen
    ? 'fixed inset-0 z-50'
    : 'absolute inset-0 z-10'

  const bgClasses = transparent
    ? 'bg-white bg-opacity-70'
    : 'bg-white bg-opacity-95'

  return (
    <div className={`${containerClasses} ${bgClasses} flex items-center justify-center`}>
      <div className="text-center">
        <LoadingSpinner size="lg" />
        {message && (
          <p className="mt-4 text-sm font-medium text-gray-700">{message}</p>
        )}
      </div>
    </div>
  )
}
