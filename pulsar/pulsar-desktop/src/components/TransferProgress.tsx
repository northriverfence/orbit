import { TransferProgress as TransferProgressType } from '../lib/fileTransferClient'

interface TransferProgressProps {
  progress: TransferProgressType
  onCancel?: (transferId: string) => void
  showDetails?: boolean
}

export default function TransferProgress({
  progress,
  onCancel,
  showDetails = true,
}: TransferProgressProps) {
  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
  }

  // Format speed
  const formatSpeed = (bytesPerSecond: number): string => {
    return `${formatFileSize(bytesPerSecond)}/s`
  }

  // Format time
  const formatTime = (seconds: number): string => {
    if (!isFinite(seconds)) return 'calculating...'
    if (seconds < 1) return '< 1s'
    if (seconds < 60) return `${Math.round(seconds)}s`
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = Math.round(seconds % 60)
    if (minutes < 60) {
      return `${minutes}m ${remainingSeconds}s`
    }
    const hours = Math.floor(minutes / 60)
    const remainingMinutes = minutes % 60
    return `${hours}h ${remainingMinutes}m`
  }

  const handleCancel = () => {
    if (onCancel) {
      onCancel(progress.transferId)
    }
  }

  return (
    <div className="w-full">
      {/* File name and percentage */}
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-gray-900 truncate flex-1">
          {progress.fileName}
        </span>
        <span className="text-sm font-semibold text-blue-600 ml-2">
          {progress.percentage.toFixed(1)}%
        </span>
      </div>

      {/* Progress bar */}
      <div className="relative w-full bg-gray-200 rounded-full h-3 mb-2">
        <div
          className="absolute top-0 left-0 h-full bg-gradient-to-r from-blue-500 to-blue-600 rounded-full transition-all duration-300 ease-out"
          style={{ width: `${Math.min(100, progress.percentage)}%` }}
        >
          {/* Shimmer effect for active transfers */}
          <div className="absolute inset-0 bg-gradient-to-r from-transparent via-white to-transparent opacity-30 animate-shimmer" />
        </div>
      </div>

      {/* Details */}
      {showDetails && (
        <div className="flex items-center justify-between text-xs text-gray-600">
          <div className="flex items-center space-x-3">
            {/* Size */}
            <span>
              {formatFileSize(progress.transferredBytes)} / {formatFileSize(progress.totalBytes)}
            </span>

            {/* Chunks */}
            <span className="text-gray-400">•</span>
            <span>
              {progress.chunksCompleted} / {progress.totalChunks} chunks
            </span>
          </div>

          <div className="flex items-center space-x-3">
            {/* Speed */}
            <span className="font-medium text-blue-600">
              {formatSpeed(progress.speed)}
            </span>

            {/* ETA */}
            <span className="text-gray-400">•</span>
            <span>
              {formatTime(progress.estimatedTimeRemaining)} remaining
            </span>

            {/* Cancel button */}
            {onCancel && (
              <>
                <span className="text-gray-400">•</span>
                <button
                  onClick={handleCancel}
                  className="text-red-600 hover:text-red-700 font-medium transition-colors"
                >
                  Cancel
                </button>
              </>
            )}
          </div>
        </div>
      )}
    </div>
  )
}

// Add shimmer animation to global CSS or use Tailwind config
// @keyframes shimmer {
//   0% { transform: translateX(-100%); }
//   100% { transform: translateX(100%); }
// }
