import React, { useState, useRef, useCallback } from 'react'
import { FileTransferClient, TransferProgress, TransferResult, TransferError } from '../lib/fileTransferClient'

interface FileUploadZoneProps {
  webtransportUrl?: string
  onUploadStart?: (files: File[]) => void
  onUploadProgress?: (progress: TransferProgress) => void
  onUploadComplete?: (result: TransferResult) => void
  onUploadError?: (error: TransferError) => void
}

interface UploadingFile {
  file: File
  progress: TransferProgress | null
  result: TransferResult | null
  error: TransferError | null
  status: 'pending' | 'uploading' | 'completed' | 'failed'
}

export default function FileUploadZone({
  webtransportUrl = 'https://127.0.0.1:4433',
  onUploadStart,
  onUploadProgress,
  onUploadComplete,
  onUploadError,
}: FileUploadZoneProps) {
  const [isDragging, setIsDragging] = useState(false)
  const [uploadingFiles, setUploadingFiles] = useState<Map<string, UploadingFile>>(new Map())
  const fileInputRef = useRef<HTMLInputElement>(null)
  const clientRef = useRef<FileTransferClient | null>(null)

  // Initialize client
  if (!clientRef.current) {
    clientRef.current = new FileTransferClient(webtransportUrl)
  }

  // Handle drag events
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(true)
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)

    const files = Array.from(e.dataTransfer.files)
    if (files.length > 0) {
      handleFiles(files)
    }
  }, [])

  // Handle file selection from input
  const handleFileInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || [])
    if (files.length > 0) {
      handleFiles(files)
    }
    // Reset input so same file can be selected again
    if (fileInputRef.current) {
      fileInputRef.current.value = ''
    }
  }, [])

  // Upload files
  const handleFiles = useCallback(async (files: File[]) => {
    if (onUploadStart) {
      onUploadStart(files)
    }

    // Add files to uploading list
    setUploadingFiles(prev => {
      const newMap = new Map(prev)
      files.forEach(file => {
        const id = `${file.name}-${Date.now()}`
        newMap.set(id, {
          file,
          progress: null,
          result: null,
          error: null,
          status: 'pending',
        })
      })
      return newMap
    })

    // Upload each file
    for (const file of files) {
      const id = `${file.name}-${Date.now()}`

      try {
        // Update status to uploading
        setUploadingFiles(prev => {
          const newMap = new Map(prev)
          const fileData = newMap.get(id)
          if (fileData) {
            newMap.set(id, { ...fileData, status: 'uploading' })
          }
          return newMap
        })

        const result = await clientRef.current!.uploadFile(file, {
          onProgress: (progress) => {
            setUploadingFiles(prev => {
              const newMap = new Map(prev)
              const fileData = newMap.get(id)
              if (fileData) {
                newMap.set(id, { ...fileData, progress })
              }
              return newMap
            })

            if (onUploadProgress) {
              onUploadProgress(progress)
            }
          },
        })

        // Update status to completed
        setUploadingFiles(prev => {
          const newMap = new Map(prev)
          const fileData = newMap.get(id)
          if (fileData) {
            newMap.set(id, { ...fileData, status: 'completed', result })
          }
          return newMap
        })

        if (onUploadComplete) {
          onUploadComplete(result)
        }
      } catch (error) {
        const transferError: TransferError = {
          transferId: id,
          errorType: 'upload_failed',
          errorMessage: error instanceof Error ? error.message : String(error),
        }

        // Update status to failed
        setUploadingFiles(prev => {
          const newMap = new Map(prev)
          const fileData = newMap.get(id)
          if (fileData) {
            newMap.set(id, { ...fileData, status: 'failed', error: transferError })
          }
          return newMap
        })

        if (onUploadError) {
          onUploadError(transferError)
        }
      }
    }
  }, [onUploadStart, onUploadProgress, onUploadComplete, onUploadError])

  // Open file picker
  const handleClick = useCallback(() => {
    fileInputRef.current?.click()
  }, [])

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
    if (seconds < 60) return `${Math.round(seconds)}s`
    const minutes = Math.floor(seconds / 60)
    const remainingSeconds = Math.round(seconds % 60)
    return `${minutes}m ${remainingSeconds}s`
  }

  return (
    <div className="w-full h-full flex flex-col">
      {/* Drop Zone */}
      <div
        className={`
          flex-shrink-0 border-2 border-dashed rounded-lg p-8 text-center cursor-pointer
          transition-colors duration-200
          ${isDragging
            ? 'border-blue-500 bg-blue-50'
            : 'border-gray-300 hover:border-gray-400 bg-gray-50 hover:bg-gray-100'
          }
        `}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
        onDragOver={handleDragOver}
        onDrop={handleDrop}
        onClick={handleClick}
      >
        <input
          ref={fileInputRef}
          type="file"
          multiple
          onChange={handleFileInputChange}
          className="hidden"
        />

        <div className="space-y-4">
          <div className="text-6xl">📁</div>
          <div>
            <p className="text-lg font-semibold text-gray-700">
              {isDragging ? 'Drop files here' : 'Drag & drop files here'}
            </p>
            <p className="text-sm text-gray-500 mt-1">
              or click to browse
            </p>
          </div>
          <div className="text-xs text-gray-400">
            Supports all file types • Maximum 100 GB per file
          </div>
        </div>
      </div>

      {/* File List */}
      {uploadingFiles.size > 0 && (
        <div className="flex-1 mt-6 overflow-y-auto">
          <h3 className="text-lg font-semibold mb-4">Transfers ({uploadingFiles.size})</h3>
          <div className="space-y-3">
            {Array.from(uploadingFiles.entries()).map(([id, fileData]) => (
              <div
                key={id}
                className="bg-white border border-gray-200 rounded-lg p-4 shadow-sm"
              >
                {/* File info */}
                <div className="flex items-start justify-between mb-3">
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 truncate">
                      {fileData.file.name}
                    </p>
                    <p className="text-xs text-gray-500">
                      {formatFileSize(fileData.file.size)}
                    </p>
                  </div>

                  {/* Status badge */}
                  <span
                    className={`
                      ml-3 px-2 py-1 text-xs font-medium rounded-full
                      ${fileData.status === 'pending' ? 'bg-gray-100 text-gray-600' : ''}
                      ${fileData.status === 'uploading' ? 'bg-blue-100 text-blue-700' : ''}
                      ${fileData.status === 'completed' ? 'bg-green-100 text-green-700' : ''}
                      ${fileData.status === 'failed' ? 'bg-red-100 text-red-700' : ''}
                    `}
                  >
                    {fileData.status === 'pending' && 'Pending'}
                    {fileData.status === 'uploading' && 'Uploading'}
                    {fileData.status === 'completed' && '✓ Completed'}
                    {fileData.status === 'failed' && '✗ Failed'}
                  </span>
                </div>

                {/* Progress bar */}
                {fileData.progress && (
                  <div className="space-y-2">
                    <div className="w-full bg-gray-200 rounded-full h-2">
                      <div
                        className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                        style={{ width: `${fileData.progress.percentage}%` }}
                      />
                    </div>

                    {/* Progress details */}
                    <div className="flex items-center justify-between text-xs text-gray-600">
                      <span>
                        {fileData.progress.percentage.toFixed(1)}% •{' '}
                        {formatFileSize(fileData.progress.transferredBytes)} of{' '}
                        {formatFileSize(fileData.progress.totalBytes)}
                      </span>
                      <span>
                        {formatSpeed(fileData.progress.speed)} •{' '}
                        {formatTime(fileData.progress.estimatedTimeRemaining)} remaining
                      </span>
                    </div>
                  </div>
                )}

                {/* Completion info */}
                {fileData.result && (
                  <div className="text-xs text-gray-600">
                    <p>Uploaded to: {fileData.result.savedPath}</p>
                    <p>
                      Average speed: {formatSpeed(fileData.result.averageSpeed)} •{' '}
                      Duration: {(fileData.result.duration / 1000).toFixed(1)}s
                    </p>
                  </div>
                )}

                {/* Error info */}
                {fileData.error && (
                  <div className="text-xs text-red-600">
                    <p className="font-medium">Error: {fileData.error.errorType}</p>
                    <p>{fileData.error.errorMessage}</p>
                  </div>
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
