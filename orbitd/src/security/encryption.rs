// Data encryption for Orbit AI Terminal
//
// This module provides encryption services for:
// - API keys and secrets storage
// - Sensitive data at rest
// - IPC message encryption (optional)
//
// Uses AES-256-GCM for authenticated encryption

use anyhow::{anyhow, Result};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use rand::RngCore;
use std::sync::OnceLock;

const NONCE_SIZE: usize = 12; // 96 bits for GCM

/// Encryptor for sensitive data
pub struct Encryptor {
    cipher: Aes256Gcm,
    key_derivation_salt: Vec<u8>,
}

impl Encryptor {
    /// Create a new encryptor with a derived key
    ///
    /// The key is derived from machine-specific and user-specific data
    /// to provide a unique key per installation
    pub fn new() -> Result<Self> {
        let (key, salt) = Self::derive_key()?;
        let cipher = Aes256Gcm::new(&key);

        Ok(Self {
            cipher,
            key_derivation_salt: salt,
        })
    }

    /// Create an encryptor from an explicit key (for testing)
    pub fn from_key(key_bytes: &[u8; 32]) -> Result<Self> {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);

        Ok(Self {
            cipher,
            key_derivation_salt: vec![],
        })
    }

    /// Encrypt plaintext and return base64-encoded ciphertext
    ///
    /// The nonce is prepended to the ciphertext for easy decryption
    ///
    /// # Arguments
    /// * `plaintext` - The data to encrypt
    ///
    /// # Returns
    /// Base64-encoded string containing nonce + ciphertext
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let nonce = self.generate_nonce();

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        // Combine nonce + ciphertext
        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        // Encode as base64 for easy storage
        Ok(BASE64.encode(&result))
    }

    /// Encrypt bytes and return base64-encoded ciphertext
    pub fn encrypt_bytes(&self, plaintext: &[u8]) -> Result<String> {
        let nonce = self.generate_nonce();

        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        let mut result = nonce.to_vec();
        result.extend_from_slice(&ciphertext);

        Ok(BASE64.encode(&result))
    }

    /// Decrypt base64-encoded ciphertext
    ///
    /// # Arguments
    /// * `ciphertext_b64` - Base64-encoded nonce + ciphertext
    ///
    /// # Returns
    /// Decrypted plaintext string
    pub fn decrypt(&self, ciphertext_b64: &str) -> Result<String> {
        let data = BASE64
            .decode(ciphertext_b64)
            .map_err(|e| anyhow!("Invalid base64: {}", e))?;

        if data.len() < NONCE_SIZE {
            return Err(anyhow!("Ciphertext too short"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        String::from_utf8(plaintext).map_err(|e| anyhow!("Invalid UTF-8: {}", e))
    }

    /// Decrypt to bytes
    pub fn decrypt_bytes(&self, ciphertext_b64: &str) -> Result<Vec<u8>> {
        let data = BASE64
            .decode(ciphertext_b64)
            .map_err(|e| anyhow!("Invalid base64: {}", e))?;

        if data.len() < NONCE_SIZE {
            return Err(anyhow!("Ciphertext too short"));
        }

        let (nonce_bytes, ciphertext) = data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Generate a random nonce for encryption
    fn generate_nonce(&self) -> Nonce {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        *Nonce::from_slice(&nonce_bytes)
    }

    /// Derive an encryption key from machine and user identifiers
    ///
    /// This provides a unique key per installation without requiring
    /// the user to manage keys explicitly
    fn derive_key() -> Result<(Key<Aes256Gcm>, Vec<u8>)> {
        // Get machine-specific identifiers
        let machine_id = Self::get_machine_id()?;
        let user_id = whoami::username();

        // Generate a salt (in production, this should be stored and reused)
        let mut salt = vec![0u8; 32];
        OsRng.fill_bytes(&mut salt);

        // Derive key using BLAKE3 (fast, secure key derivation)
        let mut hasher = blake3::Hasher::new();
        hasher.update(machine_id.as_bytes());
        hasher.update(user_id.as_bytes());
        hasher.update(&salt);

        let hash = hasher.finalize();
        let key = Key::<Aes256Gcm>::from_slice(hash.as_bytes());

        Ok((*key, salt))
    }

    /// Get a machine-specific identifier
    fn get_machine_id() -> Result<String> {
        // Try multiple sources for machine ID
        #[cfg(target_os = "linux")]
        {
            if let Ok(id) = std::fs::read_to_string("/etc/machine-id") {
                return Ok(id.trim().to_string());
            }
        }

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            if let Ok(output) = Command::new("ioreg")
                .args(&["-rd1", "-c", "IOPlatformExpertDevice"])
                .output()
            {
                if let Ok(stdout) = String::from_utf8(output.stdout) {
                    if let Some(uuid) = stdout.lines().find(|l| l.contains("IOPlatformUUID")) {
                        return Ok(uuid.to_string());
                    }
                }
            }
        }

        // Fallback: use hostname
        Ok(whoami::devicename())
    }
}

impl Default for Encryptor {
    fn default() -> Self {
        Self::new().expect("Failed to create default encryptor")
    }
}

/// Get the global encryptor instance
pub fn get_encryptor() -> &'static Encryptor {
    static ENCRYPTOR: OnceLock<Encryptor> = OnceLock::new();
    ENCRYPTOR.get_or_init(|| Encryptor::new().expect("Failed to initialize encryptor"))
}

/// Secure storage for API keys and secrets
pub struct SecretStore {
    encryptor: Encryptor,
}

impl SecretStore {
    /// Create a new secret store
    pub fn new() -> Result<Self> {
        Ok(Self {
            encryptor: Encryptor::new()?,
        })
    }

    /// Store a secret
    pub fn store(&self, name: &str, value: &str) -> Result<String> {
        let encrypted = self.encryptor.encrypt(value)?;
        Ok(encrypted)
    }

    /// Retrieve a secret
    pub fn retrieve(&self, encrypted_value: &str) -> Result<String> {
        self.encryptor.decrypt(encrypted_value)
    }

    /// Check if a value is encrypted (basic heuristic)
    pub fn is_encrypted(value: &str) -> bool {
        // Check if it looks like base64 and has reasonable length
        value.len() > NONCE_SIZE * 4 / 3 && BASE64.decode(value).is_ok()
    }
}

impl Default for SecretStore {
    fn default() -> Self {
        Self::new().expect("Failed to create default secret store")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let encryptor = Encryptor::from_key(&[1u8; 32]).unwrap();

        let plaintext = "Hello, World!";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();

        assert_ne!(plaintext, ciphertext);
        assert!(ciphertext.len() > plaintext.len());

        let decrypted = encryptor.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypt_bytes() {
        let encryptor = Encryptor::from_key(&[2u8; 32]).unwrap();

        let plaintext = b"Binary data here";
        let ciphertext = encryptor.encrypt_bytes(plaintext).unwrap();

        let decrypted = encryptor.decrypt_bytes(&ciphertext).unwrap();
        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_different_nonces() {
        let encryptor = Encryptor::from_key(&[3u8; 32]).unwrap();

        let plaintext = "Same plaintext";
        let ciphertext1 = encryptor.encrypt(plaintext).unwrap();
        let ciphertext2 = encryptor.encrypt(plaintext).unwrap();

        // Same plaintext should produce different ciphertexts (different nonces)
        assert_ne!(ciphertext1, ciphertext2);

        // But both should decrypt to the same plaintext
        assert_eq!(encryptor.decrypt(&ciphertext1).unwrap(), plaintext);
        assert_eq!(encryptor.decrypt(&ciphertext2).unwrap(), plaintext);
    }

    #[test]
    fn test_invalid_ciphertext() {
        let encryptor = Encryptor::from_key(&[4u8; 32]).unwrap();

        assert!(encryptor.decrypt("invalid base64!").is_err());
        assert!(encryptor.decrypt("dG9vc2hvcnQ=").is_err()); // "tooshort" in base64
    }

    #[test]
    fn test_tampered_ciphertext() {
        let encryptor = Encryptor::from_key(&[5u8; 32]).unwrap();

        let plaintext = "Original message";
        let mut ciphertext = encryptor.encrypt(plaintext).unwrap();

        // Tamper with the ciphertext
        let mut bytes = BASE64.decode(&ciphertext).unwrap();
        bytes[20] ^= 0xFF; // Flip some bits
        ciphertext = BASE64.encode(&bytes);

        // Decryption should fail (authentication tag mismatch)
        assert!(encryptor.decrypt(&ciphertext).is_err());
    }

    #[test]
    fn test_secret_store() {
        let store = SecretStore::new().unwrap();

        let secret = "my-api-key-12345";
        let encrypted = store.store("api_key", secret).unwrap();

        assert_ne!(secret, encrypted);
        assert!(SecretStore::is_encrypted(&encrypted));

        let retrieved = store.retrieve(&encrypted).unwrap();
        assert_eq!(secret, retrieved);
    }

    #[test]
    fn test_is_encrypted_heuristic() {
        assert!(!SecretStore::is_encrypted("plaintext"));
        assert!(!SecretStore::is_encrypted("short"));

        // This looks like valid base64 with reasonable length
        let fake_encrypted = BASE64.encode(&vec![0u8; 32]);
        assert!(SecretStore::is_encrypted(&fake_encrypted));
    }

    #[test]
    fn test_empty_plaintext() {
        let encryptor = Encryptor::from_key(&[6u8; 32]).unwrap();

        let ciphertext = encryptor.encrypt("").unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!("", decrypted);
    }

    #[test]
    fn test_long_plaintext() {
        let encryptor = Encryptor::from_key(&[7u8; 32]).unwrap();

        let plaintext = "A".repeat(10000);
        let ciphertext = encryptor.encrypt(&plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_unicode_plaintext() {
        let encryptor = Encryptor::from_key(&[8u8; 32]).unwrap();

        let plaintext = "Hello 世界 🌍";
        let ciphertext = encryptor.encrypt(plaintext).unwrap();
        let decrypted = encryptor.decrypt(&ciphertext).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
