# Pulsar - Testing Guide

**Version**: v0.1.0
**Date**: 2025-11-01
**Status**: Ready for Testing

---

## 🎯 Prerequisites

### System Requirements

**Hardware**:
- x86_64 or ARM64 processor
- 4 GB RAM minimum (8 GB recommended)
- 500 MB disk space
- Display (GUI required for desktop app)

**Software**:
- **Rust**: 1.70+ (`rustup` recommended)
- **Node.js**: 18+ with npm
- **Tauri CLI**: 2.9.0 (`cargo install tauri-cli --version 2.9.0`)
- **SSH Server**: For testing connections (OpenSSH recommended)

**Platforms**:
- ✅ Linux (tested on Ubuntu 22.04)
- ✅ macOS 12+
- ✅ Windows 10/11

---

## 🚀 Building and Running

### Method 1: Development Mode

**Best for**: Active development, debugging, hot reload

```bash
cd /opt/singulio-dev/tools/shell/fork/orbit/pulsar/pulsar-desktop

# Install frontend dependencies (first time only)
npm install

# Run in dev mode
cargo tauri dev
```

**Expected Output**:
```
  VITE v6.4.1  ready in 190 ms
  ➜  Local:   http://localhost:5173/

Starting Pulsar Desktop v0.1.0
Application window opened
```

**Features**:
- ✅ Hot reload for frontend changes
- ✅ Auto-restart on Rust changes
- ✅ DevTools enabled (F12)
- ✅ Detailed logging in console

---

### Method 2: Production Build

**Best for**: Release testing, performance validation

```bash
cd /opt/singulio-dev/tools/shell/fork/orbit/pulsar/pulsar-desktop

# Build production binary
cargo tauri build

# Binary location:
# Linux: target/release/pulsar-desktop
# macOS: target/release/bundle/macos/Pulsar.app
# Windows: target/release/pulsar-desktop.exe
```

**Build Time**: ~5 minutes (first build), ~1 minute (incremental)

**Bundle Sizes**:
- Linux: ~15 MB (AppImage)
- macOS: ~20 MB (.app bundle)
- Windows: ~12 MB (.exe)

---

## 🧪 Test Scenarios

### Test 1: Application Launch

**Objective**: Verify app starts without errors

**Steps**:
1. Run `cargo tauri dev`
2. Wait for window to appear
3. Check console for error messages

**Expected Result**:
```
✓ Pulsar window opens
✓ "Welcome to Pulsar" screen visible
✓ "New SSH Connection" button displayed
✓ Sidebar with server list (empty)
✓ No JavaScript errors in console
```

**Logs**:
```
INFO: Starting Pulsar Desktop v0.1.0
INFO: SSH manager initialized
INFO: Application ready
```

---

### Test 2: Connection Dialog

**Objective**: Validate connection form and validation

**Steps**:
1. Click "New SSH Connection" button
2. Try submitting empty form
3. Fill in valid details:
   - Host: localhost
   - Port: 22
   - Username: (your username)
   - Auth: Password
   - Password: (your password)
4. Click "Connect"

**Expected Result**:

**Empty Form**:
```
✗ "Host is required" error shown
✗ "Username is required" error shown
✗ Connect button disabled until valid
```

**Valid Form**:
```
✓ Dialog closes
✓ Connection attempt begins
✓ Terminal displays connection status
```

**Dialog Features**:
- ✅ Auto-focus on host field
- ✅ Tab navigation between fields
- ✅ Escape closes dialog
- ✅ Ctrl+Enter connects
- ✅ Security notice displayed

---

### Test 3: SSH Connection (Password Auth)

**Objective**: Test password authentication flow

**Prerequisites**: SSH server running on localhost (port 22)

**Steps**:
1. Open connection dialog
2. Enter details:
   ```
   Host: localhost
   Port: 22
   Username: your_username
   Auth Type: Password
   Password: your_password
   ```
3. Click "Connect" or press Ctrl+Enter

**Expected Result**:

**Terminal Output**:
```
Connecting to your_username@localhost:22...
✓ Connected (Session: a1b2c3d4...)
🔑 Host Key: SHA256:uNiVztksCsDhcc0u9e8BujQXVUpKZIDTMczCvj3tD2s

Type to interact with the session

[your_username@localhost]$ _
```

