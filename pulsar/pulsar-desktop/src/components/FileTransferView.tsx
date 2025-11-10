import { useState, useCallback } from 'react'
import FileUploadZone from './FileUploadZone'
import TransferQueue, { QueuedTransfer } from './TransferQueue'
import { TransferProgress, TransferResult, TransferError } from '../lib/fileTransferClient'

export default function FileTransferView() {
  const [transfers, setTransfers] = useState<Map<string, QueuedTransfer>>(new Map())
  const [activeView, setActiveView] = useState<'upload' | 'queue'>('upload')

  // Handle upload start - add files to queue
  const handleUploadStart = useCallback((files: File[]) => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      files.forEach(file => {
        const id = `${file.name}-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
        newMap.set(id, {
          id,
          fileName: file.name,
          fileSize: file.size,
          status: 'queued',
          progress: null,
          result: null,
          error: null,
          addedAt: Date.now(),
          startedAt: null,
          completedAt: null,
        })
      })
      return newMap
    })

    // Switch to queue view to show transfers
    setActiveView('queue')
  }, [])

  // Handle upload progress - update transfer progress
  const handleUploadProgress = useCallback((progress: TransferProgress) => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      // Find the transfer by matching progress data
      for (const [id, transfer] of newMap.entries()) {
        if (transfer.status === 'queued' || transfer.status === 'uploading') {
          newMap.set(id, {
            ...transfer,
            status: 'uploading',
            progress,
            startedAt: transfer.startedAt || Date.now(),
          })
          break // Update first matching transfer
        }
      }
      return newMap
    })
  }, [])

  // Handle upload complete - mark transfer as completed
  const handleUploadComplete = useCallback((result: TransferResult) => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      // Find the transfer by matching result data (we'll match by file name for now)
      for (const [id, transfer] of newMap.entries()) {
        if (transfer.status === 'uploading' && transfer.fileName === result.fileName) {
          newMap.set(id, {
            ...transfer,
            status: 'completed',
            result,
            completedAt: Date.now(),
          })
          break
        }
      }
      return newMap
    })
  }, [])

  // Handle upload error - mark transfer as failed
  const handleUploadError = useCallback((error: TransferError) => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      // Find the transfer by transfer ID
      const transfer = newMap.get(error.transferId)
      if (transfer) {
        newMap.set(error.transferId, {
          ...transfer,
          status: 'failed',
          error,
        })
      }
      return newMap
    })
  }, [])

  // Handle cancel transfer
  const handleCancelTransfer = useCallback((transferId: string) => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      const transfer = newMap.get(transferId)
      if (transfer) {
        newMap.set(transferId, {
          ...transfer,
          status: 'cancelled',
        })
      }
      return newMap
    })
  }, [])

  // Handle retry transfer
  const handleRetryTransfer = useCallback((transferId: string) => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      const transfer = newMap.get(transferId)
      if (transfer) {
        newMap.set(transferId, {
          ...transfer,
          status: 'queued',
          progress: null,
          result: null,
          error: null,
          startedAt: null,
          completedAt: null,
        })
      }
      return newMap
    })
    // TODO: Actually retry the upload
  }, [])

  // Handle clear completed transfers
  const handleClearCompleted = useCallback(() => {
    setTransfers(prev => {
      const newMap = new Map(prev)
      for (const [id, transfer] of newMap.entries()) {
        if (transfer.status === 'completed') {
          newMap.delete(id)
        }
      }
      return newMap
    })
  }, [])

  // Handle pause all (placeholder)
  const handlePauseAll = useCallback(() => {
    console.log('Pause all transfers - not yet implemented')
    // TODO: Implement pause functionality
  }, [])

  // Handle resume all (placeholder)
  const handleResumeAll = useCallback(() => {
    console.log('Resume all transfers - not yet implemented')
    // TODO: Implement resume functionality
  }, [])

  return (
    <div className="w-full h-full flex flex-col bg-gray-50">
      {/* Header with view toggle */}
      <div className="flex-shrink-0 bg-white border-b border-gray-200 px-6 py-4">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-bold text-gray-900">File Transfer</h1>

          {/* View toggle */}
          <div className="flex items-center space-x-2 bg-gray-100 rounded-lg p-1">
            <button
              onClick={() => setActiveView('upload')}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                activeView === 'upload'
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              Upload
            </button>
            <button
              onClick={() => setActiveView('queue')}
              className={`px-4 py-2 text-sm font-medium rounded-md transition-colors ${
                activeView === 'queue'
                  ? 'bg-white text-blue-600 shadow-sm'
                  : 'text-gray-600 hover:text-gray-900'
              }`}
            >
              Queue
              {transfers.size > 0 && (
                <span className="ml-2 px-2 py-0.5 text-xs bg-blue-600 text-white rounded-full">
                  {transfers.size}
                </span>
              )}
            </button>
          </div>
        </div>

        {/* Quick stats */}
        {transfers.size > 0 && (
          <div className="mt-4 flex items-center space-x-6 text-sm">
            <div className="flex items-center space-x-2">
              <span className="text-gray-500">Active:</span>
              <span className="font-medium text-blue-600">
                {Array.from(transfers.values()).filter(t => t.status === 'queued' || t.status === 'uploading').length}
              </span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-gray-500">Completed:</span>
              <span className="font-medium text-green-600">
                {Array.from(transfers.values()).filter(t => t.status === 'completed').length}
              </span>
            </div>
            <div className="flex items-center space-x-2">
              <span className="text-gray-500">Failed:</span>
              <span className="font-medium text-red-600">
                {Array.from(transfers.values()).filter(t => t.status === 'failed' || t.status === 'cancelled').length}
              </span>
            </div>
          </div>
        )}
      </div>

      {/* Main content area */}
      <div className="flex-1 overflow-hidden">
        {activeView === 'upload' ? (
          <div className="h-full p-6">
            <FileUploadZone
              onUploadStart={handleUploadStart}
              onUploadProgress={handleUploadProgress}
              onUploadComplete={handleUploadComplete}
              onUploadError={handleUploadError}
            />
          </div>
        ) : (
          <TransferQueue
            transfers={transfers}
            onCancelTransfer={handleCancelTransfer}
            onRetryTransfer={handleRetryTransfer}
            onClearCompleted={handleClearCompleted}
            onPauseAll={handlePauseAll}
            onResumeAll={handleResumeAll}
          />
        )}
      </div>
    </div>
  )
}
