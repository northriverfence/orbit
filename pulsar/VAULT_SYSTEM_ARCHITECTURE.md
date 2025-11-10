# Pulsar Vault System - Architecture & Implementation Plan

**Date:** 2025-11-09
**Phase:** Week 3 of 6-Week MVP Plan
**Priority:** HIGH (Critical Security Feature)
**Estimated Time:** 7-10 days

---

## 🎯 Overview

The Vault System provides secure, encrypted storage for SSH credentials including:
- SSH private keys (RSA, Ed25519, ECDSA)
- Passwords (server passwords, passphrases)
- Client certificates (mTLS, SSH certificates)
- Connection metadata (hosts, ports, usernames)

---

## 🏗️ Architecture

### High-Level Design

```
┌─────────────────────────────────────────────────┐
│                 Pulsar Frontend                 │
│  ┌───────────────────────────────────────────┐  │
│  │         Vault UI Components               │  │
│  │  • VaultSidebar                           │  │
│  │  • CredentialList                         │  │
│  │  • CredentialForm (Add/Edit)              │  │
│  │  • UnlockDialog                           │  │
│  │  • MasterPasswordSetup                    │  │
│  └───────────────┬───────────────────────────┘  │
│                  │ Tauri Commands                │
└──────────────────┼───────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│              Tauri Backend (Rust)                │
│  ┌───────────────────────────────────────────┐  │
│  │           Vault Module                    │  │
│  │  • VaultManager (core logic)              │  │
│  │  • Encryption (ChaCha20-Poly1305)         │  │
│  │  • KeyDerivation (Argon2id)               │  │
│  │  • VaultStorage (encrypted SQLite)        │  │
│  └───────────────┬───────────────────────────┘  │
└──────────────────┼───────────────────────────────┘
                   │
┌──────────────────▼───────────────────────────────┐
│          Storage Backend Options                 │
│  • Option A: OS Keyring (keyring-rs)            │
│  • Option B: Encrypted SQLite (recommended)      │
│  • Option C: Hybrid (metadata in SQLite,         │
│              keys in OS keyring)                 │
└──────────────────────────────────────────────────┘
```

---

## 🔐 Security Design

### Encryption Stack

**Master Key Derivation (Argon2id):**
```rust
// User's master password → 256-bit encryption key
let salt = random_bytes(32);
let config = argon2::Config {
    variant: argon2::Variant::Argon2id,
    version: argon2::Version::Version13,
    mem_cost: 65536,      // 64 MB memory
    time_cost: 4,         // 4 iterations
    lanes: 4,             // 4 parallel threads
    thread_mode: argon2::ThreadMode::Parallel,
    secret: &[],
    ad: &[],
    hash_length: 32,      // 256 bits
};
let master_key = argon2::hash_raw(password.as_bytes(), &salt, &config)?;
```

**Data Encryption (ChaCha20-Poly1305):**
```rust
// Encrypt credential with master key
let cipher = ChaCha20Poly1305::new(&master_key);
let nonce = random_bytes(12); // 96-bit nonce
let ciphertext = cipher.encrypt(&nonce, plaintext.as_bytes())?;

// Store: nonce || ciphertext || tag (16 bytes)
```

**Key Derivation Parameters:**
- Algorithm: **Argon2id** (hybrid, resistant to side-channel attacks)
- Memory: **64 MB** (balances security vs. UX on low-end devices)
- Iterations: **4** (sub-second unlock on modern hardware)
- Salt: **32 bytes** (unique per vault)
- Output: **256-bit key**

### Security Properties

✅ **Confidentiality:** ChaCha20-Poly1305 authenticated encryption
✅ **Integrity:** AEAD tag prevents tampering
✅ **Forward Secrecy:** New nonce per encryption
✅ **Key Stretching:** Argon2id makes brute-force expensive
✅ **Memory-Hard:** Resistant to GPU/ASIC attacks
✅ **Side-Channel Resistant:** Argon2id designed for this

---

## 💾 Data Model

### Vault Schema (SQLite)

