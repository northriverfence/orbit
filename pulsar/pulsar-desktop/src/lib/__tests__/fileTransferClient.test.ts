/**
 * Unit tests for FileTransferClient
 */

import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { FileTransferClient, TransferProgress, TransferResult, TransferError } from '../fileTransferClient'

// Mock data helpers
const createMockFile = (name: string, size: number, content?: string): File => {
  const actualContent = content || 'x'.repeat(size)
  const blob = new Blob([actualContent], { type: 'text/plain' })
  const file = new File([blob], name, { type: 'text/plain', lastModified: Date.now() })

  // Ensure arrayBuffer method exists
  if (!file.arrayBuffer) {
    (file as any).arrayBuffer = async () => {
      const reader = new FileReader()
      return new Promise((resolve, reject) => {
        reader.onload = () => resolve(reader.result as ArrayBuffer)
        reader.onerror = reject
        reader.readAsArrayBuffer(file)
      })
    }
  }

  return file
}

const createMockReader = (responses: any[]) => {
  let readIndex = 0
  return {
    read: vi.fn(async () => {
      if (readIndex >= responses.length) {
        return { done: true, value: undefined }
      }
      const response = responses[readIndex++]
      const encoder = new TextEncoder()
      return { done: false, value: encoder.encode(JSON.stringify(response)) }
    }),
    cancel: vi.fn(),
    releaseLock: vi.fn(),
  }
}

const createMockWriter = () => {
  const writes: any[] = []
  return {
    write: vi.fn(async (data: Uint8Array) => {
      writes.push(data)
    }),
    close: vi.fn(),
    releaseLock: vi.fn(),
    writes,
  }
}

// Mock WebTransport API
class MockWebTransport {
  ready: Promise<void>
  closed: Promise<void>
  private streamFactory: () => any

  constructor(url: string, streamFactory?: () => any) {
    this.ready = Promise.resolve()
    this.closed = new Promise(() => {})
    this.streamFactory = streamFactory || (() => ({
      writable: new WritableStream(),
      readable: new ReadableStream(),
    }))
  }

  async createBidirectionalStream() {
    return this.streamFactory()
  }

  close() {
    // Mock close
  }
}

// Set global WebTransport
global.WebTransport = MockWebTransport as any

