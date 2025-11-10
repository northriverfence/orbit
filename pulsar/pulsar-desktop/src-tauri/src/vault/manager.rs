use anyhow::{anyhow, Context, Result};
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::crypto::{MasterKey, VaultCrypto};
use super::storage::{Credential, CredentialType, VaultMetadata, VaultStorage};

/// Vault state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VaultState {
    Uninitialized,
    Locked,
    Unlocked,
}

/// Decrypted credential data for SSH keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshKeyData {
    pub private_key: String,
    pub public_key: Option<String>,
    pub passphrase: Option<String>,
}

/// Decrypted credential data for passwords
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordData {
    pub password: String,
    pub username: Option<String>,
}

/// Decrypted credential data for certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateData {
    pub certificate: String,
    pub private_key: Option<String>,
    pub passphrase: Option<String>,
}

/// Decrypted credential data (union type)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DecryptedCredentialData {
    SshKey(SshKeyData),
    Password(PasswordData),
    Certificate(CertificateData),
}

/// Decrypted credential
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedCredential {
    pub id: String,
    pub name: String,
    pub data: DecryptedCredentialData,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub username: Option<String>,
    pub host_pattern: Option<String>,
}

/// Credential summary (for listing without decryption)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialSummary {
    pub id: String,
    pub name: String,
    pub credential_type: CredentialType,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub username: Option<String>,
    pub host_pattern: Option<String>,
}

impl From<Credential> for CredentialSummary {
    fn from(cred: Credential) -> Self {
        Self {
            id: cred.id,
            name: cred.name,
            credential_type: cred.credential_type,
            tags: cred.tags,
            created_at: cred.created_at,
            updated_at: cred.updated_at,
            username: cred.username,
            host_pattern: cred.host_pattern,
        }
    }
}

/// Inner vault state
struct VaultInner {
    storage: VaultStorage,
    state: VaultState,
    master_key: Option<MasterKey>,
    metadata: Option<VaultMetadata>,
}

/// Vault manager
pub struct VaultManager {
    inner: Arc<RwLock<VaultInner>>,
}

impl VaultManager {
    /// Create a new vault manager
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let storage = VaultStorage::new(db_path).await?;

        let is_initialized = storage.is_initialized().await?;
        let state = if is_initialized {
            VaultState::Locked
        } else {
            VaultState::Uninitialized
        };

        let metadata = if is_initialized {
            storage.get_metadata().await?
        } else {
            None
        };