**Logs**:
```
INFO: Connecting to your_username@localhost:22
INFO: Host key verified for localhost:22 (SHA256:...)
INFO: SSH authentication successful
INFO: Session a1b2c3d4-... created successfully
```

**Validation**:
- ✅ Connection succeeds
- ✅ Host key fingerprint displayed
- ✅ Terminal shows shell prompt
- ✅ Session ID shown in header

---

### Test 4: SSH Connection (Public Key Auth)

**Objective**: Test public key authentication

**Prerequisites**: SSH key pair (e.g., ~/.ssh/id_rsa)

**Steps**:
1. Open connection dialog
2. Enter details:
   ```
   Host: localhost
   Port: 22
   Username: your_username
   Auth Type: Public Key
   Key Path: ~/.ssh/id_rsa
   Passphrase: (if key is encrypted)
   ```
3. Connect

**Expected Result**:
```
✓ Authentication with key succeeds
✓ Same terminal output as password auth
✓ Host key fingerprint displayed
```

**Logs**:
```
INFO: Loading SSH key from ~/.ssh/id_rsa
INFO: SSH authentication successful (public key)
```

---

### Test 5: Terminal Interaction

**Objective**: Verify bidirectional I/O

**Steps**:
1. Connect to SSH server (Test 3 or 4)
2. Type commands in terminal:
   ```bash
   echo "Hello Pulsar"
   ls -la
   pwd
   whoami
   uname -a
   ```
3. Verify output appears correctly

**Expected Result**:
```
✓ All commands execute
✓ Output displayed in real-time
✓ Colors and formatting preserved
✓ No character loss or corruption
✓ Prompt reappears after each command
```

**Interactive Programs**:
```bash
# Test interactive editors
nano test.txt     # ✓ Should work
vim test.txt      # ✓ Should work
htop              # ✓ Should work
```

---

### Test 6: Terminal Resize

**Objective**: Verify PTY resize handling

**Steps**:
1. Connect to SSH server
2. Run: `tput cols; tput lines` (shows terminal dimensions)
3. Resize Pulsar window
4. Run again: `tput cols; tput lines`

**Expected Result**:
```
Before resize: 80x24
After resize:  120x40  (should match new window size)
```

**Validation**:
- ✅ Terminal content reflows correctly
- ✅ Applications (vim, htop) resize properly
- ✅ No display corruption

---

### Test 7: Host Key Verification (First Connection)

**Objective**: Validate unknown host handling

**Prerequisites**: Server not in ~/.ssh/known_hosts

**Steps**:
1. Remove server from known_hosts:
   ```bash
   ssh-keygen -R localhost
   ```
2. Connect via Pulsar

**Expected Result**:

**Development Mode** (current):
```
✓ Connection succeeds automatically
✓ Fingerprint displayed in terminal
✓ Host key added to ~/.ssh/known_hosts
```

**Logs**:
```
WARN: Unknown host key for localhost:22 (SHA256:...)
INFO: Auto-accepting unknown host key (development mode)
INFO: Added host key for localhost:22 to known_hosts
```

**Production Mode** (future):
```
✓ Dialog prompts user to verify fingerprint
✓ User can accept or reject
✓ Rejection prevents connection
```

---

### Test 8: Host Key Verification (Changed Key)

**Objective**: Detect MITM attacks via changed keys

**Prerequisites**: Regenerate server's SSH keys

**Steps**:
1. Connect to server (adds to known_hosts)
2. Regenerate server keys:
   ```bash
   # On server:
   sudo rm /etc/ssh/ssh_host_*
   sudo ssh-keygen -A
   sudo systemctl restart sshd
   ```
3. Reconnect via Pulsar

**Expected Result**:

**Connection Rejected**:
```
✗ Connection failed: Host key verification failed
```

**Terminal**:
```
Connecting to user@localhost:22...
✗ Connection failed: Host key verification failed
```

**Logs**:
```
ERROR: HOST KEY CHANGED for localhost:22! Possible MITM attack!
ERROR: Old key: ssh-ed25519 AAAA...
ERROR: New key: ssh-ed25519 BBBB...
ERROR: Fingerprint: SHA256:xyz...
ERROR: Rejecting changed host key
```

**Manual Fix**:
```bash
# Remove old key:
ssh-keygen -R localhost:22

# Reconnect (treated as unknown host)
```

---

