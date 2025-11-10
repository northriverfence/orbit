use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePool, Row};
use std::path::PathBuf;

/// Type of credential stored in the vault
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    SshKey,
    Password,
    Certificate,
}

impl CredentialType {
    fn to_string(&self) -> &'static str {
        match self {
            Self::SshKey => "ssh_key",
            Self::Password => "password",
            Self::Certificate => "certificate",
        }
    }

    fn from_string(s: &str) -> Result<Self> {
        match s {
            "ssh_key" => Ok(Self::SshKey),
            "password" => Ok(Self::Password),
            "certificate" => Ok(Self::Certificate),
            _ => Err(anyhow::anyhow!("Invalid credential type: {}", s)),
        }
    }
}

/// Credential stored in the vault (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    pub id: String,
    pub name: String,
    pub credential_type: CredentialType,
    /// Base64-encoded encrypted data
    pub encrypted_data: String,
    /// Optional tags for organization
    pub tags: Vec<String>,
    /// Unix timestamp (milliseconds)
    pub created_at: i64,
    /// Unix timestamp (milliseconds)
    pub updated_at: i64,
    /// Optional username/key name
    pub username: Option<String>,
    /// Optional host pattern (e.g., "*.example.com")
    pub host_pattern: Option<String>,
}

/// Vault metadata (stored unencrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    /// Argon2id password hash (for verification)
    pub password_hash: String,
    /// Salt for key derivation (base64-encoded)
    pub salt: String,
    /// Vault version
    pub version: i32,
    /// Unix timestamp (milliseconds)
    pub created_at: i64,
    /// Unix timestamp (milliseconds)
    pub last_unlocked_at: i64,
}

/// SQLite storage for the vault
pub struct VaultStorage {
    pool: SqlitePool,
}