```sql
-- Vault metadata
CREATE TABLE vault_config (
    id INTEGER PRIMARY KEY CHECK (id = 1),  -- Only one config row
    salt BLOB NOT NULL,                      -- 32 bytes for Argon2
    argon2_params TEXT NOT NULL,             -- JSON: mem_cost, time_cost, etc.
    created_at INTEGER NOT NULL,
    last_unlocked INTEGER,
    auto_lock_minutes INTEGER DEFAULT 15,
    version INTEGER DEFAULT 1
);

-- Encrypted credentials
CREATE TABLE credentials (
    id TEXT PRIMARY KEY,                     -- UUID
    name TEXT NOT NULL,                      -- User-friendly name
    credential_type TEXT NOT NULL,           -- 'ssh_key', 'password', 'certificate'

    -- Encrypted fields (ChaCha20-Poly1305)
    encrypted_data BLOB NOT NULL,            -- Encrypted JSON payload
    nonce BLOB NOT NULL,                     -- 12 bytes

    -- Metadata (unencrypted for searching/filtering)
    host TEXT,                               -- Optional: associated host
    username TEXT,                           -- Optional: associated username
    port INTEGER,                            -- Optional: SSH port
    tags TEXT,                               -- JSON array: ["production", "aws"]

    created_at INTEGER NOT NULL,
    last_used INTEGER,
    use_count INTEGER DEFAULT 0,

    -- Expiry (optional)
    expires_at INTEGER,

    UNIQUE(name)
);

-- Index for fast lookups
CREATE INDEX idx_credentials_host ON credentials(host);
CREATE INDEX idx_credentials_type ON credentials(credential_type);
CREATE INDEX idx_credentials_last_used ON credentials(last_used DESC);

-- Audit log (encrypted)
CREATE TABLE audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp INTEGER NOT NULL,
    action TEXT NOT NULL,                    -- 'unlock', 'lock', 'add', 'delete', 'use'
    credential_id TEXT,                      -- Reference to credential
    details TEXT,                            -- Encrypted JSON
    FOREIGN KEY(credential_id) REFERENCES credentials(id) ON DELETE SET NULL
);
```

### Credential Types

**1. SSH Private Key:**
```json
{
  "type": "ssh_key",
  "algorithm": "ed25519",           // or "rsa", "ecdsa"
  "private_key": "-----BEGIN...---", // PEM format
  "public_key": "ssh-ed25519 AAAA...",
  "passphrase": "optional_passphrase",
  "key_size": 256,
  "fingerprint": "SHA256:abc123..."
}
```

**2. Password:**
```json
{
  "type": "password",
  "password": "secret_password",
  "username": "optional_username",
  "notes": "Additional notes"
}
```

**3. Certificate:**
```json
{
  "type": "certificate",
  "certificate": "-----BEGIN CERTIFICATE-----",
  "private_key": "-----BEGIN PRIVATE KEY-----",
  "ca_chain": ["-----BEGIN CERTIFICATE-----", ...],
  "format": "PEM"
}
```

---

## 🔧 Implementation Plan

### Phase 1: Backend Foundation (Days 1-3)

#### Task 1.1: Vault Core Module
**File:** `pulsar-desktop/src-tauri/src/vault/mod.rs`

```rust
pub mod crypto;
pub mod storage;
pub mod types;

use anyhow::Result;
use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct VaultManager {
    storage: Arc<storage::VaultStorage>,
    state: Arc<RwLock<VaultState>>,
}

pub enum VaultState {
    Locked,
    Unlocked {
        master_key: [u8; 32],
        unlocked_at: std::time::SystemTime,
    },
}

impl VaultManager {
    pub async fn new(vault_path: &Path) -> Result<Self>;
    pub async fn initialize(&self, master_password: &str) -> Result<()>;
    pub async fn unlock(&self, master_password: &str) -> Result<()>;
    pub async fn lock(&mut self) -> Result<()>;
    pub async fn is_unlocked(&self) -> bool;
    pub async fn change_master_password(&mut self, old: &str, new: &str) -> Result<()>;
}
```

#### Task 1.2: Encryption Module
**File:** `pulsar-desktop/src-tauri/src/vault/crypto.rs`

