use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::config::Config;

#[derive(Clone)]
pub struct LicenseManager {
    config_license_key: Option<String>,
    cache_path: PathBuf,
    server_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedLicense {
    key: String,
    company: String,
    user: String,
    verified_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    features: Vec<String>,
}

impl LicenseManager {
    pub fn new(config: &Config) -> Result<Self> {
        let cache_path = Config::data_dir()?.join("license.enc");
        let server_url = std::env::var("ORBIT_LICENSE_SERVER")
            .unwrap_or_else(|_| "https://license.singulio.com".to_string());

        // SECURITY: Enforce HTTPS for license server
        // License keys are sensitive credentials and must be transmitted securely
        if !server_url.starts_with("https://") {
            return Err(anyhow!(
                "License server must use HTTPS. Got: {}. \
                Set ORBIT_LICENSE_SERVER=https://your-server.com or use the default.",
                server_url
            ));
        }

        Ok(Self {
            config_license_key: config.license.key.clone(),
            cache_path,
            server_url,
        })
    }

    pub async fn validate(&self) -> Result<()> {
        // Check if license exists
        let license_key = self
            .config_license_key
            .as_ref()
            .ok_or_else(|| anyhow!("No license key configured"))?;

        // Try to load cached license
        if self.cache_path.exists() {
            if let Ok(cached) = self.load_cached_license() {
                if self.is_license_valid(&cached) {
                    info!("License valid from cache");
                    return Ok(());
                }
            }
        }

        // License expired or missing - verify with server
        warn!("License expired or missing, verifying with server...");

        let verified = self.verify_with_server(license_key).await?;

        if verified {
            self.cache_license(license_key)?;
            Ok(())
        } else {
            Err(anyhow!("License is invalid or expired"))
        }
    }

    #[allow(dead_code)]
    pub fn last_verified(&self) -> String {
        if let Ok(cached) = self.load_cached_license() {
            let now = Utc::now();
            let diff = now - cached.verified_at;

            if diff.num_hours() > 0 {
                format!("{} hours ago", diff.num_hours())
            } else {
                format!("{} minutes ago", diff.num_minutes())
            }
        } else {
            "Never".to_string()
        }
    }

    fn is_license_valid(&self, license: &CachedLicense) -> bool {
        let now = Utc::now();
        let age = now - license.verified_at;

        // Must be verified within configured interval
        // Reduced from 48h to 8h for security (faster license revocation)
        let max_age = Duration::hours(8);

        if age > max_age {
            warn!(
                verified_hours_ago = age.num_hours(),
                "License verification too old"
            );
            return false;
        }

        // Check if expired
        if now > license.expires_at {
            warn!("License expired");
            return false;
        }

        true
    }

    async fn verify_with_server(&self, license_key: &str) -> Result<bool> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let machine_id = self.get_machine_id();