        Ok(Self {
            inner: Arc::new(RwLock::new(VaultInner {
                storage,
                state,
                master_key: None,
                metadata,
            })),
        })
    }

    /// Get current vault state
    pub async fn get_state(&self) -> VaultState {
        let inner = self.inner.read().await;
        inner.state.clone()
    }

    /// Check if vault is initialized
    pub async fn is_initialized(&self) -> bool {
        let inner = self.inner.read().await;
        inner.state != VaultState::Uninitialized
    }

    /// Check if vault is unlocked
    pub async fn is_unlocked(&self) -> bool {
        let inner = self.inner.read().await;
        inner.state == VaultState::Unlocked
    }

    /// Initialize vault with a master password
    pub async fn initialize(&self, master_password: &str) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.state != VaultState::Uninitialized {
            return Err(anyhow!("Vault is already initialized"));
        }

        // Generate salt
        let salt = MasterKey::generate_salt();
        let salt_b64 = general_purpose::STANDARD.encode(&salt);

        // Derive master key
        let master_key = MasterKey::derive_from_password(master_password, &salt)?;

        // Create password hash for verification
        use argon2::{
            password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
            Argon2,
        };
        let argon2 = Argon2::default();
        let salt_string = SaltString::generate(&mut OsRng);
        let password_hash = argon2
            .hash_password(master_password.as_bytes(), &salt_string)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?
            .to_string();

        // Create metadata
        let now = chrono::Utc::now().timestamp_millis();
        let metadata = VaultMetadata {
            password_hash,
            salt: salt_b64,
            version: 1,
            created_at: now,
            last_unlocked_at: now,
        };

        // Store metadata
        inner.storage.initialize(&metadata).await?;

        // Update state
        inner.state = VaultState::Unlocked;
        inner.master_key = Some(master_key);
        inner.metadata = Some(metadata);

        tracing::info!("Vault initialized successfully");

        Ok(())
    }

    /// Unlock vault with master password
    pub async fn unlock(&self, master_password: &str) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.state == VaultState::Uninitialized {
            return Err(anyhow!("Vault is not initialized"));
        }

        if inner.state == VaultState::Unlocked {
            return Err(anyhow!("Vault is already unlocked"));
        }

        // Get metadata
        let metadata = inner.storage.get_metadata().await?
            .ok_or_else(|| anyhow!("Vault metadata not found"))?;

        // Verify password
        if !MasterKey::verify_password(master_password, &metadata.password_hash)? {
            return Err(anyhow!("Invalid password"));
        }

        // Decode salt
        let salt = general_purpose::STANDARD.decode(&metadata.salt)
            .context("Failed to decode salt")?;

        // Derive master key
        let master_key = MasterKey::derive_from_password(master_password, &salt)?;

        // Update last unlocked timestamp
        inner.storage.update_last_unlocked().await?;

        // Update state
        inner.state = VaultState::Unlocked;
        inner.master_key = Some(master_key);
        inner.metadata = Some(metadata);

        tracing::info!("Vault unlocked successfully");

        Ok(())
    }

    /// Lock vault
    pub async fn lock(&self) -> Result<()> {
        let mut inner = self.inner.write().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is not unlocked"));
        }

        // Clear master key from memory
        inner.master_key = None;
        inner.state = VaultState::Locked;

        tracing::info!("Vault locked");

        Ok(())
    }

    /// Store a credential (vault must be unlocked)
    pub async fn store_credential(&self, name: String, data: DecryptedCredentialData, tags: Vec<String>, username: Option<String>, host_pattern: Option<String>) -> Result<String> {
        let mut inner = self.inner.write().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is locked"));
        }

        let master_key = inner.master_key.as_ref()
            .ok_or_else(|| anyhow!("Master key not available"))?;

        // Serialize credential data
        let plaintext = serde_json::to_string(&data)?;

        // Encrypt data
        let encrypted_data = VaultCrypto::encrypt_string(master_key, &plaintext)?;

        // Determine credential type
        let credential_type = match &data {
            DecryptedCredentialData::SshKey(_) => CredentialType::SshKey,
            DecryptedCredentialData::Password(_) => CredentialType::Password,
            DecryptedCredentialData::Certificate(_) => CredentialType::Certificate,
        };

        // Create credential
        let now = chrono::Utc::now().timestamp_millis();
        let id = Uuid::new_v4().to_string();
        let credential = Credential {
            id: id.clone(),
            name,
            credential_type,
            encrypted_data,
            tags,
            created_at: now,
            updated_at: now,
            username,
            host_pattern,
        };

        // Store credential
        inner.storage.store_credential(&credential).await?;

        tracing::info!("Stored credential: {}", id);

        Ok(id)
    }

    /// Get a decrypted credential by ID
    pub async fn get_credential(&self, id: &str) -> Result<DecryptedCredential> {
        let inner = self.inner.read().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is locked"));
        }

        let master_key = inner.master_key.as_ref()
            .ok_or_else(|| anyhow!("Master key not available"))?;

        // Get encrypted credential
        let credential = inner.storage.get_credential(id).await?
            .ok_or_else(|| anyhow!("Credential not found: {}", id))?;

        // Decrypt data
        let plaintext = VaultCrypto::decrypt_string(master_key, &credential.encrypted_data)?;

        // Deserialize data
        let data: DecryptedCredentialData = serde_json::from_str(&plaintext)
            .context("Failed to deserialize credential data")?;

        Ok(DecryptedCredential {
            id: credential.id,
            name: credential.name,
            data,
            tags: credential.tags,
            created_at: credential.created_at,
            updated_at: credential.updated_at,
            username: credential.username,
            host_pattern: credential.host_pattern,
        })
    }

    /// List all credentials (without decrypting)
    pub async fn list_credentials(&self) -> Result<Vec<CredentialSummary>> {
        let inner = self.inner.read().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is locked"));
        }

        let credentials = inner.storage.list_credentials().await?;
        Ok(credentials.into_iter().map(CredentialSummary::from).collect())
    }

    /// List credentials by type
    pub async fn list_credentials_by_type(&self, credential_type: CredentialType) -> Result<Vec<CredentialSummary>> {
        let inner = self.inner.read().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is locked"));
        }

        let credentials = inner.storage.list_credentials_by_type(&credential_type).await?;
        Ok(credentials.into_iter().map(CredentialSummary::from).collect())
    }

    /// Find credentials by host pattern
    pub async fn find_credentials_by_host(&self, host: &str) -> Result<Vec<CredentialSummary>> {
        let inner = self.inner.read().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is locked"));
        }

        let credentials = inner.storage.find_credentials_by_host(host).await?;
        Ok(credentials.into_iter().map(CredentialSummary::from).collect())
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: &str) -> Result<()> {
        let inner = self.inner.read().await;

        if inner.state != VaultState::Unlocked {
            return Err(anyhow!("Vault is locked"));
        }

        inner.storage.delete_credential(id).await?;

        tracing::info!("Deleted credential: {}", id);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_vault() -> (VaultManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");
        let vault = VaultManager::new(db_path).await.unwrap();
        (vault, temp_dir)
    }

    #[tokio::test]
    async fn test_vault_initialization() {
        let (vault, _temp_dir) = create_test_vault().await;

        assert_eq!(vault.get_state().await, VaultState::Uninitialized);
        assert!(!vault.is_initialized().await);

        vault.initialize("test_password_123").await.unwrap();

        assert_eq!(vault.get_state().await, VaultState::Unlocked);
        assert!(vault.is_initialized().await);
    }

    #[tokio::test]
    async fn test_vault_lock_unlock() {
        let (vault, _temp_dir) = create_test_vault().await;

        vault.initialize("test_password_123").await.unwrap();
        assert!(vault.is_unlocked().await);

        vault.lock().await.unwrap();
        assert!(!vault.is_unlocked().await);

        vault.unlock("test_password_123").await.unwrap();
        assert!(vault.is_unlocked().await);
    }

    #[tokio::test]
    async fn test_wrong_password() {
        let (vault, _temp_dir) = create_test_vault().await;

        vault.initialize("correct_password").await.unwrap();
        vault.lock().await.unwrap();

        let result = vault.unlock("wrong_password").await;
        assert!(result.is_err());
        assert!(!vault.is_unlocked().await);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_ssh_key() {
        let (vault, _temp_dir) = create_test_vault().await;

        vault.initialize("test_password_123").await.unwrap();

        let ssh_key_data = DecryptedCredentialData::SshKey(SshKeyData {
            private_key: "-----BEGIN OPENSSH PRIVATE KEY-----\ntest_key\n-----END OPENSSH PRIVATE KEY-----".to_string(),
            public_key: Some("ssh-rsa AAAA... user@host".to_string()),
            passphrase: Some("key_passphrase".to_string()),
        });

        let id = vault.store_credential(
            "My SSH Key".to_string(),
            ssh_key_data.clone(),
            vec!["work".to_string()],
            Some("admin".to_string()),
            Some("*.example.com".to_string()),
        ).await.unwrap();

        let retrieved = vault.get_credential(&id).await.unwrap();
        assert_eq!(retrieved.name, "My SSH Key");
        assert_eq!(retrieved.tags, vec!["work"]);

        if let DecryptedCredentialData::SshKey(data) = retrieved.data {
            assert!(data.private_key.contains("test_key"));
            assert_eq!(data.passphrase, Some("key_passphrase".to_string()));
        } else {
            panic!("Expected SSH key data");
        }
    }

    #[tokio::test]
    async fn test_list_credentials() {
        let (vault, _temp_dir) = create_test_vault().await;

        vault.initialize("test_password_123").await.unwrap();

        let ssh_key = DecryptedCredentialData::SshKey(SshKeyData {
            private_key: "key1".to_string(),
            public_key: None,
            passphrase: None,
        });

        let password = DecryptedCredentialData::Password(PasswordData {
            password: "secret123".to_string(),
            username: Some("user".to_string()),
        });

        vault.store_credential("SSH Key".to_string(), ssh_key, vec![], None, None).await.unwrap();
        vault.store_credential("Password".to_string(), password, vec![], None, None).await.unwrap();

        let all = vault.list_credentials().await.unwrap();
        assert_eq!(all.len(), 2);

        let ssh_keys = vault.list_credentials_by_type(CredentialType::SshKey).await.unwrap();
        assert_eq!(ssh_keys.len(), 1);
    }

    #[tokio::test]
    async fn test_find_by_host() {
        let (vault, _temp_dir) = create_test_vault().await;

        vault.initialize("test_password_123").await.unwrap();

        let ssh_key = DecryptedCredentialData::SshKey(SshKeyData {
            private_key: "key".to_string(),
            public_key: None,
            passphrase: None,
        });

        vault.store_credential(
            "Production Key".to_string(),
            ssh_key,
            vec![],
            None,
            Some("prod.example.com".to_string()),
        ).await.unwrap();

        let found = vault.find_credentials_by_host("prod.example").await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "Production Key");
    }

    #[tokio::test]
    async fn test_cannot_access_when_locked() {
        let (vault, _temp_dir) = create_test_vault().await;

        vault.initialize("test_password_123").await.unwrap();

        let ssh_key = DecryptedCredentialData::SshKey(SshKeyData {
            private_key: "key".to_string(),
            public_key: None,
            passphrase: None,
        });

        let id = vault.store_credential("Key".to_string(), ssh_key, vec![], None, None).await.unwrap();

        vault.lock().await.unwrap();

        let result = vault.get_credential(&id).await;
        assert!(result.is_err());
    }
}
