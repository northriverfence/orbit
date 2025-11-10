# Pulsar Multi-Session Architecture Design

**Date**: 2025-11-04
**Version**: 1.0
**Status**: Implementation in progress

---

## Overview

The Pulsar daemon manages multiple concurrent SSH/terminal sessions independently from client applications. This enables:

- **Session Persistence**: Sessions survive client disconnections/restarts
- **Multi-Client**: Multiple clients can attach to the same daemon
- **Background Operations**: File transfers, port forwards continue when client detaches
- **Resource Efficiency**: One daemon manages all sessions

---

## Architecture Components

### 1. Session Manager (Enhanced)

**Purpose**: Thread-safe management of active sessions

**Features**:
- `Arc<RwLock<HashMap<Uuid, SessionData>>>` for concurrent access
- Session lifecycle: create, list, attach, detach, terminate
- Automatic cleanup of dead sessions
- Session metadata tracking (created_at, last_active, state)

**API**:
```rust
impl SessionManager {
    async fn create_session(&self, config: SessionConfig) -> Result<Uuid>;
    async fn get_session(&self, id: Uuid) -> Result<Arc<SessionData>>;
    async fn list_sessions(&self) -> Vec<SessionInfo>;
    async fn terminate_session(&self, id: Uuid) -> Result<()>;
    async fn cleanup_dead_sessions(&self);
}
```

### 2. IPC Server

**Purpose**: Communication channel between daemon and clients

**Protocol**: Unix Domain Socket (JSON-RPC 2.0 style)

**Message Types**:
- `create_session` - Start new SSH/terminal session
- `attach_session` - Connect client to existing session
- `detach_session` - Disconnect client (session continues)
- `send_input` - Send input to session PTY
- `receive_output` - Get output from session (streaming)
- `resize_terminal` - Update PTY dimensions
- `list_sessions` - Get all active sessions
- `terminate_session` - Kill a session
- `get_status` - Daemon health check

**Transport**:
- Socket path: `~/.config/orbit/pulsar.sock`
- Async message handling (tokio)
- Per-client connection state
- Broadcast support for multi-client scenarios

### 3. Session Persistence

**Purpose**: Restore sessions after daemon restart

**Storage**: SQLite database (`~/.config/orbit/pulsar.db`)

**Schema**:
```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    session_type TEXT NOT NULL,  -- 'local', 'ssh', 'serial'
    config TEXT NOT NULL,         -- JSON serialized config
    created_at INTEGER NOT NULL,
    last_active INTEGER NOT NULL,
    state TEXT NOT NULL           -- 'running', 'detached', 'stopped'
);

CREATE TABLE session_state (
    session_id TEXT PRIMARY KEY,
    pty_state BLOB,               -- Terminal state snapshot
    scrollback BLOB,              -- Terminal history
    environment TEXT,             -- JSON env vars
    FOREIGN KEY(session_id) REFERENCES sessions(id) ON DELETE CASCADE
);
```

**Features**:
- Auto-save session metadata on state change
- Optional scrollback history persistence
- Restore sessions on daemon startup
- Pruning of old/dead sessions

### 4. Session Data Structure

**Enhanced SessionData**:
```rust
pub struct SessionData {
    pub id: Uuid,
    pub name: String,
    pub session_type: SessionType,
    pub config: SessionConfig,
    pub pty: Arc<Mutex<PtyHandle>>,
    pub created_at: DateTime<Utc>,
    pub last_active: Arc<Mutex<DateTime<Utc>>>,
    pub state: Arc<RwLock<SessionState>>,
    pub clients: Arc<RwLock<HashSet<ClientId>>>,  // Attached clients
    pub output_broadcast: broadcast::Sender<Vec<u8>>,  // PTY output
}

pub enum SessionState {
    Running,
    Detached,
    Stopped,
}

pub enum SessionType {
    Local,
    Ssh { host: String, port: u16 },
    Serial { device: String },
}
```

### 5. Daemon Main Loop

**Responsibilities**:
- Initialize components (config, session manager, IPC server, database)
- Restore persisted sessions
- Start IPC server
- Handle shutdown signals (SIGTERM, SIGINT)
- Periodic cleanup tasks (every 60s)
- Save session state on shutdown

**Lifecycle**:
```
┌─────────────────┐
│ Load Config     │
└────────┬────────┘
         │
┌────────▼────────┐
│ Init Database   │
└────────┬────────┘
         │
┌────────▼────────┐
│ Restore Sessions│
└────────┬────────┘
         │
┌────────▼────────┐
│ Start IPC Server│
└────────┬────────┘
         │
┌────────▼────────┐
│   Event Loop    │◄──── IPC Requests
│  - Handle IPC   │
│  - Cleanup Task │
│  - Signal Watch │
└────────┬────────┘
         │ SIGTERM/SIGINT
┌────────▼────────┐
│ Graceful Stop   │
│ - Save Sessions │
│ - Close Sockets │
│ - Flush DB      │
└─────────────────┘
```

---

## Implementation Phases

### Phase 1: Enhanced Session Manager ✅ (This PR)
- Thread-safe session storage
- Session lifecycle management
- Basic session metadata

### Phase 2: IPC Server (This PR)
- Unix socket server
- JSON-RPC message protocol
- Request routing

### Phase 3: Session Persistence (This PR)
- SQLite schema
- Save/restore sessions
- State snapshots

### Phase 4: Daemon Main Loop (This PR)
- Component initialization
- Graceful shutdown
- Periodic cleanup

### Phase 5: Testing
- Unit tests for SessionManager
- Integration tests for IPC
- End-to-end daemon tests

---

## Security Considerations

1. **Socket Permissions**: Unix socket with 0700 permissions (owner-only)
2. **Session Isolation**: Each session runs in separate PTY
3. **Authentication**: Socket access implies trust (same user)
4. **Credential Storage**: Sensitive data encrypted at rest (future)
5. **Audit Logging**: Session create/terminate events logged

---

## Performance Targets

- **Session Creation**: < 100ms
- **IPC Latency**: < 5ms p99
- **Memory per Session**: < 10MB
- **Max Concurrent Sessions**: 100+
- **Database Operations**: < 10ms p99

---

## Compatibility

- **Orbit Integration**: Optional - can run standalone or with Orbit daemon
- **Desktop Client**: Backward compatible with existing Tauri commands
- **CLI Client**: Future CLI can use same IPC protocol

---

## Future Enhancements

1. **Session Sharing**: Multiple clients attach to same session (read-only or collaborative)
2. **Session Recording**: Record session history for playback
3. **Resource Limits**: Memory/CPU limits per session
4. **Session Groups**: Organize sessions into workspaces
5. **Hot Reload**: Daemon restart without killing sessions (using SCM_RIGHTS)

---

**Status**: Phase 1-4 implementation in progress
**Next**: Complete implementation and testing