        let response = client
            .post(format!("{}/api/v1/licenses/verify", self.server_url))
            .json(&serde_json::json!({
                "license_key": license_key,
                "product": "orbit",
                "version": env!("CARGO_PKG_VERSION"),
                "machine_id": machine_id,
            }))
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to contact license server: {}", e);
                anyhow!("License server unreachable - cannot verify license")
            })?;

        Ok(response.status().is_success())
    }

    fn cache_license(&self, license_key: &str) -> Result<()> {
        let license = CachedLicense {
            key: license_key.to_string(),
            company: "Development".to_string(),
            user: "dev@localhost".to_string(),
            verified_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(365),
            features: vec!["all".to_string()],
        };

        let data = serde_json::to_vec(&license)?;

        // Simple encryption with machine-specific key
        let encrypted = self.encrypt_license_data(&data)?;

        std::fs::write(&self.cache_path, encrypted)?;

        // Set restrictive permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&self.cache_path)?.permissions();
            perms.set_mode(0o600);
            std::fs::set_permissions(&self.cache_path, perms)?;
        }

        Ok(())
    }

    fn load_cached_license(&self) -> Result<CachedLicense> {
        let encrypted = std::fs::read(&self.cache_path)?;
        let decrypted = self.decrypt_license_data(&encrypted)?;
        let license: CachedLicense = serde_json::from_slice(&decrypted)?;
        Ok(license)
    }

    fn encrypt_license_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Use AES-256-GCM for authenticated encryption
        // Derive 256-bit key from machine ID using SHA-256
        let machine_id = self.get_machine_id();
        let mut hasher = Sha256::new();
        hasher.update(machine_id.as_bytes());
        let key_bytes = hasher.finalize();

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        // Generate random nonce (96 bits / 12 bytes for AES-GCM)
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(&nonce, data)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Prepend nonce to ciphertext (nonce doesn't need to be secret)
        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    fn decrypt_license_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Data format: [nonce (12 bytes)][ciphertext + auth tag]
        if data.len() < 12 {
            return Err(anyhow!("Invalid encrypted data: too short"));
        }

        // Derive same 256-bit key from machine ID
        let machine_id = self.get_machine_id();
        let mut hasher = Sha256::new();
        hasher.update(machine_id.as_bytes());
        let key_bytes = hasher.finalize();

        // Create cipher instance
        let cipher = Aes256Gcm::new_from_slice(&key_bytes)
            .map_err(|e| anyhow!("Failed to create cipher: {}", e))?;

        // Extract nonce and ciphertext
        let (nonce_bytes, ciphertext) = data.split_at(12);
        #[allow(deprecated)]
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt and verify authentication tag
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed (corrupted or tampered data): {}", e))?;

        Ok(plaintext)
    }

    fn get_machine_id(&self) -> String {
        machine_uid::get().unwrap_or_else(|_| "unknown-machine".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, LicenseConfig};
    use tempfile::TempDir;

    fn create_test_config_with_license(license_key: Option<String>) -> Config {
        Config {
            license: LicenseConfig {
                key: license_key,
                company: Some("Test Company".to_string()),
                user: Some("test@example.com".to_string()),
                validation_interval_hours: 48,
            },
            daemon: crate::config::DaemonConfig {
                socket_path: "/tmp/orbit-test.sock".into(),
                log_level: "info".to_string(),
                auto_restart: true,
            },
            provider_mode: crate::config::ProviderMode::Manual,
            default_provider: "test".to_string(),
            providers: std::collections::HashMap::new(),
            auto_routing: None,
            learning: crate::config::LearningConfig {
                enabled: true,
                confidence_threshold: 0.7,
                max_patterns: 10000,
                embedding_model: "minilm-l6-v2".to_string(),
            },
            monitoring: crate::config::MonitoringConfig {
                enabled: true,
                interval_seconds: 300,
                watch_git_repos: true,
                watch_system: true,
                desktop_notifications: false,
            },
            classification: crate::config::ClassificationConfig {
                natural_language_threshold: 0.8,
                check_path_binaries: true,
                cache_known_commands: true,
            },
            execution: crate::config::ExecutionConfig {
                auto_approve: false,
                confirm_destructive: true,
                timeout_seconds: 300,
            },
            context: crate::config::ContextConfig {
                track_directory_patterns: true,
                detect_languages: true,
                detect_frameworks: true,
                include_git_context: true,
                max_recent_commands: 20,
            },
            ui: crate::config::UiConfig {
                emoji: true,
                colors: true,
                show_provider: false,
                show_learning_stats: true,
            },
        }
    }

    #[test]
    fn test_license_manager_initialization() {
        let config = create_test_config_with_license(Some("test-key-12345".to_string()));
        let manager = LicenseManager::new(&config);
        assert!(
            manager.is_ok(),
            "License manager should initialize successfully"
        );

        let manager = manager.unwrap();
        assert_eq!(
            manager.config_license_key,
            Some("test-key-12345".to_string())
        );
    }

    #[test]
    fn test_license_manager_no_key() {
        let config = create_test_config_with_license(None);
        let manager = LicenseManager::new(&config);
        assert!(
            manager.is_ok(),
            "License manager should initialize even without key"
        );

        let manager = manager.unwrap();
        assert_eq!(manager.config_license_key, None);
    }

    #[tokio::test]
    async fn test_validate_without_license_key() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(None);
        let manager = LicenseManager::new(&config).unwrap();

        let result = manager.validate().await;
        assert!(result.is_err(), "Should fail without license key");
        assert!(
            result.unwrap_err().to_string().contains("No license key"),
            "Error should mention missing license key"
        );
    }

    #[test]
    fn test_encryption_decryption_roundtrip() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let original_data = b"test license data with special chars: !@#$%^&*()";
        let encrypted = manager.encrypt_license_data(original_data).unwrap();
        let decrypted = manager.decrypt_license_data(&encrypted).unwrap();

        assert_eq!(
            original_data.to_vec(),
            decrypted,
            "Decrypted data should match original"
        );
    }

    #[test]
    fn test_encryption_produces_different_output() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let original_data = b"sensitive license information";
        let encrypted = manager.encrypt_license_data(original_data).unwrap();

        assert_ne!(
            original_data.to_vec(),
            encrypted,
            "Encrypted data should differ from original"
        );
    }

    #[test]
    fn test_encryption_is_non_deterministic() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let data = b"test data";
        let encrypted1 = manager.encrypt_license_data(data).unwrap();
        let encrypted2 = manager.encrypt_license_data(data).unwrap();

        // AES-GCM uses random nonces, so encryptions should differ
        // This is a security feature to prevent certain attacks
        assert_ne!(
            encrypted1, encrypted2,
            "AES-GCM should produce different ciphertexts each time (random nonce)"
        );

        // But both should decrypt to the same original data
        let decrypted1 = manager.decrypt_license_data(&encrypted1).unwrap();
        let decrypted2 = manager.decrypt_license_data(&encrypted2).unwrap();
        assert_eq!(decrypted1, decrypted2);
        assert_eq!(decrypted1, data.to_vec());
    }

    #[test]
    fn test_encryption_tamper_detection() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let original_data = b"important license data";
        let mut encrypted = manager.encrypt_license_data(original_data).unwrap();

        // Tamper with the ciphertext (flip a bit)
        if encrypted.len() > 15 {
            encrypted[15] ^= 0x01;
        }

        // Decryption should fail due to authentication tag mismatch
        let result = manager.decrypt_license_data(&encrypted);
        assert!(
            result.is_err(),
            "Tampered data should fail decryption (authentication)"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("corrupted or tampered"),
            "Error should indicate tampering"
        );
    }

    #[test]
    fn test_encryption_minimum_size() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let data = b"x";
        let encrypted = manager.encrypt_license_data(data).unwrap();

        // Encrypted data should include: nonce (12 bytes) + ciphertext (at least 1 byte) + auth tag (16 bytes)
        assert!(
            encrypted.len() >= 12 + 1 + 16,
            "Encrypted data should include nonce, ciphertext, and auth tag"
        );
    }

    #[test]
    fn test_decryption_invalid_data_too_short() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        // Data shorter than nonce size (12 bytes)
        let invalid_data = vec![1, 2, 3, 4, 5];
        let result = manager.decrypt_license_data(&invalid_data);

        assert!(result.is_err(), "Should fail on too-short data");
        assert!(
            result.unwrap_err().to_string().contains("too short"),
            "Error should mention data is too short"
        );
    }

    #[test]
    fn test_is_license_valid_fresh() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let license = CachedLicense {
            key: "test-key".to_string(),
            company: "Test Corp".to_string(),
            user: "test@example.com".to_string(),
            verified_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(365),
            features: vec!["all".to_string()],
        };

        assert!(
            manager.is_license_valid(&license),
            "Fresh license should be valid"
        );
    }

    #[test]
    fn test_is_license_valid_expired() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let license = CachedLicense {
            key: "test-key".to_string(),
            company: "Test Corp".to_string(),
            user: "test@example.com".to_string(),
            verified_at: Utc::now(),
            expires_at: Utc::now() - Duration::days(1), // Expired yesterday
            features: vec!["all".to_string()],
        };

        assert!(
            !manager.is_license_valid(&license),
            "Expired license should be invalid"
        );
    }

    #[test]
    fn test_is_license_valid_old_verification() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let license = CachedLicense {
            key: "test-key".to_string(),
            company: "Test Corp".to_string(),
            user: "test@example.com".to_string(),
            verified_at: Utc::now() - Duration::hours(49), // > 48 hours ago
            expires_at: Utc::now() + Duration::days(365),
            features: vec!["all".to_string()],
        };

        assert!(
            !manager.is_license_valid(&license),
            "License with old verification should be invalid"
        );
    }

    #[test]
    fn test_is_license_valid_boundary_8_hours() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        // Well within 8 hours (should be valid)
        let license_valid = CachedLicense {
            key: "test-key".to_string(),
            company: "Test Corp".to_string(),
            user: "test@example.com".to_string(),
            verified_at: Utc::now() - Duration::hours(7),
            expires_at: Utc::now() + Duration::days(365),
            features: vec!["all".to_string()],
        };

        assert!(
            manager.is_license_valid(&license_valid),
            "License at 7 hours should be valid"
        );

        // Just over 8 hours (should be invalid)
        let license_invalid = CachedLicense {
            key: "test-key".to_string(),
            company: "Test Corp".to_string(),
            user: "test@example.com".to_string(),
            verified_at: Utc::now() - Duration::hours(9),
            expires_at: Utc::now() + Duration::days(365),
            features: vec!["all".to_string()],
        };

        assert!(
            !manager.is_license_valid(&license_invalid),
            "License at 9 hours should be invalid"
        );
    }

    #[test]
    fn test_cache_license_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let result = manager.cache_license("test-license-key-12345");
        assert!(result.is_ok(), "Cache license should succeed");

        assert!(manager.cache_path.exists(), "Cache file should be created");
    }

    #[test]
    fn test_load_cached_license_roundtrip() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        // Cache a license
        manager
            .cache_license("test-license-key-12345")
            .expect("Failed to cache license");

        // Load it back
        let loaded = manager.load_cached_license();
        assert!(loaded.is_ok(), "Should load cached license successfully");

        let loaded = loaded.unwrap();
        assert_eq!(loaded.key, "test-license-key-12345");
        assert_eq!(loaded.company, "Development");
        assert_eq!(loaded.user, "dev@localhost");
    }

    #[test]
    fn test_load_cached_license_corrupted() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        // Write corrupted data to cache file
        std::fs::write(&manager.cache_path, b"corrupted data that is not valid")
            .expect("Failed to write corrupted cache");

        let result = manager.load_cached_license();
        assert!(
            result.is_err(),
            "Loading corrupted cache should fail gracefully"
        );
    }

    #[test]
    fn test_load_cached_license_missing() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let result = manager.load_cached_license();
        assert!(result.is_err(), "Loading missing cache should return error");
    }

    #[test]
    #[cfg(unix)]
    fn test_cache_file_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        manager
            .cache_license("test-key")
            .expect("Failed to cache license");

        let metadata = std::fs::metadata(&manager.cache_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();

        // Should be 0o600 (read/write for owner only)
        assert_eq!(
            mode & 0o777,
            0o600,
            "Cache file should have restrictive permissions (0o600)"
        );
    }

    #[test]
    fn test_get_machine_id_consistency() {
        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager1 = LicenseManager::new(&config).unwrap();
        let manager2 = LicenseManager::new(&config).unwrap();

        let id1 = manager1.get_machine_id();
        let id2 = manager2.get_machine_id();

        assert_eq!(id1, id2, "Machine ID should be consistent across instances");
        assert!(!id1.is_empty(), "Machine ID should not be empty");
    }

    #[test]
    fn test_last_verified_never() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        let last = manager.last_verified();
        assert_eq!(last, "Never", "Should report 'Never' when no cache exists");
    }

    #[test]
    fn test_last_verified_recent() {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config = create_test_config_with_license(Some("test-key".to_string()));
        let manager = LicenseManager::new(&config).unwrap();

        manager
            .cache_license("test-key")
            .expect("Failed to cache license");

        let last = manager.last_verified();
        assert!(
            last.contains("minutes ago") || last.contains("0 hours ago"),
            "Should report recent verification time: {}",
            last
        );
    }

    #[test]
    fn test_cached_license_serialization() {
        let license = CachedLicense {
            key: "test-key-12345".to_string(),
            company: "Test Company".to_string(),
            user: "user@test.com".to_string(),
            verified_at: Utc::now(),
            expires_at: Utc::now() + Duration::days(365),
            features: vec!["feature1".to_string(), "feature2".to_string()],
        };

        let json = serde_json::to_string(&license);
        assert!(json.is_ok(), "Should serialize to JSON");

        let json_str = json.unwrap();
        let deserialized: Result<CachedLicense, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.key, "test-key-12345");
        assert_eq!(deserialized.company, "Test Company");
        assert_eq!(deserialized.features.len(), 2);
    }
}