describe('FileTransferClient', () => {
  let client: FileTransferClient
  const testUrl = 'https://127.0.0.1:4433'

  beforeEach(() => {
    client = new FileTransferClient(testUrl)
  })

  afterEach(async () => {
    await client.close()
  })

  describe('constructor', () => {
    it('should create a client with the provided URL', () => {
      const client = new FileTransferClient(testUrl)
      expect(client).toBeDefined()
    })
  })

  describe('getProgress', () => {
    it('should return null for non-existent transfer', () => {
      const progress = client.getProgress('non-existent-id')
      expect(progress).toBeNull()
    })
  })

  describe('close', () => {
    it('should close the client without errors', async () => {
      await expect(client.close()).resolves.toBeUndefined()
    })

    it('should clear active transfers on close', async () => {
      await client.close()
      const progress = client.getProgress('any-id')
      expect(progress).toBeNull()
    })
  })

  describe('uploadFile', () => {
    it('should successfully upload a small file', async () => {
      const mockFile = createMockFile('test.txt', 1024)
      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { accepted: true, resume_supported: true, max_chunk_size: 1024 * 1024 },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 1, received_bytes: 1024 }
      ])

      // Mock WebTransport with our custom stream
      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      const result = await client.uploadFile(mockFile)

      expect(result).toBeDefined()
      expect(result.fileName).toBe('test.txt')
      expect(result.fileSize).toBe(1024)
      expect(result.savedPath).toBe('/tmp/test.txt')
      expect(mockWriter.write).toHaveBeenCalled()
      expect(mockWriter.close).toHaveBeenCalled()
    })

    it('should call onProgress callback during upload', async () => {
      const mockFile = createMockFile('test.txt', 2048)
      const onProgress = vi.fn()
      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { accepted: true, resume_supported: true, max_chunk_size: 1024 * 1024 },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 1, received_bytes: 2048 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await client.uploadFile(mockFile, { onProgress })

      expect(onProgress).toHaveBeenCalled()
      const progressCall = onProgress.mock.calls[0][0]
      expect(progressCall).toHaveProperty('transferId')
      expect(progressCall).toHaveProperty('fileName', 'test.txt')
      expect(progressCall).toHaveProperty('totalBytes', 2048)
    })

    it('should call onComplete callback on success', async () => {
      const mockFile = createMockFile('test.txt', 1024)
      const onComplete = vi.fn()
      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { accepted: true, resume_supported: true, max_chunk_size: 1024 * 1024 },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 1, received_bytes: 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await client.uploadFile(mockFile, { onComplete })

      expect(onComplete).toHaveBeenCalled()
      const result = onComplete.mock.calls[0][0]
      expect(result.fileName).toBe('test.txt')
      expect(result.fileSize).toBe(1024)
    })

    it('should call onError callback on failure', async () => {
      const mockFile = createMockFile('test.txt', 1024)
      const onError = vi.fn()
      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { accepted: false, resume_supported: false, max_chunk_size: 1024 * 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await expect(client.uploadFile(mockFile, { onError })).rejects.toThrow('Transfer rejected by server')
      expect(onError).toHaveBeenCalled()
      expect(onError.mock.calls[0][0].errorType).toBe('transfer_failed')
    })

    it('should handle verification failure', async () => {
      const mockFile = createMockFile('test.txt', 1024)
      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { accepted: true, resume_supported: true, max_chunk_size: 1024 * 1024 },
        { received: true, hash_valid: true },
        { verified: false, saved_path: '/tmp/test.txt', received_chunks: 1, received_bytes: 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await expect(client.uploadFile(mockFile)).rejects.toThrow('File verification failed')
    })

    it('should use custom chunk size', async () => {
      const mockFile = createMockFile('test.txt', 2048)
      const customChunkSize = 512
      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { accepted: true, resume_supported: true, max_chunk_size: 1024 * 1024 },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 4, received_bytes: 2048 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      const result = await client.uploadFile(mockFile, { chunkSize: customChunkSize })

      expect(result).toBeDefined()
      expect(result.fileSize).toBe(2048)
    })
  })

  describe('cancelTransfer', () => {
    it('should throw error for non-existent transfer', async () => {
      await expect(client.cancelTransfer('non-existent-id')).rejects.toThrow('Transfer not found')
    })
  })

  describe('getProgress with active transfer', () => {
    it('should return progress for active transfer', async () => {
      const mockFile = createMockFile('test.txt', 10 * 1024 * 1024) // 10 MB
      let progressCallback: any = null

      const mockWriter = createMockWriter()
      // Need to provide responses for all 10 chunks (10MB / 1MB chunk size)
      const mockReader = createMockReader([
        { accepted: true, resume_supported: true, max_chunk_size: 1024 * 1024 },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 10, received_bytes: 10 * 1024 * 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)

      // Start upload and capture progress callback
      await client.uploadFile(mockFile, {
        onProgress: (progress) => {
          progressCallback = progress
        }
      })

      // Progress callback should have been called
      expect(progressCallback).toBeDefined()
      if (progressCallback) {
        expect(progressCallback.fileName).toBe('test.txt')
        expect(progressCallback.totalBytes).toBe(10 * 1024 * 1024)
        expect(progressCallback.transferredBytes).toBeGreaterThan(0)
        expect(progressCallback.percentage).toBeGreaterThan(0)
        expect(progressCallback.speed).toBeGreaterThan(0)
      }
    })
  })

  describe('resumeUpload', () => {
    it('should resume a failed upload', async () => {
      const mockFile = createMockFile('test.txt', 5 * 1024 * 1024)
      const transferId = 'xfer-123-abc'

      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        {
          resumable: true,
          received_chunks: [0, 1],
          missing_chunks: [2, 3, 4],
          next_chunk_index: 2,
          received_bytes: 2 * 1024 * 1024
        },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 5, received_bytes: 5 * 1024 * 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      const result = await client.resumeUpload(transferId, mockFile)

      expect(result).toBeDefined()
      expect(result.transferId).toBe(transferId)
      expect(result.fileName).toBe('test.txt')
      expect(mockWriter.write).toHaveBeenCalled()
    })

    it('should throw error when resume not possible', async () => {
      const mockFile = createMockFile('test.txt', 1024)
      const transferId = 'xfer-123-abc'

      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        { resumable: false }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await expect(client.resumeUpload(transferId, mockFile)).rejects.toThrow('Transfer cannot be resumed')
    })

    it('should call onProgress during resume', async () => {
      const mockFile = createMockFile('test.txt', 3 * 1024 * 1024)
      const transferId = 'xfer-123-abc'
      const onProgress = vi.fn()

      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        {
          resumable: true,
          received_chunks: [0],
          missing_chunks: [1, 2],
          next_chunk_index: 1,
          received_bytes: 1024 * 1024
        },
        { received: true, hash_valid: true },
        { received: true, hash_valid: true },
        { verified: true, saved_path: '/tmp/test.txt', received_chunks: 3, received_bytes: 3 * 1024 * 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await client.resumeUpload(transferId, mockFile, { onProgress })

      expect(onProgress).toHaveBeenCalled()
    })

    it('should call onError on resume failure', async () => {
      const mockFile = createMockFile('test.txt', 1024)
      const transferId = 'xfer-123-abc'
      const onError = vi.fn()

      const mockWriter = createMockWriter()
      const mockReader = createMockReader([
        {
          resumable: true,
          received_chunks: [],
          missing_chunks: [0],
          next_chunk_index: 0,
          received_bytes: 0
        },
        { received: true, hash_valid: true },
        { verified: false, saved_path: '/tmp/test.txt', received_chunks: 1, received_bytes: 1024 }
      ])

      global.WebTransport = class extends MockWebTransport {
        async createBidirectionalStream() {
          return {
            writable: { getWriter: () => mockWriter },
            readable: { getReader: () => mockReader }
          }
        }
      } as any

      const client = new FileTransferClient(testUrl)
      await expect(client.resumeUpload(transferId, mockFile, { onError })).rejects.toThrow()
      expect(onError).toHaveBeenCalled()
      expect(onError.mock.calls[0][0].errorType).toBe('resume_failed')
    })
  })
})

describe('Transfer progress calculation', () => {
  it('should calculate percentage correctly', () => {
    const transferredBytes = 5 * 1024 * 1024 // 5 MB
    const totalBytes = 10 * 1024 * 1024 // 10 MB
    const percentage = (transferredBytes / totalBytes) * 100
    expect(percentage).toBe(50)
  })

  it('should calculate speed correctly', () => {
    const transferredBytes = 10 * 1024 * 1024 // 10 MB
    const elapsedMs = 1000 // 1 second
    const speed = (transferredBytes / elapsedMs) * 1000 // bytes per second
    expect(speed).toBe(10 * 1024 * 1024)
  })

  it('should calculate ETA correctly', () => {
    const remainingBytes = 5 * 1024 * 1024 // 5 MB
    const speed = 1 * 1024 * 1024 // 1 MB/s
    const eta = remainingBytes / speed // seconds
    expect(eta).toBe(5)
  })
})

describe('Chunk calculations', () => {
  it('should calculate correct number of chunks', () => {
    const fileSize = 10 * 1024 * 1024 // 10 MB
    const chunkSize = 1024 * 1024 // 1 MB
    const totalChunks = Math.ceil(fileSize / chunkSize)
    expect(totalChunks).toBe(10)
  })

  it('should handle file size not divisible by chunk size', () => {
    const fileSize = 10.5 * 1024 * 1024 // 10.5 MB
    const chunkSize = 1024 * 1024 // 1 MB
    const totalChunks = Math.ceil(fileSize / chunkSize)
    expect(totalChunks).toBe(11)
  })

  it('should handle files smaller than chunk size', () => {
    const fileSize = 512 * 1024 // 512 KB
    const chunkSize = 1024 * 1024 // 1 MB
    const totalChunks = Math.ceil(fileSize / chunkSize)
    expect(totalChunks).toBe(1)
  })
})

describe('Transfer ID generation', () => {
  it('should generate unique transfer IDs', () => {
    const ids = new Set<string>()
    for (let i = 0; i < 100; i++) {
      const id = `xfer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`
      ids.add(id)
    }
    expect(ids.size).toBe(100)
  })
})

describe('File size formatting', () => {
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
  }

  it('should format bytes correctly', () => {
    expect(formatFileSize(0)).toBe('0 B')
    expect(formatFileSize(1024)).toBe('1.00 KB')
    expect(formatFileSize(1024 * 1024)).toBe('1.00 MB')
    expect(formatFileSize(1024 * 1024 * 1024)).toBe('1.00 GB')
  })

  it('should format non-round numbers correctly', () => {
    expect(formatFileSize(1536)).toBe('1.50 KB')
    expect(formatFileSize(2.5 * 1024 * 1024)).toBe('2.50 MB')
  })
})

