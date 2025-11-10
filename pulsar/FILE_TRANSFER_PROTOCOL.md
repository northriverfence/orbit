# Pulsar File Transfer Protocol Specification

## Overview

The Pulsar File Transfer Protocol provides high-performance, resumable file transfers using WebTransport (HTTP/3 over QUIC) as the underlying transport. This protocol supports chunked transfers, parallel streams, integrity validation, and resume capability.

**Version**: 1.0.0
**Transport**: WebTransport (HTTP/3 + QUIC)
**Status**: Design Specification

---

## Architecture

### Transport Layer (Already Complete)

- **WebTransport Server**: Port 4433 (UDP)
- **Protocol**: HTTP/3 over QUIC
- **TLS**: Self-signed certificate (development)
- **Advantages**:
  - Multiple parallel streams
  - Built-in congestion control
  - 0-RTT connection resumption
  - Native browser support

### Application Layer (To Be Implemented)

```
┌─────────────────────────────────────────────────────────────┐
│                    Pulsar Desktop Client                     │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Drag & Drop │→ │ File Chunker │→ │ Transfer Manager  │  │
│  │     UI      │  │   (1MB)      │  │   (Queue/Resume)  │  │
│  └─────────────┘  └──────────────┘  └───────────────────┘  │
│                           ↓                     ↓            │
│                    ┌─────────────────────────────┐           │
│                    │  WebTransport Client        │           │
│                    │  (Bidirectional Streams)    │           │
│                    └─────────────────────────────┘           │
└────────────────────────────┬────────────────────────────────┘
                             │ QUIC (UDP)
                             ↓
┌─────────────────────────────────────────────────────────────┐
│                    Pulsar Daemon (Rust)                      │
│  ┌─────────────────────────────────────────────────────────┐│
│  │           WebTransport Server (Quinn)                    ││
│  └─────────────────────────────────────────────────────────┘│
│                           ↓                                  │
│  ┌─────────────────────────────────────────────────────────┐│
│  │         File Transfer Protocol Handler                   ││
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐  ││
│  │  │ Chunk        │  │ Chunk        │  │ Integrity    │  ││
│  │  │ Receiver     │  │ Assembler    │  │ Validator    │  ││
│  │  │ (Streams)    │  │ (Disk Buffer)│  │ (BLAKE3)     │  ││
│  │  └──────────────┘  └──────────────┘  └──────────────┘  ││
│  └─────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────┘
```

---

## Protocol Messages

### Message Format

All messages are encoded as JSON with a binary payload section.

```typescript
interface TransferMessage {
  type: MessageType
  transferId: string
  timestamp: number
  payload: MessagePayload
}

enum MessageType {
  TRANSFER_START = 'transfer_start',
  CHUNK_DATA = 'chunk_data',
  CHUNK_ACK = 'chunk_ack',
  TRANSFER_COMPLETE = 'transfer_complete',
  TRANSFER_ABORT = 'transfer_abort',
  RESUME_REQUEST = 'resume_request',
  INTEGRITY_CHECK = 'integrity_check',
}
```

### 1. Transfer Start Message

Initiates a new file transfer session.

```json
{
  "type": "transfer_start",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897123456,
  "payload": {
    "fileName": "document.pdf",
    "fileSize": 10485760,
    "chunkSize": 1048576,
    "totalChunks": 10,
    "mimeType": "application/pdf",
    "blake3Hash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    "metadata": {
      "modifiedTime": "2025-11-06T13:00:00.000Z",
      "permissions": "0644"
    }
  }
}
```

**Server Response**:
```json
{
  "type": "transfer_ack",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897123457,
  "payload": {
    "accepted": true,
    "resumeSupported": true,
    "maxChunkSize": 1048576
  }
}
```

### 2. Chunk Data Message

Sends a single chunk of file data.

```json
{
  "type": "chunk_data",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897123500,
  "payload": {
    "chunkIndex": 0,
    "chunkSize": 1048576,
    "chunkHash": "abc123...",
    "data": "<binary data>"
  }
}
```

**Binary Format**:
```
┌──────────────────────────────────────────────────────────┐
│ JSON Header (variable length)                             │
│ {type, transferId, timestamp, chunkIndex, chunkSize}      │
├──────────────────────────────────────────────────────────┤
│ Chunk Data (chunkSize bytes)                              │
│ Raw binary file data                                      │
└──────────────────────────────────────────────────────────┘
```

