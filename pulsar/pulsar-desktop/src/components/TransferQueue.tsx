import { useState } from 'react'
import { TransferProgress as TransferProgressType, TransferResult, TransferError } from '../lib/fileTransferClient'
import TransferProgress from './TransferProgress'

export interface QueuedTransfer {
  id: string
  fileName: string
  fileSize: number
  status: 'queued' | 'uploading' | 'completed' | 'failed' | 'cancelled'
  progress: TransferProgressType | null
  result: TransferResult | null
  error: TransferError | null
  addedAt: number
  startedAt: number | null
  completedAt: number | null
}

interface TransferQueueProps {
  transfers: Map<string, QueuedTransfer>
  onCancelTransfer?: (transferId: string) => void
  onRetryTransfer?: (transferId: string) => void
  onClearCompleted?: () => void
  onPauseAll?: () => void
  onResumeAll?: () => void
  maxVisible?: number
}

type FilterType = 'all' | 'active' | 'completed' | 'failed'

export default function TransferQueue({
  transfers,
  onCancelTransfer,
  onRetryTransfer,
  onClearCompleted,
  onPauseAll,
  onResumeAll,
  maxVisible = 10,
}: TransferQueueProps) {
  const [filter, setFilter] = useState<FilterType>('all')
  const [selectedTransfer, setSelectedTransfer] = useState<QueuedTransfer | null>(null)
  const [showDetailsModal, setShowDetailsModal] = useState(false)

  // Convert map to array and sort by addedAt (newest first)
  const transfersArray = Array.from(transfers.values()).sort((a, b) => b.addedAt - a.addedAt)

  // Filter transfers based on selected filter
  const filteredTransfers = transfersArray.filter((transfer) => {
    switch (filter) {
      case 'active':
        return transfer.status === 'queued' || transfer.status === 'uploading'
      case 'completed':
        return transfer.status === 'completed'
      case 'failed':
        return transfer.status === 'failed' || transfer.status === 'cancelled'
      default:
        return true
    }
  })

  // Calculate statistics
  const stats = {
    total: transfersArray.length,
    active: transfersArray.filter((t) => t.status === 'queued' || t.status === 'uploading').length,
    completed: transfersArray.filter((t) => t.status === 'completed').length,
    failed: transfersArray.filter((t) => t.status === 'failed' || t.status === 'cancelled').length,
  }

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
  }

  // Format duration
  const formatDuration = (ms: number): string => {
    const seconds = Math.floor(ms / 1000)
    if (seconds < 60) return `${seconds}s`
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = seconds % 60
    if (minutes < 60) return `${minutes}m ${remainingSeconds}s`
    const hours = Math.floor(minutes / 60)
    const remainingMinutes = minutes % 60
    return `${hours}h ${remainingMinutes}m`
  }

  // Open details modal
  const handleShowDetails = (transfer: QueuedTransfer) => {
    setSelectedTransfer(transfer)
    setShowDetailsModal(true)
  }

  // Close details modal
  const handleCloseDetails = () => {
    setShowDetailsModal(false)
    setSelectedTransfer(null)
  }

  return (
    <div className="w-full h-full flex flex-col bg-gray-50">
      {/* Header */}
      <div className="flex-shrink-0 bg-white border-b border-gray-200 p-4">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-xl font-bold text-gray-900">Transfer Queue</h2>
          <div className="flex items-center space-x-2">
            {onPauseAll && stats.active > 0 && (
              <button
                onClick={onPauseAll}
                className="px-3 py-1 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-md transition-colors"
              >
                Pause All
              </button>
            )}
            {onResumeAll && (
              <button
                onClick={onResumeAll}
                className="px-3 py-1 text-sm font-medium text-blue-700 bg-blue-100 hover:bg-blue-200 rounded-md transition-colors"
              >
                Resume All
              </button>
            )}
            {onClearCompleted && stats.completed > 0 && (
              <button
                onClick={onClearCompleted}
                className="px-3 py-1 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-md transition-colors"
              >
                Clear Completed
              </button>
            )}
          </div>
        </div>

        {/* Statistics */}
        <div className="flex items-center space-x-6 text-sm">
          <div className="flex items-center space-x-2">
            <span className="font-medium text-gray-700">Total:</span>
            <span className="text-gray-900">{stats.total}</span>
          </div>
          <div className="flex items-center space-x-2">
            <span className="font-medium text-blue-700">Active:</span>
            <span className="text-blue-900">{stats.active}</span>
          </div>
          <div className="flex items-center space-x-2">
            <span className="font-medium text-green-700">Completed:</span>
            <span className="text-green-900">{stats.completed}</span>
          </div>
          <div className="flex items-center space-x-2">
            <span className="font-medium text-red-700">Failed:</span>
            <span className="text-red-900">{stats.failed}</span>
          </div>
        </div>

        {/* Filter tabs */}
        <div className="flex items-center space-x-1 mt-4">
          {(['all', 'active', 'completed', 'failed'] as FilterType[]).map((filterType) => (
            <button
              key={filterType}
              onClick={() => setFilter(filterType)}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                filter === filterType
                  ? 'bg-blue-600 text-white'
                  : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
              }`}
            >
              {filterType.charAt(0).toUpperCase() + filterType.slice(1)}
            </button>
          ))}
        </div>
      </div>

      {/* Transfer list */}
      <div className="flex-1 overflow-y-auto p-4">
        {filteredTransfers.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-500">
            <div className="text-6xl mb-4">📋</div>
            <p className="text-lg font-medium">No transfers to display</p>
            <p className="text-sm mt-1">
              {filter === 'all' ? 'Upload some files to get started' : `No ${filter} transfers`}
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredTransfers.slice(0, maxVisible).map((transfer) => (
              <div
                key={transfer.id}
                className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm hover:shadow-md transition-shadow"
              >
                {/* Transfer header */}
                <div className="flex items-start justify-between mb-3">
                  <div className="flex-1 min-w-0 mr-4">
                    <p className="text-sm font-medium text-gray-900 truncate">{transfer.fileName}</p>
                    <p className="text-xs text-gray-500 mt-1">{formatFileSize(transfer.fileSize)}</p>
                  </div>

                  {/* Status badge */}
                  <span
                    className={`flex-shrink-0 px-2 py-1 text-xs font-medium rounded-full ${
                      transfer.status === 'queued'
                        ? 'bg-gray-100 text-gray-600'
                        : transfer.status === 'uploading'
                        ? 'bg-blue-100 text-blue-700'
                        : transfer.status === 'completed'
                        ? 'bg-green-100 text-green-700'
                        : transfer.status === 'failed'
                        ? 'bg-red-100 text-red-700'
                        : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    {transfer.status === 'queued' && '⏱ Queued'}
                    {transfer.status === 'uploading' && '⬆ Uploading'}
                    {transfer.status === 'completed' && '✓ Completed'}
                    {transfer.status === 'failed' && '✗ Failed'}
                    {transfer.status === 'cancelled' && '⊘ Cancelled'}
                  </span>
                </div>

                {/* Progress bar for active transfers */}
                {transfer.status === 'uploading' && transfer.progress && (
                  <div className="mb-3">
                    <TransferProgress
                      progress={transfer.progress}
                      onCancel={onCancelTransfer}
                      showDetails={true}
                    />
                  </div>
                )}

                {/* Completion info */}
                {transfer.status === 'completed' && transfer.result && (
                  <div className="mb-3 text-xs text-gray-600">
                    <p>Uploaded to: {transfer.result.savedPath}</p>
                    <p>
                      Duration: {formatDuration(transfer.result.duration)} • Average speed:{' '}
                      {formatFileSize(transfer.result.averageSpeed)}/s
                    </p>
                  </div>
                )}

                {/* Error info */}
                {transfer.status === 'failed' && transfer.error && (
                  <div className="mb-3 text-xs text-red-600 bg-red-50 p-2 rounded">
                    <p className="font-medium">Error: {transfer.error.errorType}</p>
                    <p>{transfer.error.errorMessage}</p>
                  </div>
                )}

                {/* Actions */}
                <div className="flex items-center justify-between pt-3 border-t border-gray-100">
                  <div className="text-xs text-gray-500">
                    Added {new Date(transfer.addedAt).toLocaleString()}
                  </div>
                  <div className="flex items-center space-x-2">
                    {/* Details button */}
                    <button
                      onClick={() => handleShowDetails(transfer)}
                      className="px-3 py-1 text-xs font-medium text-blue-600 hover:text-blue-700 hover:bg-blue-50 rounded transition-colors"
                    >
                      Details
                    </button>

                    {/* Retry button for failed transfers */}
                    {transfer.status === 'failed' && onRetryTransfer && (
                      <button
                        onClick={() => onRetryTransfer(transfer.id)}
                        className="px-3 py-1 text-xs font-medium text-green-600 hover:text-green-700 hover:bg-green-50 rounded transition-colors"
                      >
                        Retry
                      </button>
                    )}

                    {/* Cancel button for active transfers */}
                    {(transfer.status === 'queued' || transfer.status === 'uploading') &&
                      onCancelTransfer && (
                        <button
                          onClick={() => onCancelTransfer(transfer.id)}
                          className="px-3 py-1 text-xs font-medium text-red-600 hover:text-red-700 hover:bg-red-50 rounded transition-colors"
                        >
                          Cancel
                        </button>
                      )}
                  </div>
                </div>
              </div>
            ))}

            {/* Show more indicator */}
            {filteredTransfers.length > maxVisible && (
              <div className="text-center text-sm text-gray-500 py-2">
                Showing {maxVisible} of {filteredTransfers.length} transfers
              </div>
            )}
          </div>
        )}
      </div>

      {/* Details modal */}
      {showDetailsModal && selectedTransfer && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[80vh] overflow-y-auto">
            {/* Modal header */}
            <div className="flex items-center justify-between p-6 border-b border-gray-200">
              <h3 className="text-xl font-bold text-gray-900">Transfer Details</h3>
              <button
                onClick={handleCloseDetails}
                className="text-gray-400 hover:text-gray-600 transition-colors"
              >
                <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Modal body */}
            <div className="p-6 space-y-4">
              {/* File info */}
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-2">File Information</h4>
                <dl className="grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <dt className="text-gray-500">File Name</dt>
                    <dd className="text-gray-900 font-medium">{selectedTransfer.fileName}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-500">File Size</dt>
                    <dd className="text-gray-900 font-medium">
                      {formatFileSize(selectedTransfer.fileSize)}
                    </dd>
                  </div>
                  <div>
                    <dt className="text-gray-500">Transfer ID</dt>
                    <dd className="text-gray-900 font-mono text-xs">{selectedTransfer.id}</dd>
                  </div>
                  <div>
                    <dt className="text-gray-500">Status</dt>
                    <dd className="text-gray-900 font-medium">{selectedTransfer.status}</dd>
                  </div>
                </dl>
              </div>

              {/* Timing info */}
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-2">Timing</h4>
                <dl className="grid grid-cols-2 gap-3 text-sm">
                  <div>
                    <dt className="text-gray-500">Added At</dt>
                    <dd className="text-gray-900">{new Date(selectedTransfer.addedAt).toLocaleString()}</dd>
                  </div>
                  {selectedTransfer.startedAt && (
                    <div>
                      <dt className="text-gray-500">Started At</dt>
                      <dd className="text-gray-900">
                        {new Date(selectedTransfer.startedAt).toLocaleString()}
                      </dd>
                    </div>
                  )}
                  {selectedTransfer.completedAt && (
                    <div>
                      <dt className="text-gray-500">Completed At</dt>
                      <dd className="text-gray-900">
                        {new Date(selectedTransfer.completedAt).toLocaleString()}
                      </dd>
                    </div>
                  )}
                  {selectedTransfer.startedAt && selectedTransfer.completedAt && (
                    <div>
                      <dt className="text-gray-500">Duration</dt>
                      <dd className="text-gray-900">
                        {formatDuration(selectedTransfer.completedAt - selectedTransfer.startedAt)}
                      </dd>
                    </div>
                  )}
                </dl>
              </div>

              {/* Progress info */}
              {selectedTransfer.progress && (
                <div>
                  <h4 className="text-sm font-medium text-gray-700 mb-2">Progress</h4>
                  <dl className="grid grid-cols-2 gap-3 text-sm">
                    <div>
                      <dt className="text-gray-500">Percentage</dt>
                      <dd className="text-gray-900 font-medium">
                        {selectedTransfer.progress.percentage.toFixed(2)}%
                      </dd>
                    </div>
                    <div>
                      <dt className="text-gray-500">Transferred</dt>
                      <dd className="text-gray-900">
                        {formatFileSize(selectedTransfer.progress.transferredBytes)} /{' '}
                        {formatFileSize(selectedTransfer.progress.totalBytes)}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-gray-500">Chunks</dt>
                      <dd className="text-gray-900">
                        {selectedTransfer.progress.chunksCompleted} / {selectedTransfer.progress.totalChunks}
                      </dd>
                    </div>
                    <div>
                      <dt className="text-gray-500">Speed</dt>
                      <dd className="text-gray-900">{formatFileSize(selectedTransfer.progress.speed)}/s</dd>
                    </div>
                  </dl>
                </div>
              )}

              {/* Result info */}
              {selectedTransfer.result && (
                <div>
                  <h4 className="text-sm font-medium text-gray-700 mb-2">Transfer Result</h4>
                  <dl className="space-y-2 text-sm">
                    <div>
                      <dt className="text-gray-500">Saved Path</dt>
                      <dd className="text-gray-900 font-mono text-xs break-all">
                        {selectedTransfer.result.savedPath}
                      </dd>
                    </div>
                    <div className="grid grid-cols-2 gap-3">
                      <div>
                        <dt className="text-gray-500">Duration</dt>
                        <dd className="text-gray-900">{formatDuration(selectedTransfer.result.duration)}</dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Average Speed</dt>
                        <dd className="text-gray-900">
                          {formatFileSize(selectedTransfer.result.averageSpeed)}/s
                        </dd>
                      </div>
                    </div>
                  </dl>
                </div>
              )}

              {/* Error info */}
              {selectedTransfer.error && (
                <div>
                  <h4 className="text-sm font-medium text-red-700 mb-2">Error Details</h4>
                  <div className="bg-red-50 border border-red-200 rounded p-3 space-y-2 text-sm">
                    <div>
                      <dt className="text-red-700 font-medium">Error Type</dt>
                      <dd className="text-red-900">{selectedTransfer.error.errorType}</dd>
                    </div>
                    <div>
                      <dt className="text-red-700 font-medium">Error Message</dt>
                      <dd className="text-red-900">{selectedTransfer.error.errorMessage}</dd>
                    </div>
                  </div>
                </div>
              )}
            </div>

            {/* Modal footer */}
            <div className="flex items-center justify-end space-x-3 p-6 border-t border-gray-200">
              {selectedTransfer.status === 'failed' && onRetryTransfer && (
                <button
                  onClick={() => {
                    onRetryTransfer(selectedTransfer.id)
                    handleCloseDetails()
                  }}
                  className="px-4 py-2 text-sm font-medium text-white bg-green-600 hover:bg-green-700 rounded-md transition-colors"
                >
                  Retry Transfer
                </button>
              )}
              <button
                onClick={handleCloseDetails}
                className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-md transition-colors"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