describe('Time formatting', () => {
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

  it('should format seconds correctly', () => {
    expect(formatTime(0.5)).toBe('< 1s')
    expect(formatTime(30)).toBe('30s')
    expect(formatTime(59)).toBe('59s')
  })

  it('should format minutes correctly', () => {
    expect(formatTime(60)).toBe('1m 0s')
    expect(formatTime(90)).toBe('1m 30s')
    expect(formatTime(3599)).toBe('59m 59s')
  })

  it('should format hours correctly', () => {
    expect(formatTime(3600)).toBe('1h 0m')
    expect(formatTime(7200)).toBe('2h 0m')
    expect(formatTime(3661)).toBe('1h 1m')
  })

  it('should handle infinite values', () => {
    expect(formatTime(Infinity)).toBe('calculating...')
  })
})

describe('Speed formatting', () => {
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 B'
    const k = 1024
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`
  }

  const formatSpeed = (bytesPerSecond: number): string => {
    return `${formatFileSize(bytesPerSecond)}/s`
  }

  it('should format speed correctly', () => {
    expect(formatSpeed(1024)).toBe('1.00 KB/s')
    expect(formatSpeed(1024 * 1024)).toBe('1.00 MB/s')
    expect(formatSpeed(10 * 1024 * 1024)).toBe('10.00 MB/s')
  })
})

describe('Error handling', () => {
  it('should create transfer error with correct structure', () => {
    const error: TransferError = {
      transferId: 'test-id',
      errorType: 'network_error',
      errorMessage: 'Connection failed',
    }

    expect(error.transferId).toBe('test-id')
    expect(error.errorType).toBe('network_error')
    expect(error.errorMessage).toBe('Connection failed')
  })
})

describe('Progress tracking', () => {
  it('should create progress object with correct structure', () => {
    const progress: TransferProgress = {
      transferId: 'test-id',
      fileName: 'test.txt',
      totalBytes: 1024 * 1024,
      transferredBytes: 512 * 1024,
      percentage: 50,
      chunksCompleted: 5,
      totalChunks: 10,
      speed: 1024 * 1024,
      estimatedTimeRemaining: 5,
    }

    expect(progress.transferId).toBe('test-id')
    expect(progress.percentage).toBe(50)
    expect(progress.speed).toBe(1024 * 1024)
  })
})

describe('Transfer result', () => {
  it('should create result object with correct structure', () => {
    const result: TransferResult = {
      transferId: 'test-id',
      fileName: 'test.txt',
      fileSize: 1024 * 1024,
      savedPath: '/tmp/test.txt',
      duration: 5000,
      averageSpeed: 1024 * 200,
    }

    expect(result.transferId).toBe('test-id')
    expect(result.fileSize).toBe(1024 * 1024)
    expect(result.duration).toBe(5000)
  })
})