**Server Response**:
```json
{
  "type": "chunk_ack",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897123501,
  "payload": {
    "chunkIndex": 0,
    "received": true,
    "hashValid": true
  }
}
```

### 3. Transfer Complete Message

Signals that all chunks have been sent.

```json
{
  "type": "transfer_complete",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897130000,
  "payload": {
    "totalChunks": 10,
    "totalBytes": 10485760,
    "finalHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

**Server Response**:
```json
{
  "type": "transfer_success",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897130001,
  "payload": {
    "verified": true,
    "savedPath": "/tmp/document.pdf",
    "receivedChunks": 10,
    "receivedBytes": 10485760,
    "computedHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

### 4. Resume Request Message

Requests resume information for a failed transfer.

```json
{
  "type": "resume_request",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897140000,
  "payload": {
    "fileName": "document.pdf",
    "fileSize": 10485760,
    "originalHash": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
  }
}
```

**Server Response**:
```json
{
  "type": "resume_info",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897140001,
  "payload": {
    "resumable": true,
    "receivedChunks": [0, 1, 2, 3, 4, 5],
    "missingChunks": [6, 7, 8, 9],
    "nextChunkIndex": 6,
    "receivedBytes": 6291456
  }
}
```

### 5. Transfer Abort Message

Cancels an in-progress transfer.

```json
{
  "type": "transfer_abort",
  "transferId": "xfer-1730897123456-abc123",
  "timestamp": 1730897145000,
  "payload": {
    "reason": "user_cancelled"
  }
}
```

---

## Transfer Flow

### Normal Transfer Flow

```
Client                                  Server
  │                                       │
  │─────── TRANSFER_START ──────────────→│
  │                                       │ (Create transfer session)
  │←────── TRANSFER_ACK ─────────────────│
  │                                       │
  │─────── CHUNK_DATA [0] ──────────────→│
  │←────── CHUNK_ACK [0] ────────────────│
  │                                       │
  │─────── CHUNK_DATA [1] ──────────────→│
  │←────── CHUNK_ACK [1] ────────────────│
  │                                       │
  │         ... (parallel streams) ...    │
  │                                       │
  │─────── CHUNK_DATA [9] ──────────────→│
  │←────── CHUNK_ACK [9] ────────────────│
  │                                       │
  │─────── TRANSFER_COMPLETE ───────────→│
  │                                       │ (Verify hash, save file)
  │←────── TRANSFER_SUCCESS ─────────────│
  │                                       │
```

### Resume Flow

```
Client                                  Server
  │                                       │
  │─────── RESUME_REQUEST ──────────────→│
  │                                       │ (Check partial file)
  │←────── RESUME_INFO ──────────────────│
  │        (missing: [6,7,8,9])           │
  │                                       │
  │─────── CHUNK_DATA [6] ──────────────→│
  │←────── CHUNK_ACK [6] ────────────────│
  │                                       │
  │─────── CHUNK_DATA [7] ──────────────→│
  │←────── CHUNK_ACK [7] ────────────────│
  │                                       │
  │─────── CHUNK_DATA [8] ──────────────→│
  │←────── CHUNK_ACK [8] ────────────────│
  │                                       │
  │─────── CHUNK_DATA [9] ──────────────→│
  │←────── CHUNK_ACK [9] ────────────────│
  │                                       │
  │─────── TRANSFER_COMPLETE ───────────→│
  │←────── TRANSFER_SUCCESS ─────────────│
  │                                       │
```

---

## Chunking Strategy

### Chunk Size

**Default**: 1 MB (1,048,576 bytes)

**Rationale**:
- Optimal for QUIC's congestion control
- Balance between memory usage and throughput
- Suitable for network speeds 1 Mbps - 1 Gbps

**Configurable Range**: 256 KB - 4 MB

### Parallel Streams

**Max Parallel Chunks**: 4 streams

**Strategy**:
- Send 4 chunks simultaneously
- Wait for at least 1 ACK before sending next chunk
- Sliding window approach

**Benefits**:
- Utilizes QUIC's parallel stream capability
- Reduces impact of single chunk retransmission
- Maintains high throughput

---

## Integrity Validation

### BLAKE3 Hashing

**Algorithm**: BLAKE3
**Features**:
- Faster than SHA-256 (10x on modern hardware)
- Cryptographically secure
- Tree-based (supports parallel hashing)
- 256-bit output

**Implementation**:
```rust
use blake3::Hasher;

// Full file hash
let mut hasher = Hasher::new();
hasher.update(&file_data);
let hash = hasher.finalize();

// Per-chunk hash
let chunk_hash = blake3::hash(&chunk_data);
```

**Validation Points**:
1. **Per-Chunk**: Validate each chunk after receipt
2. **Full File**: Validate complete file after assembly
3. **Resume**: Validate existing chunks before resume

---

## Resume Capability

### State Persistence

**Server-Side Storage**:
```
/tmp/pulsar/transfers/{transfer_id}/
├── metadata.json          (transfer info)
├── chunks.db              (received chunks bitmap)
├── chunks/
│   ├── chunk-000.bin      (received chunks)
│   ├── chunk-001.bin
│   └── ...
└── final/{filename}       (assembled file)
```

**Metadata Schema**:
```json
{
  "transferId": "xfer-1730897123456-abc123",
  "fileName": "document.pdf",
  "fileSize": 10485760,
  "chunkSize": 1048576,
  "totalChunks": 10,
  "receivedChunks": [0, 1, 2, 3, 4, 5],
  "blake3Hash": "e3b0c44...",
  "startedAt": "2025-11-06T13:00:00.000Z",
  "lastActivity": "2025-11-06T13:01:30.000Z",
  "status": "incomplete"
}
```

### Resume Logic

```rust
pub async fn handle_resume_request(
    transfer_id: &str,
    file_name: &str,
    file_size: u64,
    expected_hash: &str,
) -> Result<ResumeInfo> {
    // Load transfer metadata
    let metadata = load_transfer_metadata(transfer_id)?;

    // Validate file matches
    if metadata.file_name != file_name || metadata.file_size != file_size {
        return Err(Error::FileMismatch);
    }

    // Check which chunks exist
    let received_chunks = scan_received_chunks(transfer_id)?;
    let missing_chunks = find_missing_chunks(&received_chunks, metadata.total_chunks);

    Ok(ResumeInfo {
        resumable: !missing_chunks.is_empty(),
        received_chunks,
        missing_chunks,
        next_chunk_index: missing_chunks[0],
        received_bytes: calculate_received_bytes(&received_chunks, metadata.chunk_size),
    })
}
```

---

## Error Handling

### Error Types

```rust
pub enum TransferError {
    NetworkError(String),
    ChunkHashMismatch { chunk_index: u32, expected: String, actual: String },
    FileHashMismatch { expected: String, actual: String },
    DiskFull,
    PermissionDenied,
    TransferTimeout,
    ChunkOutOfOrder { expected: u32, received: u32 },
    InvalidChunkSize { chunk_index: u32, expected: usize, actual: usize },
    TransferNotFound(String),
    ResumeNotSupported,
}
```

### Retry Strategy

**Chunk-Level Retries**:
- Max retries per chunk: 3
- Exponential backoff: 100ms, 200ms, 400ms
- After 3 failures: Abort transfer

**Transfer-Level Recovery**:
- Network failure → Resume transfer
- Server restart → Resume transfer
- User cancellation → Clean up partial files

---

## Performance Characteristics

### Throughput

**Expected Performance** (1 Gbps network):
- Theoretical max: 125 MB/s
- With overhead: ~100 MB/s
- 1 GB file: ~10 seconds

**Parallel Streams**:
- 4 parallel chunks × 1 MB = 4 MB in flight
- RTT 50ms → ~80 MB/s sustained

### Memory Usage

**Per Transfer**:
- Chunk buffer: 1 MB × 4 streams = 4 MB
- Metadata: ~10 KB
- Hash state: ~32 bytes

**10 Concurrent Transfers**: ~40 MB memory

### Disk I/O

**Write Strategy**:
- Buffered writes (4 KB buffer)
- Async I/O (tokio::fs)
- Direct chunk-to-disk (no full file in memory)

---

## Security Considerations

### Authentication

**Current**: Self-signed TLS certificate (development)
**Production**: Use proper CA-signed certificates

### Authorization

**Future**: Integrate with Pulsar's authentication system
- Session-based file transfer permissions
- Directory whitelist/blacklist
- Max file size limits

### Encryption

**Transport**: TLS 1.3 via QUIC
**At Rest**: Optional (future feature)

---

## API Specification

### Rust Backend API

```rust
// File transfer handler
pub struct FileTransferHandler {
    transfers: Arc<RwLock<HashMap<String, TransferSession>>>,
    storage_path: PathBuf,
}

impl FileTransferHandler {
    // Start a new transfer
    pub async fn start_transfer(
        &self,
        stream: &mut RecvStream,
        message: TransferStartMessage,
    ) -> Result<TransferAckMessage>;

    // Receive a chunk
    pub async fn receive_chunk(
        &self,
        stream: &mut RecvStream,
        message: ChunkDataMessage,
    ) -> Result<ChunkAckMessage>;

    // Complete transfer
    pub async fn complete_transfer(
        &self,
        stream: &mut RecvStream,
        message: TransferCompleteMessage,
    ) -> Result<TransferSuccessMessage>;

    // Resume transfer
    pub async fn resume_transfer(
        &self,
        stream: &mut RecvStream,
        message: ResumeRequestMessage,
    ) -> Result<ResumeInfoMessage>;

    // Abort transfer
    pub async fn abort_transfer(
        &self,
        transfer_id: &str,
    ) -> Result<()>;
}
```

### TypeScript Frontend API

```typescript
// File transfer client
export class FileTransferClient {
  constructor(webtransportUrl: string);

  // Upload a file
  async uploadFile(
    file: File,
    options?: TransferOptions
  ): Promise<TransferResult>;

  // Resume a failed upload
  async resumeUpload(
    transferId: string,
    file: File
  ): Promise<TransferResult>;

  // Cancel a transfer
  async cancelTransfer(transferId: string): Promise<void>;

  // Get transfer progress
  getProgress(transferId: string): TransferProgress | null;

  // Events
  on(event: 'progress', callback: (progress: TransferProgress) => void): void;
  on(event: 'complete', callback: (result: TransferResult) => void): void;
  on(event: 'error', callback: (error: TransferError) => void): void;
}
```

---

## Implementation Roadmap

### Phase 1: Core Protocol (Week 1-2)

**Backend**:
- [ ] File transfer handler module
- [ ] Chunk receiving and buffering
- [ ] Chunk assembly and disk writes
- [ ] BLAKE3 hash validation

**Frontend**:
- [ ] File chunking utility
- [ ] WebTransport client wrapper
- [ ] Transfer state management

### Phase 2: UI Components (Week 2-3)

**Frontend**:
- [ ] Drag-and-drop zone component
- [ ] Transfer progress component
- [ ] Transfer queue component
- [ ] Transfer history component

### Phase 3: Resume & Polish (Week 3-4)

**Backend**:
- [ ] Transfer state persistence
- [ ] Resume logic
- [ ] Cleanup expired transfers

**Frontend**:
- [ ] Resume UI
- [ ] Error handling
- [ ] Notifications

---

## Testing Strategy

### Unit Tests

- Chunk splitting algorithm
- Hash calculation
- Chunk reassembly
- Resume state detection

### Integration Tests

- Full file transfer (small files)
- Large file transfer (>100 MB)
- Multiple concurrent transfers
- Resume after network failure
- Hash validation failures

### Performance Tests

- Throughput benchmarks
- Memory usage profiling
- Concurrent transfer limits
- Network condition simulation

---

## Summary

The Pulsar File Transfer Protocol provides:

✅ **Chunked Transfers**: 1 MB chunks for optimal QUIC performance
✅ **Parallel Streams**: 4 concurrent chunks for high throughput
✅ **Resume Capability**: Server-side state persistence
✅ **Integrity Validation**: BLAKE3 hashing per-chunk and full-file
✅ **Error Recovery**: Automatic retry with exponential backoff
✅ **WebTransport**: HTTP/3 + QUIC for modern performance

**Estimated Throughput**: 80-100 MB/s on 1 Gbps network
**Memory Per Transfer**: 4-5 MB
**Resume Support**: Full resumption after network failure

**Status**: Ready for implementation
**Next**: Implement core protocol (Phase 1)