### Test 9: Fingerprint Verification

**Objective**: Manually verify host key fingerprint

**Steps**:
1. Connect to localhost
2. Note fingerprint displayed:
   ```
   🔑 Host Key: SHA256:uNiVztksCsDhcc0u9e8BujQXVUpKZIDTMczCvj3tD2s
   ```
3. Compare with server fingerprint:
   ```bash
   ssh-keyscan -t ed25519 localhost | ssh-keygen -lf -
   ```

**Expected Result**:
```
✓ Fingerprints match exactly
✓ Format: "256 SHA256:... (ED25519)"
✓ Algorithm confirmed (SHA256, ED25519)
```

**Example**:
```bash
# Pulsar shows:
🔑 Host Key: SHA256:uNiVztksCsDhcc0u9e8BujQXVUpKZIDTMczCvj3tD2s

# ssh-keyscan shows:
256 SHA256:uNiVztksCsDhcc0u9e8BujQXVUpKZIDTMczCvj3tD2s localhost (ED25519)

✓ Match confirmed!
```

---

### Test 10: Multiple Concurrent Sessions

**Objective**: Test session isolation

**Steps**:
1. Connect to server A (localhost:22)
2. Open new connection (future: multiple tabs)
3. Connect to server B (different host or port)
4. Send commands to both sessions
5. Verify output doesn't cross

**Expected Result**:
```
✓ Each session independent
✓ Commands routed to correct session
✓ Output appears in correct terminal
✓ Disconnect affects only one session
```

---

### Test 11: Disconnect and Reconnect

**Objective**: Validate session cleanup

**Steps**:
1. Connect to server
2. Run some commands
3. Click "Disconnect" button
4. Reconnect to same server

**Expected Result**:

**After Disconnect**:
```
✓ Session removed from manager
✓ Terminal cleared
✓ Welcome screen reappears
✓ No resource leaks
```

**After Reconnect**:
```
✓ New session ID assigned
✓ New connection established
✓ Terminal shows fresh shell
✓ Previous session history lost (expected)
```

**Logs**:
```
INFO: Session a1b2c3d4-... disconnected
INFO: Connecting to user@localhost:22...
INFO: Session e5f6g7h8-... created successfully
```

---

### Test 12: Error Handling

**Objective**: Graceful handling of error conditions

**Test 12a: Connection Refused**
```bash
# Steps:
1. Stop SSH server: sudo systemctl stop sshd
2. Try to connect

# Expected:
✗ Connection failed: Connection refused (OS Error 111)
```

**Test 12b: Authentication Failed**
```bash
# Steps:
1. Enter wrong password
2. Click Connect

# Expected:
✗ Connection failed: Authentication failed
```

**Test 12c: Network Timeout**
```bash
# Steps:
1. Connect to unreachable host (192.0.2.1)
2. Wait for timeout

# Expected:
✗ Connection failed: Connection timed out
```

**Test 12d: Invalid Hostname**
```bash
# Steps:
1. Enter: "invalid.example.local"
2. Connect

# Expected:
✗ Connection failed: Name or service not known
```

---

## 📊 Verification Checklist

### UI/UX Validation

- [ ] Application launches without errors
- [ ] Connection dialog opens/closes smoothly
- [ ] Form validation shows appropriate errors
- [ ] Keyboard shortcuts work (Tab, Escape, Ctrl+Enter)
- [ ] Terminal displays correctly (fonts, colors)
- [ ] Window resize handled properly
- [ ] Disconnect button functional

### Security Validation

- [ ] Host key fingerprint displayed after connection
- [ ] Unknown hosts handled correctly
- [ ] Changed keys rejected (MITM protection)
- [ ] Fingerprint matches ssh-keyscan output
- [ ] Password not logged in plaintext
- [ ] known_hosts file created/updated correctly

### Functionality Validation

- [ ] Password authentication works
- [ ] Public key authentication works
- [ ] Encrypted keys (with passphrase) work
- [ ] Terminal I/O bidirectional and real-time
- [ ] Interactive programs (vim, htop) work
- [ ] Terminal resize propagates to server
- [ ] Multiple concurrent sessions isolated
- [ ] Disconnect/reconnect cycle clean

### Error Handling Validation

- [ ] Connection refused handled gracefully
- [ ] Authentication failures reported clearly
- [ ] Network timeouts don't crash app
- [ ] Invalid inputs validated
- [ ] Resource cleanup on errors

