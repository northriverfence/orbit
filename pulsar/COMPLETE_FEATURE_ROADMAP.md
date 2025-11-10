# Pulsar - Complete Feature Roadmap

Based on pricing and features analysis from reference images.

## Pricing Tiers

### Starter (FREE)
**Target**: Home lab enthusiasts seeking a modern SSH client

**Features**:
- ✅ SSH and SFTP
- ✅ Local vault
- 🔄 AI-powered autocomplete
- ✅ Port forwarding
- ✅ Telnet support
- ✅ All cloud catalog integrations (AWS, DigitalOcean, Azure)
- ✅ All automation features (snippets, history, autocomplete, startup snippets, multi-execution, environment variables)
- ✅ All tunneling features (port forwarding, SOCKS proxy, jump host, agent forwarding)
- ✅ Basic security (session logs, PIN lock, biometric keys, TouchID/FaceID, FIDO2, SSH certificates)
- ✅ Terminal sharing via link

### Pro ($10/month, paid annually)
**Target**: Individuals responsible for infrastructure 24/7

**All Starter features plus**:
- 🔄 Personal vault (cloud sync)
- 🔄 Sync across mobile and desktop
- 🔄 Snippets automation (enhanced)
- ✅ HTTP Proxy tunneling

### Team ($20/user/month, paid annually)
**Target**: Teams managing infrastructure together

**All Pro features plus**:
- 🔄 Team vault for secure sharing
- 🔄 Real-time collaboration
- 🔄 Consolidated billing
- 🔄 API Bridge integration
- 🔄 Ansible integration
- 🔄 Verta integration

### Business ($30/user/month, paid annually)
**Target**: Companies requiring access control and advanced security

**All Team features plus**:
- 🔄 Multiple vaults with granular permissions
- 🔄 SOC2 Type II report
- 🔄 SAML SSO
- 🔄 Purchase order support
- 🔄 Payment by bank transfer
- 🔄 Dedicated success manager

---

## Feature Implementation Roadmap

### Phase 1: Core Foundation (Weeks 1-6) ⏳ IN PROGRESS

**Status**: 70% Complete

**Completed**:
- [x] Project structure and Cargo workspace
- [x] Tauri 2.9 + React setup
- [x] Collapsible sidebar UI
- [x] TFT protocol foundation
- [x] SSH/SFTP transport layer (placeholder)

**In Progress**:
- [ ] xterm.js integration
- [ ] Basic SSH connection (russh)
- [ ] File transfer UI

**Remaining**:
- [ ] Server management (add/edit/delete)
- [ ] Session persistence
- [ ] Local vault implementation

### Phase 2: Starter Tier Complete (Weeks 7-12)

**Protocols**:
- [ ] SSH client (russh)
- [ ] SFTP implementation (russh-sftp)
- [ ] Telnet support
- [ ] Serial port communication

**Local Vault**:
- [ ] OS keyring integration (macOS Keychain, Windows Credential Manager, Linux Secret Service)
- [ ] Credential storage (username/password)
- [ ] SSH key management
- [ ] Certificate storage
- [ ] Secure encryption at rest

**Tunneling**:
- [ ] Port forwarding (local, remote, dynamic)
- [ ] SOCKS proxy
- [ ] HTTP proxy (Pro tier preview)
- [ ] Jump host / host chain
- [ ] Agent forwarding

**Automation**:
- [ ] Snippet system (create, edit, organize)
- [ ] Shell command history (searchable)
- [ ] Autocomplete engine
- [ ] Startup snippets (run on connect)
- [ ] Snippet multi-execution
- [ ] Environment variable management

**Security (Starter)**:
- [ ] Session logging (all commands, output)
- [ ] PIN lock for app
- [ ] Biometric authentication (TouchID, FaceID, Windows Hello)
- [ ] FIDO2 hardware key support
- [ ] SSH certificate authentication

**Cloud Integrations (Starter)**:
- [ ] AWS catalog (EC2 instance discovery)
- [ ] DigitalOcean droplet listing
- [ ] Azure VM discovery
- [ ] Auto-import connections from cloud APIs

**UI/UX**:
- [ ] Workspace management
- [ ] Multi-terminal grid (2x2)
- [ ] Pulse Link broadcast mode
- [ ] Theme system
- [ ] Font customization
- [ ] Mouse functionality (selection, URLs, context menu)
- [ ] Keybinding customization

### Phase 3: Pro Tier Features (Weeks 13-18)

**Personal Vault**:
- [ ] Cloud storage backend (encrypted)
- [ ] End-to-end encryption
- [ ] Cross-device sync (desktop ↔ mobile)
- [ ] Conflict resolution
- [ ] Offline mode

**Mobile Support**:
- [ ] iOS app (Tauri Mobile)
- [ ] Android app (Tauri Mobile)
- [ ] Tablet optimization
- [ ] Touch-optimized UI
- [ ] Background sync

**Enhanced Features**:
- [ ] Advanced snippet automation
- [ ] HTTP proxy tunneling
- [ ] Scheduled tasks

### Phase 4: Team Tier Features (Weeks 19-24)

**Team Vault**:
- [ ] Shared credential storage
- [ ] Role-based access control (RBAC)
- [ ] Team member management
- [ ] Audit logs for vault access
- [ ] Sharing permissions (view, edit, admin)

**Real-time Collaboration**:
- [ ] Terminal sharing with live cursors
- [ ] Co-editing in same session
- [ ] Voice/video integration (optional)
- [ ] Session recording and playback
- [ ] Collaboration analytics

**Cloud Integrations (Team)**:
- [ ] API Bridge (custom integrations)
- [ ] Ansible inventory import
- [ ] Verta integration