```rust
use argon2::{self, Config, ThreadMode, Variant, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;

pub struct VaultCrypto {
    argon2_config: Config<'static>,
}

impl VaultCrypto {
    pub fn derive_key(&self, password: &str, salt: &[u8]) -> Result<[u8; 32]>;
    pub fn encrypt(&self, key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>>;
    pub fn decrypt(&self, key: &[u8; 32], ciphertext: &[u8]) -> Result<Vec<u8>>;
    pub fn generate_salt() -> [u8; 32];
}
```

#### Task 1.3: Storage Module
**File:** `pulsar-desktop/src-tauri/src/vault/storage.rs`

```rust
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use crate::vault::types::*;

pub struct VaultStorage {
    pool: SqlitePool,
}

impl VaultStorage {
    pub async fn new(db_path: &Path) -> Result<Self>;
    pub async fn initialize_schema(&self) -> Result<()>;
    pub async fn get_vault_config(&self) -> Result<Option<VaultConfig>>;
    pub async fn save_vault_config(&self, config: &VaultConfig) -> Result<()>;

    // Credential CRUD
    pub async fn add_credential(&self, cred: &Credential) -> Result<()>;
    pub async fn get_credential(&self, id: &str) -> Result<Option<Credential>>;
    pub async fn list_credentials(&self, filter: &CredentialFilter) -> Result<Vec<CredentialMeta>>;
    pub async fn update_credential(&self, id: &str, cred: &Credential) -> Result<()>;
    pub async fn delete_credential(&self, id: &str) -> Result<()>;
    pub async fn record_credential_use(&self, id: &str) -> Result<()>;

    // Audit log
    pub async fn log_action(&self, action: &AuditEntry) -> Result<()>;
}
```

#### Task 1.4: Tauri Commands
**File:** `pulsar-desktop/src-tauri/src/vault/commands.rs`

```rust
use tauri::State;
use crate::vault::{VaultManager, types::*};

#[tauri::command]
pub async fn vault_initialize(
    master_password: String,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<(), String> { ... }

#[tauri::command]
pub async fn vault_unlock(
    master_password: String,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<(), String> { ... }

#[tauri::command]
pub async fn vault_lock(
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<(), String> { ... }

#[tauri::command]
pub async fn vault_is_unlocked(
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<bool, String> { ... }

#[tauri::command]
pub async fn vault_add_credential(
    name: String,
    credential_type: String,
    data: serde_json::Value,
    metadata: CredentialMetadata,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<String, String> { ... }

#[tauri::command]
pub async fn vault_get_credential(
    id: String,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<DecryptedCredential, String> { ... }

#[tauri::command]
pub async fn vault_list_credentials(
    filter: Option<CredentialFilter>,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<Vec<CredentialMeta>, String> { ... }

#[tauri::command]
pub async fn vault_delete_credential(
    id: String,
    vault: State<'_, Arc<RwLock<VaultManager>>>,
) -> Result<(), String> { ... }
```

---

### Phase 2: Frontend UI (Days 4-6)

#### Task 2.1: Vault State Management
**File:** `src/lib/vault-client.ts`

```typescript
export interface Credential {
  id: string;
  name: string;
  type: 'ssh_key' | 'password' | 'certificate';
  host?: string;
  username?: string;
  port?: number;
  tags: string[];
  createdAt: number;
  lastUsed?: number;
  useCount: number;
  expiresAt?: number;
}

export interface DecryptedCredential extends Credential {
  data: SSHKeyData | PasswordData | CertificateData;
}

export class VaultClient {
  async initialize(masterPassword: string): Promise<void>;
  async unlock(masterPassword: string): Promise<void>;
  async lock(): Promise<void>;
  async isUnlocked(): Promise<boolean>;

  async addCredential(name: string, type: string, data: any, metadata: any): Promise<string>;
  async getCredential(id: string): Promise<DecryptedCredential>;
  async listCredentials(filter?: CredentialFilter): Promise<Credential[]>;
  async deleteCredential(id: string): Promise<void>;
  async updateCredential(id: string, data: any): Promise<void>;
}
```

