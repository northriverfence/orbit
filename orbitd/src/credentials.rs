use anyhow::{anyhow, Result};
use keyring::Entry;
use tracing::{debug, warn};

/// Secure credential storage using system keychain
///
/// Stores API keys and other sensitive credentials in the system's secure storage:
/// - macOS: Keychain
/// - Linux: libsecret (GNOME Keyring, KWallet)
/// - Windows: Credential Manager
///
/// This provides protection against:
/// - Plaintext credential theft
/// - Process memory dumps revealing keys
/// - Accidental commits to version control
pub struct CredentialStore {
    service_name: String,
}

impl CredentialStore {
    /// Create a new credential store for Orbit
    pub fn new() -> Self {
        Self {
            service_name: "orbit".to_string(),
        }
    }

    /// Store an API key securely in the system keychain
    ///
    /// # Arguments
    /// * `provider` - Provider name (e.g., "claude", "openai", "gemini")
    /// * `api_key` - The API key to store
    ///
    /// # Example
    /// ```no_run
    /// let store = CredentialStore::new();
    /// store.set_api_key("claude", "sk-ant-...").await?;
    /// ```
    pub fn set_api_key(&self, provider: &str, api_key: &str) -> Result<()> {
        debug!("Storing API key for provider: {}", provider);

        let entry = Entry::new(&self.service_name, provider)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        entry
            .set_password(api_key)
            .map_err(|e| anyhow!("Failed to store API key in keychain: {}", e))?;

        debug!("API key stored successfully for provider: {}", provider);
        Ok(())
    }

    /// Retrieve an API key from the system keychain
    ///
    /// # Arguments
    /// * `provider` - Provider name (e.g., "claude", "openai", "gemini")
    ///
    /// # Returns
    /// The API key if found, or an error if not present or inaccessible
    ///
    /// # Example
    /// ```no_run
    /// let store = CredentialStore::new();
    /// let api_key = store.get_api_key("claude").await?;
    /// ```
    pub fn get_api_key(&self, provider: &str) -> Result<String> {
        debug!("Retrieving API key for provider: {}", provider);

        let entry = Entry::new(&self.service_name, provider)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        let password = entry.get_password().map_err(|e| {
            warn!("Failed to retrieve API key for {}: {}", provider, e);
            anyhow!(
                "API key not found for provider '{}'. Please run 'orbit init' to configure.",
                provider
            )
        })?;

        Ok(password)
    }

    /// Delete an API key from the system keychain
    ///
    /// # Arguments
    /// * `provider` - Provider name to delete
    pub fn delete_api_key(&self, provider: &str) -> Result<()> {
        debug!("Deleting API key for provider: {}", provider);

        let entry = Entry::new(&self.service_name, provider)
            .map_err(|e| anyhow!("Failed to create keyring entry: {}", e))?;

        entry
            .delete_credential()
            .map_err(|e| anyhow!("Failed to delete API key: {}", e))?;

        debug!("API key deleted for provider: {}", provider);
        Ok(())
    }

    /// Check if an API key exists in the keychain
    ///
    /// # Arguments
    /// * `provider` - Provider name to check
    pub fn has_api_key(&self, provider: &str) -> bool {
        let entry = match Entry::new(&self.service_name, provider) {
            Ok(e) => e,
            Err(_) => return false,
        };

        entry.get_password().is_ok()
    }

    /// Migrate API keys from config file to keychain
    ///
    /// This is a helper for one-time migration from plaintext config
    /// to secure keychain storage.
    ///
    /// # Arguments
    /// * `provider` - Provider name
    /// * `api_key` - API key from config file
    ///
    /// # Example
    /// ```no_run
    /// // During config load:
    /// if let Some(api_key) = config.providers.get("claude").and_then(|p| p.api_key.clone()) {
    ///     store.migrate_from_config("claude", &api_key)?;
    ///     warn!("API key migrated to keychain. Remove from config file!");
    /// }
    /// ```
    pub fn migrate_from_config(&self, provider: &str, api_key: &str) -> Result<()> {
        // Only migrate if not already in keychain
        if !self.has_api_key(provider) {
            self.set_api_key(provider, api_key)?;
            warn!(
                "Migrated {} API key to system keychain. Please remove from config file!",
                provider
            );
        }
        Ok(())
    }

