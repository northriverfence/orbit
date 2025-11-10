import React, { useState, useRef } from 'react'
import { FileTransferClient, TransferProgress, TransferResult, TransferError } from '../lib/fileTransferClient'

interface ResumeDialogProps {
  transferId: string
  fileName: string
  fileSize: number
  webtransportUrl?: string
  onResumStart?: () => void
  onResumeProgress?: (progress: TransferProgress) => void
  onResumeComplete?: (result: TransferResult) => void
  onResumeError?: (error: TransferError) => void
  onClose: () => void
}

export default function ResumeDialog({
  transferId,
  fileName,
  fileSize,
  webtransportUrl = 'https://127.0.0.1:4433',
  onResumStart,
  onResumeProgress,
  onResumeComplete,
  onResumeError,
  onClose,
}: ResumeDialogProps) {
  const [isResuming, setIsResuming] = useState(false)
  const [progress, setProgress] = useState<TransferProgress | null>(null)
  const [error, setError] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const clientRef = useRef<FileTransferClient | null>(null)

  // Initialize client
  if (!clientRef.current) {
    clientRef.current = new FileTransferClient(webtransportUrl)
  }

  // Format file size
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
  }

  // Handle file selection
  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    // Validate file name matches
    if (file.name !== fileName) {
      setError(`File name mismatch. Expected: ${fileName}, Got: ${file.name}`)
      return
    }

    // Validate file size matches
    if (file.size !== fileSize) {
      setError(`File size mismatch. Expected: ${formatFileSize(fileSize)}, Got: ${formatFileSize(file.size)}`)
      return
    }

    setError(null)
    setIsResuming(true)

    if (onResumStart) {
      onResumStart()
    }

    try {
      const result = await clientRef.current!.resumeUpload(transferId, file, {
        onProgress: (prog) => {
          setProgress(prog)
          if (onResumeProgress) {
            onResumeProgress(prog)
          }
        },
      })

      if (onResumeComplete) {
        onResumeComplete(result)
      }

      // Close dialog on success
      setTimeout(() => {
        onClose()
      }, 1000)
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      setError(errorMessage)

      const transferError: TransferError = {
        transferId,
        errorType: 'resume_failed',
        errorMessage,
      }

      if (onResumeError) {
        onResumeError(transferError)
      }

      setIsResuming(false)
    }
  }

  // Open file picker
  const handleSelectFile = () => {
    fileInputRef.current?.click()
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full">
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <h3 className="text-xl font-bold text-gray-900">Resume Transfer</h3>
          {!isResuming && (
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 transition-colors"
            >
              <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          )}
        </div>

        {/* Body */}
        <div className="p-6">
          {!isResuming ? (
            <>
              {/* File info */}
              <div className="mb-6">
                <p className="text-sm text-gray-600 mb-4">
                  Select the original file to resume the upload. The file must match the original exactly.
                </p>
                <div className="bg-gray-50 border border-gray-200 rounded-lg p-4 space-y-2">
                  <div>
                    <span className="text-xs text-gray-500">File Name:</span>
                    <p className="text-sm font-medium text-gray-900">{fileName}</p>
                  </div>
                  <div>
                    <span className="text-xs text-gray-500">File Size:</span>
                    <p className="text-sm font-medium text-gray-900">{formatFileSize(fileSize)}</p>
                  </div>
                  <div>
                    <span className="text-xs text-gray-500">Transfer ID:</span>
                    <p className="text-xs font-mono text-gray-700">{transferId}</p>
                  </div>
                </div>
              </div>

              {/* Error message */}
              {error && (
                <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                  <p className="text-sm text-red-700 font-medium">Error</p>
                  <p className="text-sm text-red-600">{error}</p>
                </div>
              )}

              {/* File input */}
              <input
                ref={fileInputRef}
                type="file"
                onChange={handleFileSelect}
                className="hidden"
              />

              {/* Select file button */}
              <button
                onClick={handleSelectFile}
                className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors flex items-center justify-center space-x-2"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
                  />
                </svg>
                <span>Select File to Resume</span>
              </button>
            </>
          ) : (
            <>
              {/* Resuming state */}
              <div className="space-y-4">
                <div className="flex items-center justify-center">
                  <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>

                <p className="text-center text-sm text-gray-600">Resuming upload...</p>

                {/* Progress info */}
                {progress && (
                  <div className="space-y-3">
                    {/* Progress bar */}
                    <div className="w-full bg-gray-200 rounded-full h-3">
                      <div
                        className="bg-gradient-to-r from-blue-500 to-blue-600 h-3 rounded-full transition-all duration-300"
                        style={{ width: `${progress.percentage}%` }}
                      />
                    </div>

                    {/* Progress details */}
                    <div className="flex items-center justify-between text-xs text-gray-600">
                      <span>{progress.percentage.toFixed(1)}%</span>
                      <span>
                        {formatFileSize(progress.transferredBytes)} / {formatFileSize(progress.totalBytes)}
                      </span>
                    </div>

                    {/* Chunks and speed */}
                    <div className="flex items-center justify-between text-xs text-gray-600">
                      <span>
                        Chunk {progress.chunksCompleted} / {progress.totalChunks}
                      </span>
                      <span>{formatFileSize(progress.speed)}/s</span>
                    </div>
                  </div>
                )}

                {error && (
                  <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                    <p className="text-sm text-red-700 font-medium">Resume Failed</p>
                    <p className="text-sm text-red-600">{error}</p>
                  </div>
                )}
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        {!isResuming && (
          <div className="flex items-center justify-end space-x-3 p-6 border-t border-gray-200">
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 rounded-md transition-colors"
            >
              Cancel
            </button>
          </div>
        )}
      </div>
    </div>
  )
}