#### Task 2.2: Vault UI Components

**Component Structure:**
```
components/vault/
├── VaultSidebar.tsx           - Sidebar section with unlock status
├── UnlockDialog.tsx           - Master password prompt
├── MasterPasswordSetup.tsx    - Initial vault creation
├── CredentialList.tsx         - List of saved credentials
├── CredentialCard.tsx         - Individual credential display
├── CredentialForm.tsx         - Add/Edit credential form
├── SSHKeyForm.tsx             - SSH key specific form
├── PasswordForm.tsx           - Password specific form
├── CertificateForm.tsx        - Certificate specific form
└── VaultSettings.tsx          - Auto-lock, change password
```

#### Task 2.3: Master Password Setup
**File:** `src/components/vault/MasterPasswordSetup.tsx`

```tsx
export default function MasterPasswordSetup({ onComplete }: Props) {
  // Password strength meter
  // Confirmation field
  // Security recommendations
  // Initialize vault on submit
}
```

#### Task 2.4: Unlock Dialog
**File:** `src/components/vault/UnlockDialog.tsx`

```tsx
export default function UnlockDialog({ isOpen, onUnlock }: Props) {
  // Master password input
  // Remember for session checkbox
  // Unlock button
  // Error handling (wrong password)
}
```

#### Task 2.5: Credential List
**File:** `src/components/vault/CredentialList.tsx`

```tsx
export default function CredentialList({ filter }: Props) {
  // Search/filter UI
  // Group by type tabs
  // Credential cards
  // Quick actions (copy, delete, edit)
  // Add new credential button
}
```

---

### Phase 3: Integration (Days 7-8)

#### Task 3.1: Connection Dialog Integration

**File:** `src/components/ConnectionDialog.tsx`

```tsx
// Add vault credential selector
<Select
  label="Saved Credentials"
  options={vaultCredentials}
  onChange={(id) => autoFillFromVault(id)}
/>

// Add "Save to Vault" checkbox
<Checkbox
  label="Save credentials to vault"
  checked={saveToVault}
  onChange={setSaveToVault}
/>
```

#### Task 3.2: Auto-fill Logic

```tsx
const autoFillFromVault = async (credentialId: string) => {
  const cred = await vaultClient.getCredential(credentialId);

  if (cred.type === 'ssh_key') {
    setAuthMethod('privateKey');
    setPrivateKey(cred.data.private_key);
    setPassphrase(cred.data.passphrase || '');
  } else if (cred.type === 'password') {
    setAuthMethod('password');
    setPassword(cred.data.password);
  }

  setHost(cred.host || '');
  setUsername(cred.username || '');
  setPort(cred.port || 22);
};
```

#### Task 3.3: Save After Connection

```tsx
const handleConnect = async () => {
  const connected = await connectSSH(...);

  if (connected && saveToVault) {
    await vaultClient.addCredential(
      `${username}@${host}`,
      authMethod === 'privateKey' ? 'ssh_key' : 'password',
      authMethod === 'privateKey'
        ? { private_key: privateKey, passphrase }
        : { password },
      { host, username, port }
    );
  }
};
```

---

## 🔒 Security Considerations

### Threats & Mitigations

| Threat | Mitigation |
|--------|-----------|
| **Memory dumps** | Zero sensitive data after use, use SecureString |
| **Brute force** | Argon2id with high mem_cost (64 MB) |
| **Rainbow tables** | Unique 32-byte salt per vault |
| **Timing attacks** | Constant-time comparison for auth |
| **Database corruption** | Integrity checks, backup before modifications |
| **Malware screen capture** | Warn users, offer biometric unlock (future) |
| **Clipboard snooping** | Clear clipboard after 30 seconds |
| **Shoulder surfing** | Mask password inputs, use ••• display |

### Best Practices Implemented

✅ **Never log sensitive data** - No passwords/keys in logs
✅ **Zeroize after use** - Clear memory immediately
✅ **Auto-lock** - 15-minute idle timeout (configurable)
✅ **Strong defaults** - Argon2id recommended parameters
✅ **Audit trail** - Log all vault access (encrypted)
✅ **Export encryption** - Backup files also encrypted
✅ **Key rotation** - Support changing master password

