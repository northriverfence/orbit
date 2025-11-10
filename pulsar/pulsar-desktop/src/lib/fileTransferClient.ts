/**
 * File Transfer Client
 *
 * Provides high-performance file uploads using WebTransport (HTTP/3 over QUIC)
 * with chunking, integrity validation, and resume capability.
 */

import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';

export interface TransferOptions {
  chunkSize?: number; // Default: 1 MB
  maxParallelChunks?: number; // Default: 4
  onProgress?: (progress: TransferProgress) => void;
  onComplete?: (result: TransferResult) => void;
  onError?: (error: TransferError) => void;
}

export interface TransferProgress {
  transferId: string;
  fileName: string;
  totalBytes: number;
  transferredBytes: number;
  percentage: number;
  chunksCompleted: number;
  totalChunks: number;
  speed: number; // bytes per second
  estimatedTimeRemaining: number; // seconds
}

export interface TransferResult {
  transferId: string;
  fileName: string;
  fileSize: number;
  savedPath: string;
  duration: number; // milliseconds
  averageSpeed: number; // bytes per second
}

export interface TransferError {
  transferId: string;
  errorType: string;
  errorMessage: string;
}

interface TransferStartMessage {
  type: 'transfer_start';
  transfer_id: string;
  timestamp: number;
  file_name: string;
  file_size: number;
  chunk_size: number;
  total_chunks: number;
  mime_type?: string;
  blake3_hash: string;
  metadata?: {
    modified_time?: string;
    permissions?: string;
  };
}

interface ChunkDataMessage {
  type: 'chunk_data';
  transfer_id: string;
  timestamp: number;
  chunk_index: number;
  chunk_size: number;
  chunk_hash: string;
}

interface TransferCompleteMessage {
  type: 'transfer_complete';
  transfer_id: string;
  timestamp: number;
  total_chunks: number;
  total_bytes: number;
  final_hash: string;
}

interface TransferAckMessage {
  transfer_id: string;
  timestamp: number;
  accepted: boolean;
  resume_supported: boolean;
  max_chunk_size: number;
}

interface ChunkAckMessage {
  transfer_id: string;
  timestamp: number;
  chunk_index: number;
  received: boolean;
  hash_valid: boolean;
}

interface TransferSuccessMessage {
  transfer_id: string;
  timestamp: number;
  verified: boolean;
  saved_path: string;
  received_chunks: number;
  received_bytes: number;
  computed_hash: string;
}

interface ResumeRequestMessage {
  type: 'resume_request';
  transfer_id: string;
  timestamp: number;
  file_name: string;
  file_size: number;
  original_hash: string;
}

interface ResumeInfoMessage {
  transfer_id: string;
  timestamp: number;
  resumable: boolean;
  received_chunks: number[];
  missing_chunks: number[];
  next_chunk_index: number;
  received_bytes: number;
}

/**
 * File Transfer Client using WebTransport
 */
export class FileTransferClient {
  private webTransportUrl: string;
  private transport: WebTransport | null = null;
  private activeTransfers: Map<string, ActiveTransfer> = new Map();

  constructor(url: string) {
    this.webTransportUrl = url;
  }

  /**
   * Connect to the WebTransport server
   */
  private async connect(): Promise<WebTransport> {
    if (this.transport) {
      try {
        await this.transport.ready;
        return this.transport;
      } catch {
        // Transport failed, create new one
      }
    }

    this.transport = new WebTransport(this.webTransportUrl);
    await this.transport.ready;
    console.log('WebTransport connected');
    return this.transport;
  }

