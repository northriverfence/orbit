# Pulsar

**Professional SSH Terminal & Connection Manager**

Pulsar is a cross-platform SSH terminal emulator with advanced file transfer capabilities, designed for power users and teams managing multiple remote servers.

## Features

### Terminal Management
- **Multi-session support**: Manage multiple SSH connections simultaneously
- **Workspace organization**: Group terminals into named workspaces
- **Pulse Link broadcast**: Send input to multiple terminals at once
- **Session persistence**: Resume connections after restart

### File Transfer (TFT Protocol)
- **Multiple transports**: QUIC/HTTP/3 (primary), SSH/SFTP (fallback), WebRTC (P2P)
- **Chunked transfers**: 1MB chunks with independent compression (zstd)
- **Resume capability**: Pick up where you left off after network failures
- **Integrity verification**: BLAKE3 hashing with Merkle tree verification
- **End-to-end encryption**: Optional ChaCha20-Poly1305 encryption

### Security
- **Vault system**: Secure storage for credentials, SSH keys, certificates
- **OS keyring integration**: Native credential storage
- **MFA support**: WebAuthn/FIDO2, TOTP, SMS/Email OTP
- **TLS 1.3**: Modern encryption for all network transports

### UI/UX
- **Collapsible sidebar**: Only one section expands at a time
- **Professional design**: Clean, desktop-native appearance
- **Theme support**: Customize appearance to match your preferences
- **Multi-terminal view**: Up to 4 terminals in 2x2 grid layout

## Project Structure

```
pulsar/
├── tft-core/              # TFT protocol implementation (Rust)
├── tft-transports/        # Transport layer (QUIC, SSH, WebRTC)
├── terminal-core/         # PTY and terminal emulation (Rust)
├── pulsar-daemon/         # Background service (Rust)
└── pulsar-desktop/        # Desktop GUI (Tauri 2.9 + React)
    ├── src/               # React frontend (TypeScript)
    └── src-tauri/         # Tauri backend (Rust)
```

## Getting Started

### Prerequisites

- **Rust**: 1.75+ (install from [rustup.rs](https://rustup.rs))
- **Node.js**: 20+ (for frontend development)
- **Bun** or **npm**: Package manager

### Development Setup

1. **Clone and navigate**:
   ```bash
   cd /opt/singulio-dev/tools/shell/fork/orbit/pulsar
   ```

2. **Install frontend dependencies**:
   ```bash
   cd pulsar-desktop
   npm install  # or: bun install
   ```

3. **Run in development mode**:
   ```bash
   npm run tauri dev
   ```

   This will:
   - Start Vite dev server (port 5173)
   - Compile Rust backend
   - Launch Tauri desktop app with hot-reload

### Building for Production

```bash
cd pulsar-desktop
npm run tauri build
```

Built applications will be in `pulsar-desktop/src-tauri/target/release/bundle/`.

## Architecture

### Frontend (React + TypeScript)
- **Framework**: React 18 with Vite
- **Styling**: Tailwind CSS
- **Terminal**: xterm.js with addons (fit, web-links, search)
- **State**: React hooks (will add Zustand for complex state)

### Backend (Rust + Tauri)
- **Framework**: Tauri 2.9
- **Async runtime**: Tokio
- **Terminal**: portable-pty + vte parser
- **Networking**: quinn (QUIC), russh (SSH), webrtc

### Communication
- **Frontend ↔ Backend**: Tauri IPC commands
- **Desktop ↔ Daemon**: Unix socket (optional)
- **Desktop ↔ Orbit**: Unix socket (optional)

## Development Roadmap

### Phase 1: Foundation (Weeks 1-6) ✅ In Progress
- [x] Project structure and Cargo workspace
- [x] Tauri 2.9 + React setup
- [x] Collapsible sidebar UI (accordion pattern)
- [ ] xterm.js integration
- [ ] Basic SSH connection (russh)
- [ ] TFT protocol core (NDJSON messages)

### Phase 2: Core Features (Weeks 7-12)
- [ ] Multi-session management
- [ ] Workspace system
- [ ] File transfer UI (drag-drop)
- [ ] QUIC/HTTP/3 transport
- [ ] SSH/SFTP transport
- [ ] Vault system (credentials, keys)

### Phase 3: Advanced Features (Weeks 13-18)
- [ ] Pulse Link broadcast mode
- [ ] Multi-terminal grid view
- [ ] Port tunneling (local, remote, dynamic)
- [ ] Session persistence and resume
- [ ] Theme system

### Phase 4: Polish & Distribution (Weeks 19-24)
- [ ] WebRTC transport
- [ ] Performance optimization
- [ ] Platform-specific builds (Windows, macOS, Linux)
- [ ] Documentation and tutorials
- [ ] Release v1.0

## Integration with Orbit

Pulsar can optionally integrate with the Orbit shell assistant:

- **Shared config**: Uses `~/.config/orbit/` directory
- **IPC notifications**: Notifies Orbit of session events
- **Standalone**: Works independently if Orbit is not running

## Contributing

This is an internal Singulio project. See the main Singulio repository for contribution guidelines.

## License

Proprietary - Singulio Platform Team

---

**Status**: Early Development (Phase 1)
**Version**: 0.1.0
**Last Updated**: 2025-10-31