---

## 📊 Success Criteria

### Functional Requirements
- [ ] User can create vault with master password
- [ ] User can unlock/lock vault
- [ ] User can add SSH keys, passwords, certificates
- [ ] User can list, search, filter credentials
- [ ] User can edit/delete credentials
- [ ] Credentials auto-fill in connection dialog
- [ ] Vault auto-locks after timeout
- [ ] User can change master password
- [ ] User can export/import vault (encrypted)

### Security Requirements
- [ ] Master password never stored plaintext
- [ ] All credentials encrypted at rest
- [ ] Encryption uses modern algorithms (Argon2id, ChaCha20-Poly1305)
- [ ] Vault survives app restart (persisted securely)
- [ ] No sensitive data in logs
- [ ] Memory cleared after unlock/lock
- [ ] Audit log tracks all access

### UX Requirements
- [ ] Unlock takes <2 seconds on modern hardware
- [ ] Password strength indicator during setup
- [ ] Clear error messages (wrong password, etc.)
- [ ] Intuitive credential management UI
- [ ] Quick access from sidebar
- [ ] Keyboard shortcuts for common actions

---

## 📦 Dependencies

### Rust Crates
```toml
[dependencies]
argon2 = "0.5"                    # Key derivation
chacha20poly1305 = "0.10"         # AEAD encryption
rand = "0.8"                      # Secure random
sqlx = { version = "0.8", features = ["sqlite", "runtime-tokio-rustls"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
zeroize = "1.7"                   # Secure memory clearing
```

### TypeScript Packages
```json
{
  "dependencies": {
    "@tauri-apps/api": "^1.5.0"
  }
}
```

---

## 🧪 Testing Plan

### Unit Tests (Rust)
- [x] Argon2 key derivation correctness
- [x] ChaCha20-Poly1305 encrypt/decrypt roundtrip
- [x] Salt generation uniqueness
- [x] Database schema creation
- [x] Credential CRUD operations

### Integration Tests
- [x] Vault initialization → unlock → add credential → lock → unlock → retrieve
- [x] Wrong password handling
- [x] Auto-lock after timeout
- [x] Change master password
- [x] Export/import vault

### Security Tests
- [x] Brute force resistance (timing)
- [x] Memory dumps (no sensitive data)
- [x] Tampering detection (AEAD tag)
- [x] SQL injection (parameterized queries)

### UI Tests
- [x] Master password setup flow
- [x] Unlock/lock flow
- [x] Add credential flow
- [x] Auto-fill from vault

---

## 📝 Documentation

1. **User Guide:** How to use vault system
2. **Security Model:** Encryption details for auditors
3. **API Reference:** Tauri commands and TypeScript client
4. **Migration Guide:** Importing from other password managers
5. **Troubleshooting:** Common issues and solutions

---

## 🚀 Future Enhancements (Post-MVP)

- [ ] Biometric unlock (TouchID, FaceID, Windows Hello)
- [ ] SSH agent integration (serve keys to system)
- [ ] Hardware security key support (YubiKey)
- [ ] Cloud sync (encrypted, E2E)
- [ ] Shared vaults (team credentials)
- [ ] TOTP/2FA token storage
- [ ] Password generator
- [ ] Credential sharing (temporary access)
- [ ] Browser extension integration
- [ ] CLI access to vault

---

## ⏱️ Timeline Estimate

**Total: 7-10 days**

- **Days 1-3:** Backend (Rust) - Vault core, encryption, storage, Tauri commands
- **Days 4-6:** Frontend (TypeScript/React) - UI components, state management
- **Days 7-8:** Integration - Connection dialog, auto-fill, testing
- **Days 9-10:** Polish, security review, documentation

**Critical Path:** Backend must be complete before frontend work can fully proceed

**Parallel Work Possible:** UI mockups and design can start on Day 1

---

**Status:** 📋 **ARCHITECTURE COMPLETE - READY FOR IMPLEMENTATION**
**Next:** 🔨 **START BACKEND IMPLEMENTATION**