  /**
   * Upload a file
   */
  async uploadFile(file: File, options: TransferOptions = {}): Promise<TransferResult> {
    const chunkSize = options.chunkSize || 1024 * 1024; // 1 MB default

    // Generate transfer ID
    const transferId = `xfer-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;

    // Calculate file hash
    console.log(`Computing BLAKE3 hash for ${file.name}...`);
    const fileData = await file.arrayBuffer();
    const fileHash = bytesToHex(blake3(new Uint8Array(fileData)));

    // Calculate chunks
    const totalChunks = Math.ceil(file.size / chunkSize);

    // Create active transfer
    const activeTransfer: ActiveTransfer = {
      transferId,
      fileName: file.name,
      fileSize: file.size,
      chunkSize,
      totalChunks,
      completedChunks: new Set(),
      startTime: Date.now(),
      transferredBytes: 0,
      onProgress: options.onProgress,
    };

    this.activeTransfers.set(transferId, activeTransfer);

    // Connect to WebTransport
    const transport = await this.connect();

    // Open bidirectional stream
    const stream = await transport.createBidirectionalStream();
    const writer = stream.writable.getWriter();
    const reader = stream.readable.getReader();

    try {
      // Send transfer start message
      const startMessage: TransferStartMessage = {
        type: 'transfer_start',
        transfer_id: transferId,
        timestamp: Date.now(),
        file_name: file.name,
        file_size: file.size,
        chunk_size: chunkSize,
        total_chunks: totalChunks,
        mime_type: file.type,
        blake3_hash: fileHash,
        metadata: {
          modified_time: new Date(file.lastModified).toISOString(),
        },
      };

      const startJson = JSON.stringify(startMessage);
      await writer.write(new TextEncoder().encode(startJson));

      // Wait for acknowledgment
      const { value: ackData } = await reader.read();
      const ackMessage: TransferAckMessage = JSON.parse(new TextDecoder().decode(ackData));

      if (!ackMessage.accepted) {
        throw new Error('Transfer rejected by server');
      }

      console.log(`Transfer ${transferId} started for ${file.name}`);

      // Send chunks
      await this.sendChunks(
        writer,
        reader,
        transferId,
        fileData,
        chunkSize,
        totalChunks,
        activeTransfer
      );

      // Send transfer complete message
      const completeMessage: TransferCompleteMessage = {
        type: 'transfer_complete',
        transfer_id: transferId,
        timestamp: Date.now(),
        total_chunks: totalChunks,
        total_bytes: file.size,
        final_hash: fileHash,
      };

      const completeJson = JSON.stringify(completeMessage);
      await writer.write(new TextEncoder().encode(completeJson));

      // Wait for success message
      const { value: successData } = await reader.read();
      const successMessage: TransferSuccessMessage = JSON.parse(new TextDecoder().decode(successData));

      if (!successMessage.verified) {
        throw new Error('File verification failed');
      }

      // Calculate results
      const duration = Date.now() - activeTransfer.startTime;
      const averageSpeed = (file.size / duration) * 1000; // bytes per second

      const result: TransferResult = {
        transferId,
        fileName: file.name,
        fileSize: file.size,
        savedPath: successMessage.saved_path,
        duration,
        averageSpeed,
      };

      // Clean up
      this.activeTransfers.delete(transferId);
      await writer.close();
      await reader.cancel();

      console.log(`Transfer ${transferId} completed successfully`);

      if (options.onComplete) {
        options.onComplete(result);
      }

      return result;
    } catch (error) {
      console.error(`Transfer ${transferId} failed:`, error);

      const transferError: TransferError = {
        transferId,
        errorType: 'transfer_failed',
        errorMessage: error instanceof Error ? error.message : String(error),
      };

      if (options.onError) {
        options.onError(transferError);
      }

      this.activeTransfers.delete(transferId);
      throw error;
    }
  }

  /**
   * Send file chunks
   */
  private async sendChunks(
    writer: WritableStreamDefaultWriter<Uint8Array>,
    reader: ReadableStreamDefaultReader<Uint8Array>,
    transferId: string,
    fileData: ArrayBuffer,
    chunkSize: number,
    totalChunks: number,
    activeTransfer: ActiveTransfer
  ): Promise<void> {
    const maxParallel = 4;
    let chunkIndex = 0;

    while (chunkIndex < totalChunks) {
      // Send up to maxParallel chunks
      const promises: Promise<void>[] = [];

      for (let i = 0; i < maxParallel && chunkIndex < totalChunks; i++, chunkIndex++) {
        promises.push(this.sendChunk(writer, reader, transferId, fileData, chunkIndex, chunkSize, activeTransfer));
      }

      // Wait for all chunks to be acknowledged
      await Promise.all(promises);
    }
  }

  /**
   * Send a single chunk
   */
  private async sendChunk(
    writer: WritableStreamDefaultWriter<Uint8Array>,
    reader: ReadableStreamDefaultReader<Uint8Array>,
    transferId: string,
    fileData: ArrayBuffer,
    chunkIndex: number,
    chunkSize: number,
    activeTransfer: ActiveTransfer
  ): Promise<void> {
    const start = chunkIndex * chunkSize;
    const end = Math.min(start + chunkSize, fileData.byteLength);
    const chunkData = fileData.slice(start, end);
    const chunkArray = new Uint8Array(chunkData);

    // Compute chunk hash
    const chunkHash = bytesToHex(blake3(chunkArray));

    // Send chunk data message
    const chunkMessage: ChunkDataMessage = {
      type: 'chunk_data',
      transfer_id: transferId,
      timestamp: Date.now(),
      chunk_index: chunkIndex,
      chunk_size: chunkArray.length,
      chunk_hash: chunkHash,
    };

    const chunkJson = JSON.stringify(chunkMessage);
    await writer.write(new TextEncoder().encode(chunkJson));

    // Send chunk data
    await writer.write(chunkArray);

    // Wait for chunk acknowledgment
    const { value: ackData } = await reader.read();
    const ackMessage: ChunkAckMessage = JSON.parse(new TextDecoder().decode(ackData));

    if (!ackMessage.received || !ackMessage.hash_valid) {
      throw new Error(`Chunk ${chunkIndex} failed validation`);
    }

    // Update progress
    activeTransfer.completedChunks.add(chunkIndex);
    activeTransfer.transferredBytes += chunkArray.length;

    if (activeTransfer.onProgress) {
      const elapsed = Date.now() - activeTransfer.startTime;
      const speed = (activeTransfer.transferredBytes / elapsed) * 1000;
      const remaining = activeTransfer.fileSize - activeTransfer.transferredBytes;
      const estimatedTimeRemaining = remaining / speed;

      const progress: TransferProgress = {
        transferId: activeTransfer.transferId,
        fileName: activeTransfer.fileName,
        totalBytes: activeTransfer.fileSize,
        transferredBytes: activeTransfer.transferredBytes,
        percentage: (activeTransfer.transferredBytes / activeTransfer.fileSize) * 100,
        chunksCompleted: activeTransfer.completedChunks.size,
        totalChunks: activeTransfer.totalChunks,
        speed,
        estimatedTimeRemaining,
      };

      activeTransfer.onProgress(progress);
    }
  }

  /**
   * Send specific chunks (for resume)
   */
  private async sendSpecificChunks(
    writer: WritableStreamDefaultWriter<Uint8Array>,
    reader: ReadableStreamDefaultReader<Uint8Array>,
    transferId: string,
    fileData: ArrayBuffer,
    chunkSize: number,
    chunkIndices: number[],
    activeTransfer: ActiveTransfer
  ): Promise<void> {
    const maxParallel = 4;
    let i = 0;

    while (i < chunkIndices.length) {
      // Send up to maxParallel chunks
      const promises: Promise<void>[] = [];

      for (let j = 0; j < maxParallel && i < chunkIndices.length; j++, i++) {
        const chunkIndex = chunkIndices[i];
        promises.push(this.sendChunk(writer, reader, transferId, fileData, chunkIndex, chunkSize, activeTransfer));
      }

      // Wait for all chunks to be acknowledged
      await Promise.all(promises);
    }
  }

  /**
   * Resume a failed upload
   */
  async resumeUpload(transferId: string, file: File, options: TransferOptions = {}): Promise<TransferResult> {
    const chunkSize = options.chunkSize || 1024 * 1024; // 1 MB default

    // Calculate file hash
    console.log(`Computing BLAKE3 hash for ${file.name}...`);
    const fileData = await file.arrayBuffer();
    const fileHash = bytesToHex(blake3(new Uint8Array(fileData)));

    // Connect to WebTransport
    const transport = await this.connect();

    // Open bidirectional stream
    const stream = await transport.createBidirectionalStream();
    const writer = stream.writable.getWriter();
    const reader = stream.readable.getReader();

    try {
      // Send resume request
      const resumeRequest: ResumeRequestMessage = {
        type: 'resume_request',
        transfer_id: transferId,
        timestamp: Date.now(),
        file_name: file.name,
        file_size: file.size,
        original_hash: fileHash,
      };

      const resumeJson = JSON.stringify(resumeRequest);
      await writer.write(new TextEncoder().encode(resumeJson));

      // Wait for resume info
      const { value: resumeData } = await reader.read();
      const resumeInfo: ResumeInfoMessage = JSON.parse(new TextDecoder().decode(resumeData));

      if (!resumeInfo.resumable) {
        throw new Error('Transfer cannot be resumed');
      }

      console.log(`Resuming transfer ${transferId} from chunk ${resumeInfo.next_chunk_index}`);
      console.log(`Already received: ${resumeInfo.received_chunks.length} chunks`);
      console.log(`Missing: ${resumeInfo.missing_chunks.length} chunks`);

      // Calculate total chunks
      const totalChunks = Math.ceil(file.size / chunkSize);

      // Create active transfer
      const activeTransfer: ActiveTransfer = {
        transferId,
        fileName: file.name,
        fileSize: file.size,
        chunkSize,
        totalChunks,
        completedChunks: new Set(resumeInfo.received_chunks),
        startTime: Date.now(),
        transferredBytes: resumeInfo.received_bytes,
        onProgress: options.onProgress,
      };

      this.activeTransfers.set(transferId, activeTransfer);

      // Send only missing chunks
      await this.sendSpecificChunks(
        writer,
        reader,
        transferId,
        fileData,
        chunkSize,
        resumeInfo.missing_chunks,
        activeTransfer
      );

      // Send transfer complete message
      const completeMessage: TransferCompleteMessage = {
        type: 'transfer_complete',
        transfer_id: transferId,
        timestamp: Date.now(),
        total_chunks: totalChunks,
        total_bytes: file.size,
        final_hash: fileHash,
      };

      const completeJson = JSON.stringify(completeMessage);
      await writer.write(new TextEncoder().encode(completeJson));

      // Wait for success message
      const { value: successData } = await reader.read();
      const successMessage: TransferSuccessMessage = JSON.parse(new TextDecoder().decode(successData));

      if (!successMessage.verified) {
        throw new Error('File verification failed');
      }

      // Calculate results
      const duration = Date.now() - activeTransfer.startTime;
      const averageSpeed = ((file.size - resumeInfo.received_bytes) / duration) * 1000;

      const result: TransferResult = {
        transferId,
        fileName: file.name,
        fileSize: file.size,
        savedPath: successMessage.saved_path,
        duration,
        averageSpeed,
      };

      // Clean up
      this.activeTransfers.delete(transferId);
      await writer.close();
      await reader.cancel();

      console.log(`Transfer ${transferId} resumed and completed successfully`);

      if (options.onComplete) {
        options.onComplete(result);
      }

      return result;
    } catch (error) {
      console.error(`Resume transfer ${transferId} failed:`, error);

      const transferError: TransferError = {
        transferId,
        errorType: 'resume_failed',
        errorMessage: error instanceof Error ? error.message : String(error),
      };

      if (options.onError) {
        options.onError(transferError);
      }

      this.activeTransfers.delete(transferId);
      throw error;
    }
  }

  /**
   * Cancel a transfer
   */
  async cancelTransfer(transferId: string): Promise<void> {
    const activeTransfer = this.activeTransfers.get(transferId);
    if (!activeTransfer) {
      throw new Error('Transfer not found');
    }

    // TODO: Send abort message
    this.activeTransfers.delete(transferId);
    console.log(`Transfer ${transferId} cancelled`);
  }

  /**
   * Get transfer progress
   */
  getProgress(transferId: string): TransferProgress | null {
    const activeTransfer = this.activeTransfers.get(transferId);
    if (!activeTransfer) {
      return null;
    }

    const elapsed = Date.now() - activeTransfer.startTime;
    const speed = (activeTransfer.transferredBytes / elapsed) * 1000;
    const remaining = activeTransfer.fileSize - activeTransfer.transferredBytes;
    const estimatedTimeRemaining = remaining / speed;

    return {
      transferId: activeTransfer.transferId,
      fileName: activeTransfer.fileName,
      totalBytes: activeTransfer.fileSize,
      transferredBytes: activeTransfer.transferredBytes,
      percentage: (activeTransfer.transferredBytes / activeTransfer.fileSize) * 100,
      chunksCompleted: activeTransfer.completedChunks.size,
      totalChunks: activeTransfer.totalChunks,
      speed,
      estimatedTimeRemaining,
    };
  }

  /**
   * Close the client and clean up resources
   */
  async close(): Promise<void> {
    if (this.transport) {
      this.transport.close();
      this.transport = null;
    }

    this.activeTransfers.clear();
    console.log('FileTransferClient closed');
  }
}

interface ActiveTransfer {
  transferId: string;
  fileName: string;
  fileSize: number;
  chunkSize: number;
  totalChunks: number;
  completedChunks: Set<number>;
  startTime: number;
  transferredBytes: number;
  onProgress?: (progress: TransferProgress) => void;
}