### Performance Validation

- [ ] Connection establishes quickly (< 2s)
- [ ] Terminal input lag < 50ms
- [ ] No memory leaks during long sessions
- [ ] CPU usage reasonable (< 5% idle)
- [ ] No UI freezing during connection

---

## 🐛 Known Issues

### Issue 1: Terminal Core Warnings

**Symptom**: Deprecation warnings during compilation
```
warning: use of deprecated function `GenericArray::from_slice`
```

**Impact**: None (cosmetic warning only)

**Status**: Future update to generic-array 1.x

---

### Issue 2: Headless Server Testing

**Symptom**: Can't run GUI on headless servers

**Workaround**:
```bash
# Use Xvfb for headless testing
Xvfb :99 -screen 0 1024x768x24 &
export DISPLAY=:99
cargo tauri dev
```

**Alternative**: Test on local machine with display

---

## 📈 Performance Benchmarks

### Connection Time

**Target**: < 2 seconds for typical connections

| Scenario | Expected Time | Notes |
|----------|---------------|-------|
| localhost (password) | < 500ms | Local network |
| LAN host (password) | < 1s | Same subnet |
| Remote host (password) | < 2s | Internet |
| Public key auth | +100ms | Key loading overhead |

### Terminal Latency

**Target**: < 50ms input-to-display

| Operation | Expected Latency |
|-----------|-----------------|
| Single character input | < 20ms |
| Command execution | < 100ms |
| Large output (1000 lines) | < 500ms |
| Window resize | < 200ms |

### Resource Usage

**Target**: Minimal resource consumption

| Metric | Idle | Active Session |
|--------|------|----------------|
| Memory | < 100 MB | < 150 MB |
| CPU | < 1% | < 5% |
| Disk I/O | Minimal | Log writes only |

---

## 🔍 Debugging Tips

### Enable Verbose Logging

```bash
RUST_LOG=debug cargo tauri dev
```

**Output**:
```
DEBUG: SSH read: 1024 bytes
DEBUG: SSH write: 42 bytes
DEBUG: Terminal resize: 120x40
```

### Check Known Hosts

```bash
cat ~/.ssh/known_hosts
```

**Expected Format**:
```
localhost ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIxyz...
example.com:2222 ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDabc...
```

### Monitor Network Traffic

```bash
# Watch SSH connections
sudo tcpdump -i any port 22 -A
```

### Inspect Session State

**In logs**:
```
INFO: Active sessions: 2
INFO: Session a1b2c3d4: user@localhost:22
INFO: Session e5f6g7h8: admin@example.com:22
```

---

## 🚀 Next Steps

After completing all tests:

1. **Document Results**: Create test report with results
2. **Report Issues**: File bugs for any failures
3. **Performance Tuning**: Optimize slow areas
4. **UI Enhancements**: Improve UX based on feedback
5. **Production Readiness**: Security audit, compliance

---

## 📝 Test Report Template

```markdown
# Pulsar Test Report

**Date**: YYYY-MM-DD
**Tester**: Your Name
**Version**: v0.1.0
**Platform**: Linux/macOS/Windows

## Summary

- Tests Run: X
- Passed: Y
- Failed: Z
- Success Rate: Y/X%

## Test Results

### ✅ Passed

- Test 1: Application Launch
- Test 3: SSH Connection (Password)
- Test 5: Terminal Interaction
- ...

### ❌ Failed

- Test 8: Host Key Verification (Changed Key)
  - **Issue**: Connection not rejected
  - **Expected**: Error message
  - **Actual**: Connected successfully
  - **Severity**: High (security)

### ⚠ Skipped

- Test 10: Multiple Concurrent Sessions
  - **Reason**: Feature not yet implemented

## Performance

- Connection Time: 850ms (✓ < 2s target)
- Terminal Latency: 35ms (✓ < 50ms target)
- Memory Usage: 95 MB (✓ < 100 MB target)

## Recommendations

1. Fix host key rejection logic
2. Add multiple tab support
3. Improve error messages

## Overall Assessment

**Status**: Ready for Beta / Needs Work / Production Ready
```

---

**Happy Testing!** 🧪

For questions or issues, see: [GitHub Issues](https://github.com/your-repo/pulsar/issues)