impl VaultStorage {
    /// Create a new vault storage instance
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            tokio::fs::create_dir_all(parent).await
                .context("Failed to create vault directory")?;
        }

        // Connect to database
        let pool = SqlitePool::connect(&format!("sqlite://{}?mode=rwc", db_path.display()))
            .await
            .context("Failed to connect to vault database")?;

        // Run migrations
        Self::run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    /// Run database migrations
    async fn run_migrations(pool: &SqlitePool) -> Result<()> {
        // Create vault_metadata table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS vault_metadata (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                password_hash TEXT NOT NULL,
                salt TEXT NOT NULL,
                version INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                last_unlocked_at INTEGER NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create vault_metadata table")?;

        // Create credentials table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS credentials (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                credential_type TEXT NOT NULL,
                encrypted_data TEXT NOT NULL,
                tags TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                username TEXT,
                host_pattern TEXT
            )
            "#,
        )
        .execute(pool)
        .await
        .context("Failed to create credentials table")?;

        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_credentials_type ON credentials(credential_type)")
            .execute(pool)
            .await
            .context("Failed to create type index")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_credentials_name ON credentials(name)")
            .execute(pool)
            .await
            .context("Failed to create name index")?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_credentials_host ON credentials(host_pattern)")
            .execute(pool)
            .await
            .context("Failed to create host pattern index")?;

        Ok(())
    }

    /// Check if vault is initialized
    pub async fn is_initialized(&self) -> Result<bool> {
        let count: i32 = sqlx::query_scalar("SELECT COUNT(*) FROM vault_metadata")
            .fetch_one(&self.pool)
            .await
            .context("Failed to check if vault is initialized")?;

        Ok(count > 0)
    }

    /// Initialize the vault with metadata
    pub async fn initialize(&self, metadata: &VaultMetadata) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO vault_metadata (id, password_hash, salt, version, created_at, last_unlocked_at)
            VALUES (1, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&metadata.password_hash)
        .bind(&metadata.salt)
        .bind(metadata.version)
        .bind(metadata.created_at)
        .bind(metadata.last_unlocked_at)
        .execute(&self.pool)
        .await
        .context("Failed to initialize vault metadata")?;

        Ok(())
    }

    /// Get vault metadata
    pub async fn get_metadata(&self) -> Result<Option<VaultMetadata>> {
        let row = sqlx::query("SELECT password_hash, salt, version, created_at, last_unlocked_at FROM vault_metadata WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .context("Failed to fetch vault metadata")?;

        match row {
            Some(row) => Ok(Some(VaultMetadata {
                password_hash: row.try_get("password_hash")?,
                salt: row.try_get("salt")?,
                version: row.try_get("version")?,
                created_at: row.try_get("created_at")?,
                last_unlocked_at: row.try_get("last_unlocked_at")?,
            })),
            None => Ok(None),
        }
    }

    /// Update last unlocked timestamp
    pub async fn update_last_unlocked(&self) -> Result<()> {
        let now = chrono::Utc::now().timestamp_millis();
        sqlx::query("UPDATE vault_metadata SET last_unlocked_at = ? WHERE id = 1")
            .bind(now)
            .execute(&self.pool)
            .await
            .context("Failed to update last unlocked timestamp")?;

        Ok(())
    }

    /// Store a credential
    pub async fn store_credential(&self, credential: &Credential) -> Result<()> {
        let tags_json = serde_json::to_string(&credential.tags)?;

        sqlx::query(
            r#"
            INSERT INTO credentials (id, name, credential_type, encrypted_data, tags, created_at, updated_at, username, host_pattern)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                credential_type = excluded.credential_type,
                encrypted_data = excluded.encrypted_data,
                tags = excluded.tags,
                updated_at = excluded.updated_at,
                username = excluded.username,
                host_pattern = excluded.host_pattern
            "#,
        )
        .bind(&credential.id)
        .bind(&credential.name)
        .bind(credential.credential_type.to_string())
        .bind(&credential.encrypted_data)
        .bind(&tags_json)
        .bind(credential.created_at)
        .bind(credential.updated_at)
        .bind(&credential.username)
        .bind(&credential.host_pattern)
        .execute(&self.pool)
        .await
        .context("Failed to store credential")?;

        Ok(())
    }

    /// Get a credential by ID
    pub async fn get_credential(&self, id: &str) -> Result<Option<Credential>> {
        let row = sqlx::query(
            "SELECT id, name, credential_type, encrypted_data, tags, created_at, updated_at, username, host_pattern FROM credentials WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .context("Failed to fetch credential")?;

        match row {
            Some(row) => {
                let tags_json: String = row.try_get("tags")?;
                let tags: Vec<String> = serde_json::from_str(&tags_json)?;

                Ok(Some(Credential {
                    id: row.try_get("id")?,
                    name: row.try_get("name")?,
                    credential_type: CredentialType::from_string(row.try_get("credential_type")?)?,
                    encrypted_data: row.try_get("encrypted_data")?,
                    tags,
                    created_at: row.try_get("created_at")?,
                    updated_at: row.try_get("updated_at")?,
                    username: row.try_get("username")?,
                    host_pattern: row.try_get("host_pattern")?,
                }))
            }
            None => Ok(None),
        }
    }

    /// List all credentials (without decrypting)
    pub async fn list_credentials(&self) -> Result<Vec<Credential>> {
        let rows = sqlx::query(
            "SELECT id, name, credential_type, encrypted_data, tags, created_at, updated_at, username, host_pattern FROM credentials ORDER BY name"
        )
        .fetch_all(&self.pool)
        .await
        .context("Failed to list credentials")?;

        let mut credentials = Vec::new();
        for row in rows {
            let tags_json: String = row.try_get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)?;

            credentials.push(Credential {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                credential_type: CredentialType::from_string(row.try_get("credential_type")?)?,
                encrypted_data: row.try_get("encrypted_data")?,
                tags,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                username: row.try_get("username")?,
                host_pattern: row.try_get("host_pattern")?,
            });
        }

        Ok(credentials)
    }

    /// Search credentials by type
    pub async fn list_credentials_by_type(&self, credential_type: &CredentialType) -> Result<Vec<Credential>> {
        let rows = sqlx::query(
            "SELECT id, name, credential_type, encrypted_data, tags, created_at, updated_at, username, host_pattern FROM credentials WHERE credential_type = ? ORDER BY name"
        )
        .bind(credential_type.to_string())
        .fetch_all(&self.pool)
        .await
        .context("Failed to list credentials by type")?;

        let mut credentials = Vec::new();
        for row in rows {
            let tags_json: String = row.try_get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)?;

            credentials.push(Credential {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                credential_type: CredentialType::from_string(row.try_get("credential_type")?)?,
                encrypted_data: row.try_get("encrypted_data")?,
                tags,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                username: row.try_get("username")?,
                host_pattern: row.try_get("host_pattern")?,
            });
        }

        Ok(credentials)
    }

    /// Search credentials by host pattern
    pub async fn find_credentials_by_host(&self, host: &str) -> Result<Vec<Credential>> {
        // This is a simple LIKE search - could be enhanced with proper pattern matching
        let rows = sqlx::query(
            "SELECT id, name, credential_type, encrypted_data, tags, created_at, updated_at, username, host_pattern FROM credentials WHERE host_pattern LIKE ? ORDER BY name"
        )
        .bind(format!("%{}%", host))
        .fetch_all(&self.pool)
        .await
        .context("Failed to find credentials by host")?;

        let mut credentials = Vec::new();
        for row in rows {
            let tags_json: String = row.try_get("tags")?;
            let tags: Vec<String> = serde_json::from_str(&tags_json)?;

            credentials.push(Credential {
                id: row.try_get("id")?,
                name: row.try_get("name")?,
                credential_type: CredentialType::from_string(row.try_get("credential_type")?)?,
                encrypted_data: row.try_get("encrypted_data")?,
                tags,
                created_at: row.try_get("created_at")?,
                updated_at: row.try_get("updated_at")?,
                username: row.try_get("username")?,
                host_pattern: row.try_get("host_pattern")?,
            });
        }

        Ok(credentials)
    }

    /// Delete a credential
    pub async fn delete_credential(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM credentials WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .context("Failed to delete credential")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_storage() -> (VaultStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_vault.db");
        let storage = VaultStorage::new(db_path).await.unwrap();
        (storage, temp_dir)
    }

    #[tokio::test]
    async fn test_vault_initialization() {
        let (storage, _temp_dir) = create_test_storage().await;

        assert!(!storage.is_initialized().await.unwrap());

        let metadata = VaultMetadata {
            password_hash: "test_hash".to_string(),
            salt: "test_salt".to_string(),
            version: 1,
            created_at: 1234567890,
            last_unlocked_at: 1234567890,
        };

        storage.initialize(&metadata).await.unwrap();

        assert!(storage.is_initialized().await.unwrap());

        let retrieved = storage.get_metadata().await.unwrap().unwrap();
        assert_eq!(retrieved.password_hash, metadata.password_hash);
        assert_eq!(retrieved.salt, metadata.salt);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_credential() {
        let (storage, _temp_dir) = create_test_storage().await;

        let credential = Credential {
            id: "test-id".to_string(),
            name: "My SSH Key".to_string(),
            credential_type: CredentialType::SshKey,
            encrypted_data: "encrypted_base64_data".to_string(),
            tags: vec!["work".to_string(), "production".to_string()],
            created_at: 1234567890,
            updated_at: 1234567890,
            username: Some("admin".to_string()),
            host_pattern: Some("*.example.com".to_string()),
        };

        storage.store_credential(&credential).await.unwrap();

        let retrieved = storage.get_credential("test-id").await.unwrap().unwrap();
        assert_eq!(retrieved.name, credential.name);
        assert_eq!(retrieved.credential_type, credential.credential_type);
        assert_eq!(retrieved.tags, credential.tags);
        assert_eq!(retrieved.username, credential.username);
    }

    #[tokio::test]
    async fn test_list_credentials() {
        let (storage, _temp_dir) = create_test_storage().await;

        let cred1 = Credential {
            id: "id1".to_string(),
            name: "Credential 1".to_string(),
            credential_type: CredentialType::SshKey,
            encrypted_data: "data1".to_string(),
            tags: vec![],
            created_at: 1234567890,
            updated_at: 1234567890,
            username: None,
            host_pattern: None,
        };

        let cred2 = Credential {
            id: "id2".to_string(),
            name: "Credential 2".to_string(),
            credential_type: CredentialType::Password,
            encrypted_data: "data2".to_string(),
            tags: vec![],
            created_at: 1234567891,
            updated_at: 1234567891,
            username: None,
            host_pattern: None,
        };

        storage.store_credential(&cred1).await.unwrap();
        storage.store_credential(&cred2).await.unwrap();

        let all = storage.list_credentials().await.unwrap();
        assert_eq!(all.len(), 2);

        let ssh_keys = storage.list_credentials_by_type(&CredentialType::SshKey).await.unwrap();
        assert_eq!(ssh_keys.len(), 1);
        assert_eq!(ssh_keys[0].name, "Credential 1");
    }

    #[tokio::test]
    async fn test_delete_credential() {
        let (storage, _temp_dir) = create_test_storage().await;

        let credential = Credential {
            id: "test-id".to_string(),
            name: "Test".to_string(),
            credential_type: CredentialType::SshKey,
            encrypted_data: "data".to_string(),
            tags: vec![],
            created_at: 1234567890,
            updated_at: 1234567890,
            username: None,
            host_pattern: None,
        };

        storage.store_credential(&credential).await.unwrap();
        assert!(storage.get_credential("test-id").await.unwrap().is_some());

        storage.delete_credential("test-id").await.unwrap();
        assert!(storage.get_credential("test-id").await.unwrap().is_none());
    }
}