    /// Get an API key with fallback to environment variable
    ///
    /// Tries in order:
    /// 1. System keychain
    /// 2. Environment variable (ORBIT_{PROVIDER}_API_KEY)
    /// 3. Returns error if not found
    ///
    /// This provides backward compatibility while encouraging migration to keychain.
    pub fn get_api_key_with_fallback(&self, provider: &str) -> Result<String> {
        // Try keychain first
        if let Ok(key) = self.get_api_key(provider) {
            return Ok(key);
        }

        // Fallback to environment variable
        let env_var = format!("ORBIT_{}_API_KEY", provider.to_uppercase());
        if let Ok(key) = std::env::var(&env_var) {
            warn!(
                "Using API key from environment variable {}. Consider storing in keychain with 'orbit init'",
                env_var
            );
            return Ok(key);
        }

        Err(anyhow!(
            "API key not found for provider '{}'. Please run 'orbit init' to configure.",
            provider
        ))
    }
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Keyring tests require actual system keychain access
    // These tests may fail in CI environments without keychain support

    #[test]
    fn test_credential_store_initialization() {
        let store = CredentialStore::new();
        assert_eq!(store.service_name, "orbit");
    }

    #[test]
    #[ignore] // Requires actual keychain access
    fn test_set_and_get_api_key() {
        let store = CredentialStore::new();
        let test_provider = "test-provider-12345";
        let test_key = "sk-test-key-67890";

        // Set API key
        let result = store.set_api_key(test_provider, test_key);
        assert!(result.is_ok(), "Should store API key successfully");

        // Get API key
        let retrieved = store.get_api_key(test_provider);
        assert!(retrieved.is_ok(), "Should retrieve API key successfully");
        assert_eq!(retrieved.unwrap(), test_key);

        // Cleanup
        let _ = store.delete_api_key(test_provider);
    }

    #[test]
    #[ignore] // Requires actual keychain access
    fn test_delete_api_key() {
        let store = CredentialStore::new();
        let test_provider = "test-delete-12345";
        let test_key = "sk-test-delete-67890";

        // Set and verify
        store.set_api_key(test_provider, test_key).unwrap();
        assert!(store.has_api_key(test_provider));

        // Delete
        let result = store.delete_api_key(test_provider);
        assert!(result.is_ok(), "Should delete API key successfully");

        // Verify deletion
        assert!(!store.has_api_key(test_provider));
    }

    #[test]
    #[ignore] // Requires actual keychain access
    fn test_has_api_key() {
        let store = CredentialStore::new();
        let test_provider = "test-exists-12345";

        // Should not exist initially
        assert!(!store.has_api_key(test_provider));

        // Store key
        store
            .set_api_key(test_provider, "test-key")
            .expect("Failed to set API key");

        // Should exist now
        assert!(store.has_api_key(test_provider));

        // Cleanup
        let _ = store.delete_api_key(test_provider);
    }

    #[test]
    fn test_get_api_key_not_found() {
        let store = CredentialStore::new();
        let result = store.get_api_key("nonexistent-provider-99999");
        assert!(result.is_err(), "Should return error for missing key");
        assert!(
            result.unwrap_err().to_string().contains("not found"),
            "Error should mention key not found"
        );
    }

    #[test]
    fn test_get_api_key_with_fallback_env() {
        let store = CredentialStore::new();
        let test_provider = "envtest";

        // Set environment variable
        std::env::set_var("ORBIT_ENVTEST_API_KEY", "env-key-12345");

        // Should retrieve from environment
        let result = store.get_api_key_with_fallback(test_provider);
        assert!(result.is_ok(), "Should retrieve from environment variable");
        assert_eq!(result.unwrap(), "env-key-12345");

        // Cleanup
        std::env::remove_var("ORBIT_ENVTEST_API_KEY");
    }

    #[test]
    #[ignore] // Requires actual keychain access
    fn test_migrate_from_config() {
        let store = CredentialStore::new();
        let test_provider = "test-migrate-12345";
        let test_key = "sk-migrate-67890";

        // Ensure clean state
        let _ = store.delete_api_key(test_provider);

        // Migrate
        let result = store.migrate_from_config(test_provider, test_key);
        assert!(result.is_ok(), "Should migrate successfully");

        // Verify migration
        assert!(store.has_api_key(test_provider));
        let retrieved = store.get_api_key(test_provider).unwrap();
        assert_eq!(retrieved, test_key);

        // Try migrating again (should be idempotent)
        let result2 = store.migrate_from_config(test_provider, "different-key");
        assert!(result2.is_ok(), "Should handle duplicate migration");

        // Should still have original key
        let retrieved2 = store.get_api_key(test_provider).unwrap();
        assert_eq!(retrieved2, test_key, "Should not overwrite existing key");

        // Cleanup
        let _ = store.delete_api_key(test_provider);
    }
}
