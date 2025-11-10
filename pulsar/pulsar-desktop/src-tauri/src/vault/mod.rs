mod crypto;
mod manager;
mod storage;

pub use manager::{
    CertificateData, CredentialSummary, DecryptedCredential, DecryptedCredentialData,
    PasswordData, SshKeyData, VaultManager, VaultState,
};
pub use storage::CredentialType;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Global vault instance
pub struct Vault {
    manager: Arc<RwLock<Option<VaultManager>>>,
}

impl Vault {
    /// Create a new vault instance
    pub fn new() -> Self {
        Self {
            manager: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the vault manager with a database path
    pub async fn init(&self, db_path: std::path::PathBuf) -> Result<()> {
        let manager = VaultManager::new(db_path).await?;
        let mut lock = self.manager.write().await;
        *lock = Some(manager);
        Ok(())
    }

    /// Get the vault manager
    pub async fn get_manager(&self) -> Result<Arc<RwLock<Option<VaultManager>>>> {
        Ok(self.manager.clone())
    }

    /// Execute an operation with the vault manager
    pub async fn with_manager<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce(&VaultManager) -> futures::future::BoxFuture<'_, Result<R>>,
    {
        let lock = self.manager.read().await;
        let manager = lock.as_ref().ok_or_else(|| anyhow::anyhow!("Vault not initialized"))?;
        f(manager).await
    }
}

impl Default for Vault {
    fn default() -> Self {
        Self::new()
    }
}
