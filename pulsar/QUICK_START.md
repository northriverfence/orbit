# Pulsar Terminal - Quick Start Guide

**Get up and running in 5 minutes!**

---

## 🚀 Quick Start

### 1. Start the Daemon

```bash
cd /opt/singulio-dev/tools/shell/fork/orbit/pulsar/pulsar-daemon
cargo run --release

# Daemon starts on: ~/.config/orbit/pulsar.sock
```

### 2. Test with Python Script

```bash
cd /opt/singulio-dev/tools/shell/fork/orbit/pulsar
python3 test-pty-io.py

# Should see: ✓ All tests passed!
```

### 3. Use in Desktop App

```bash
cd /opt/singulio-dev/tools/shell/fork/orbit/pulsar/pulsar-desktop

# Install frontend dependencies
npm install xterm xterm-addon-fit xterm-addon-web-links

# Run desktop app
npm run tauri dev
```

### 4. Add Terminal Component

```tsx
import PulsarTerminal from './components/PulsarTerminal';

function App() {
  return (
    <div style={{ width: '100vw', height: '100vh' }}>
      <PulsarTerminal />
    </div>
  );
}
```

---

## 📋 Available Commands (11 Total)

### Session Management
```typescript
// Create local session
const sessionId = await invoke('daemon_create_local_session', {
  name: 'My Terminal',
  cols: 80,
  rows: 24
});

// Create SSH session
const sessionId = await invoke('daemon_create_ssh_session', {
  name: 'Remote Server',
  host: '192.168.1.100',
  port: 22,
  cols: 80,
  rows: 24
});

// List all sessions
const sessions = await invoke('daemon_list_sessions', {});

// Attach to session
await invoke('daemon_attach_session', {
  sessionId: 'uuid-here'
});

// Detach from session
await invoke('daemon_detach_session', {
  sessionId: 'uuid-here',
  clientId: 'client-uuid'
});

// Terminate session
await invoke('daemon_terminate_session', {
  sessionId: 'uuid-here'
});
```

### Terminal I/O
```typescript
// Send input (base64-encoded)
const bytesWritten = await invoke('daemon_send_input', {
  sessionId: 'uuid-here',
  data: btoa('ls -la\n')
});

// Receive output (base64-encoded)
const result = await invoke('daemon_receive_output', {
  sessionId: 'uuid-here',
  timeout_ms: 100  // optional
});
const output = atob(result.data);
console.log(output);
```

### Terminal Control
```typescript
// Resize terminal
await invoke('daemon_resize_terminal', {
  sessionId: 'uuid-here',
  cols: 120,
  rows: 30
});
```

### Status & Health
```typescript
// Get daemon status
const status = await invoke('daemon_get_status', {});
// Returns: { version, uptime_seconds, num_sessions, num_clients }

// Check connection
const isConnected = await invoke('daemon_check_connection', {});
```

---

## 🧪 Manual Testing

### Test with netcat

```bash
# Connect to daemon
nc -U ~/.config/orbit/pulsar.sock

# Create session
{"id":"1","method":"create_session","params":{"name":"test","type":"Local"}}

# Send input (base64 of "ls\n")
{"id":"2","method":"send_input","params":{"session_id":"UUID-HERE","data":"bHMK"}}

# Receive output
{"id":"3","method":"receive_output","params":{"session_id":"UUID-HERE"}}

# Terminate session
{"id":"4","method":"terminate_session","params":{"session_id":"UUID-HERE"}}
```

---

## 📁 File Locations

### Daemon
- **Binary**: `pulsar-daemon/target/release/pulsar-daemon`
- **Socket**: `~/.config/orbit/pulsar.sock`
- **Config**: `~/.config/orbit/daemon.yaml` (optional)

### Desktop App
- **Component**: `pulsar-desktop/src/components/PulsarTerminal.tsx`
- **Example Page**: `pulsar-desktop/src/pages/TerminalPage.tsx`
- **Commands**: `pulsar-desktop/src-tauri/src/daemon_commands.rs`

### Documentation
- **Implementation**: `PTY_IO_IMPLEMENTATION_COMPLETE.md`
- **Component Guide**: `FRONTEND_TERMINAL_COMPONENT.md`
- **Phase Summary**: `PHASE4_COMPLETE_SUMMARY.md`
- **Quick Start**: `QUICK_START.md` (this file)

---

## 🐛 Troubleshooting

### Daemon won't start
```bash
# Check if socket exists
ls -la ~/.config/orbit/pulsar.sock

# If exists, remove and retry
rm ~/.config/orbit/pulsar.sock
cargo run --release
```

### Test script fails
```bash
# Ensure daemon is running
ps aux | grep pulsar-daemon

# Check socket permissions
ls -la ~/.config/orbit/pulsar.sock

# Should show: srwxr-xr-x
```

### Terminal not responding
```bash
# Check daemon logs
tail -f /tmp/pulsar-daemon.log

# List active sessions
python3 -c "
import socket, json
s = socket.socket(socket.AF_UNIX)
s.connect('$HOME/.config/orbit/pulsar.sock')
s.sendall(b'{\"id\":\"1\",\"method\":\"list_sessions\",\"params\":{}}\n')
print(s.recv(4096).decode())
"
```

### High CPU usage
```typescript
// Increase poll interval in component
<PulsarTerminal pollInterval={100} />  // 100ms instead of 50ms
```

---

## 📚 More Information

- **Full Implementation Guide**: `PTY_IO_IMPLEMENTATION_COMPLETE.md`
- **Component Documentation**: `FRONTEND_TERMINAL_COMPONENT.md`
- **Phase 4 Summary**: `PHASE4_COMPLETE_SUMMARY.md`

---

## ✅ Verification Checklist

- [ ] Daemon starts without errors
- [ ] Socket file created: `~/.config/orbit/pulsar.sock`
- [ ] Python test passes: `✓ All tests passed!`
- [ ] Can create session via IPC
- [ ] Can send input to PTY
- [ ] Can receive output from PTY
- [ ] Sessions terminate cleanly
- [ ] Desktop app compiles
- [ ] Terminal component renders
- [ ] Typing works in terminal
- [ ] Commands execute and show output

---

**Status**: Ready for production use! 🎉