**Billing & Admin**:
- [ ] Consolidated billing dashboard
- [ ] Usage analytics per team member
- [ ] Invoice management
- [ ] Team analytics

### Phase 5: Business Tier Features (Weeks 25-30)

**Advanced Security**:
- [ ] SAML SSO integration
  - [ ] Okta
  - [ ] Azure AD
  - [ ] Google Workspace
  - [ ] Custom SAML providers
- [ ] SOC2 Type II compliance
  - [ ] Audit logging
  - [ ] Data encryption certification
  - [ ] Access control documentation
  - [ ] Incident response procedures

**Multiple Vaults**:
- [ ] Vault creation and management
- [ ] Granular permissions per vault
- [ ] Vault-level RBAC
- [ ] Cross-vault search
- [ ] Vault inheritance

**Enterprise Admin**:
- [ ] Purchase order support
- [ ] Bank transfer payments
- [ ] Custom contracts
- [ ] Dedicated success manager portal
- [ ] Enterprise SLA
- [ ] Priority support

**Compliance & Reporting**:
- [ ] SOC2 Type II report generation
- [ ] Audit trail export
- [ ] Compliance dashboard
- [ ] Security posture reporting

### Phase 6: Advanced Features (Weeks 31-36)

**AI Integration**:
- [ ] AI-powered autocomplete (OpenAI/Claude)
- [ ] Command suggestions
- [ ] Error detection and fixes
- [ ] Script generation
- [ ] Natural language commands

**Performance & Scale**:
- [ ] GPU-accelerated terminal rendering (wgpu)
- [ ] Large file transfer optimization
- [ ] Session multiplexing
- [ ] WebRTC P2P for collaboration
- [ ] QUIC/HTTP/3 for file transfers

**IDE Integrations**:
- [ ] VS Code extension
- [ ] Visual Studio extension
- [ ] JetBrains plugin
- [ ] Sublime Text plugin

**Web Version**:
- [ ] Browser-based client
- [ ] Chrome/Firefox extensions
- [ ] ChromeOS native app
- [ ] Progressive Web App (PWA)

---

## Technical Implementation Details

### Security Architecture

**Encryption**:
- TLS 1.3 for all network traffic
- ChaCha20-Poly1305 for file encryption
- AES-256-GCM for vault encryption
- BLAKE3 for integrity verification

**Authentication**:
- OAuth2/OIDC for SSO
- SAML 2.0 for enterprise SSO
- WebAuthn/FIDO2 for hardware keys
- Biometric (platform-native APIs)

**Key Management**:
- HSM support for enterprise (PKCS#11)
- Per-tenant encryption keys
- Key rotation policies
- Vault master key encryption

### Cloud Sync Architecture

**Backend Options**:
1. **Self-hosted** (Starter/Pro):
   - S3-compatible storage
   - WebDAV
   - Local network share

2. **Managed Cloud** (Team/Business):
   - AWS S3 + DynamoDB
   - Azure Blob Storage + Cosmos DB
   - Google Cloud Storage + Firestore

**Sync Protocol**:
- Differential sync (only changed data)
- Conflict-free replicated data types (CRDTs)
- Event sourcing for audit trail
- Real-time WebSocket notifications

### Collaboration Architecture

**Real-time Protocol**:
- WebRTC DataChannels for P2P
- Signaling server for connection establishment
- Fallback to TURN relay for NAT traversal
- Operational Transform (OT) for cursor positions

**Session Sharing**:
- Ephemeral share links (time-limited)
- Read-only vs. interactive modes
- Recording on/off toggle
- Participant management

### Database Schema

**Local (SQLite)**:
- Servers, sessions, vaults
- Command history, snippets
- Configuration, preferences

**Cloud (PostgreSQL/DynamoDB)**:
- User accounts, teams
- Shared vaults, permissions
- Billing, subscriptions
- Audit logs

---

## Development Priorities

### Critical Path (Must Have)
1. SSH/SFTP with local vault
2. Port forwarding and tunneling
3. Snippet automation
4. Cloud provider integrations
5. Terminal sharing

### High Priority (Should Have)
1. Personal vault with sync
2. Team vault and RBAC
3. Real-time collaboration
4. Mobile apps
5. AI autocomplete

### Medium Priority (Nice to Have)
1. SAML SSO
2. Multiple vaults
3. IDE integrations
4. Web version
5. Advanced analytics

### Low Priority (Future)
1. Voice/video in collaboration
2. Custom SAML providers
3. White-label options
4. On-premise deployment

---

## Monetization Strategy

### Free Tier Strategy
- Full local functionality
- No credit card required
- Upsell to Pro for sync
- Community support

### Pro Tier Conversion
- Free trial (14 days)
- Annual discount (2 months free)
- Personal use focus
- Email support

### Team Tier
- Free trial (30 days)
- Volume discounts (10+ users)
- Quarterly business reviews
- Slack/Teams support

### Business Tier
- Custom pricing
- Enterprise agreements
- Dedicated support
- On-site training

---

## Success Metrics

### User Acquisition
- Free tier signups
- Trial conversions
- Team seat growth
- Enterprise wins

### Engagement
- Daily active users (DAU)
- Sessions per user
- Connections managed
- Snippets created

### Revenue
- Monthly Recurring Revenue (MRR)
- Annual Contract Value (ACV)
- Churn rate
- Net Revenue Retention (NRR)

### Product
- Feature adoption rates
- Performance metrics (latency, uptime)
- Support ticket volume
- NPS score

---

**Status**: Phase 1 in progress (70% complete)
**Next Milestone**: Starter tier feature complete (Phase 2)
**Target Date**: Q2 2026 for full Business tier
