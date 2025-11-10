use anyhow::{anyhow, Context, Result};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use base64::{Engine as _, engine::general_purpose};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng as AeadOsRng},
    ChaCha20Poly1305, Nonce,
};
use rand::RngCore;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Salt length in bytes (16 bytes = 128 bits)
const SALT_LEN: usize = 16;

/// Nonce length for ChaCha20-Poly1305 (12 bytes = 96 bits)
const NONCE_LEN: usize = 12;

/// Master key derived from password
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct MasterKey {
    key: [u8; 32],
}

impl MasterKey {
    /// Derive a master key from a password using Argon2id
    pub fn derive_from_password(password: &str, salt: &[u8]) -> Result<Self> {
        if salt.len() != SALT_LEN {
            return Err(anyhow!("Invalid salt length: expected {}, got {}", SALT_LEN, salt.len()));
        }

        // Create Argon2id instance with recommended parameters
        // Time cost: 2 iterations (default)
        // Memory cost: 19456 KiB (~19 MB)
        // Parallelism: 1 thread
        let argon2 = Argon2::default();

        // Encode salt as SaltString
        let salt_string = SaltString::encode_b64(salt)
            .map_err(|e| anyhow!("Failed to encode salt: {}", e))?;

        // Hash password
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| anyhow!("Failed to hash password: {}", e))?;

        // Extract 32-byte key from hash
        let hash_bytes = password_hash.hash
            .ok_or_else(|| anyhow!("Password hash missing hash output"))?;

        let mut key = [0u8; 32];
        let hash_slice = hash_bytes.as_bytes();
        if hash_slice.len() < 32 {
            return Err(anyhow!("Password hash too short: {} bytes", hash_slice.len()));
        }
        key.copy_from_slice(&hash_slice[..32]);

        Ok(Self { key })
    }

    /// Verify a password against a stored hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let argon2 = Argon2::default();
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| anyhow!("Failed to parse password hash: {}", e))?;

        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Generate a random salt
    pub fn generate_salt() -> [u8; SALT_LEN] {
        let mut salt = [0u8; SALT_LEN];
        OsRng.fill_bytes(&mut salt);
        salt
    }

    /// Get the key bytes (use with caution)
    fn key_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

/// Encrypted data with nonce
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// Random nonce used for encryption
    pub nonce: [u8; NONCE_LEN],
    /// Encrypted ciphertext (includes authentication tag)
    pub ciphertext: Vec<u8>,
}

impl EncryptedData {
    /// Encode to base64 for storage (format: base64(nonce || ciphertext))
    pub fn to_base64(&self) -> String {
        let mut combined = Vec::with_capacity(NONCE_LEN + self.ciphertext.len());
        combined.extend_from_slice(&self.nonce);
        combined.extend_from_slice(&self.ciphertext);
        general_purpose::STANDARD.encode(&combined)
    }

    /// Decode from base64
    pub fn from_base64(encoded: &str) -> Result<Self> {
        let decoded = general_purpose::STANDARD.decode(encoded)
            .context("Failed to decode base64")?;

        if decoded.len() < NONCE_LEN {
            return Err(anyhow!("Encrypted data too short"));
        }

        let mut nonce = [0u8; NONCE_LEN];
        nonce.copy_from_slice(&decoded[..NONCE_LEN]);

        let ciphertext = decoded[NONCE_LEN..].to_vec();

        Ok(Self { nonce, ciphertext })
    }
}

/// Vault crypto operations
pub struct VaultCrypto;

impl VaultCrypto {
    /// Encrypt plaintext using ChaCha20-Poly1305
    pub fn encrypt(master_key: &MasterKey, plaintext: &[u8]) -> Result<EncryptedData> {
        // Create cipher instance
        let cipher = ChaCha20Poly1305::new(master_key.key_bytes().into());

        // Generate random nonce
        let mut nonce_bytes = [0u8; NONCE_LEN];
        AeadOsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;

        Ok(EncryptedData {
            nonce: nonce_bytes,
            ciphertext,
        })
    }

    /// Decrypt ciphertext using ChaCha20-Poly1305
    pub fn decrypt(master_key: &MasterKey, encrypted: &EncryptedData) -> Result<Vec<u8>> {
        // Create cipher instance
        let cipher = ChaCha20Poly1305::new(master_key.key_bytes().into());

        // Create nonce
        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| anyhow!("Decryption failed: {}", e))?;

        Ok(plaintext)
    }

    /// Encrypt a string and return base64-encoded result
    pub fn encrypt_string(master_key: &MasterKey, plaintext: &str) -> Result<String> {
        let encrypted = Self::encrypt(master_key, plaintext.as_bytes())?;
        Ok(encrypted.to_base64())
    }

    /// Decrypt a base64-encoded string
    pub fn decrypt_string(master_key: &MasterKey, encrypted_b64: &str) -> Result<String> {
        let encrypted = EncryptedData::from_base64(encrypted_b64)?;
        let plaintext_bytes = Self::decrypt(master_key, &encrypted)?;
        String::from_utf8(plaintext_bytes)
            .context("Decrypted data is not valid UTF-8")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_key_derivation() {
        let password = "test_password_123";
        let salt = MasterKey::generate_salt();

        // Derive key
        let key1 = MasterKey::derive_from_password(password, &salt).unwrap();
        let key2 = MasterKey::derive_from_password(password, &salt).unwrap();

        // Same password and salt should produce same key
        assert_eq!(key1.key_bytes(), key2.key_bytes());

        // Different salt should produce different key
        let salt2 = MasterKey::generate_salt();
        let key3 = MasterKey::derive_from_password(password, &salt2).unwrap();
        assert_ne!(key1.key_bytes(), key3.key_bytes());
    }

    #[test]
    fn test_encryption_decryption() {
        let password = "test_password_123";
        let salt = MasterKey::generate_salt();
        let master_key = MasterKey::derive_from_password(password, &salt).unwrap();

        let plaintext = "This is a secret message!";

        // Encrypt
        let encrypted = VaultCrypto::encrypt(&master_key, plaintext.as_bytes()).unwrap();

        // Decrypt
        let decrypted = VaultCrypto::decrypt(&master_key, &encrypted).unwrap();
        let decrypted_string = String::from_utf8(decrypted).unwrap();

        assert_eq!(plaintext, decrypted_string);
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let password = "test_password_123";
        let salt = MasterKey::generate_salt();
        let master_key = MasterKey::derive_from_password(password, &salt).unwrap();

        let plaintext = "Secret SSH key content";

        // Encrypt to base64
        let encrypted_b64 = VaultCrypto::encrypt_string(&master_key, plaintext).unwrap();

        // Decrypt from base64
        let decrypted = VaultCrypto::decrypt_string(&master_key, &encrypted_b64).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypted_data_encoding() {
        let encrypted = EncryptedData {
            nonce: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
            ciphertext: vec![13, 14, 15, 16, 17],
        };

        // Encode to base64
        let encoded = encrypted.to_base64();

        // Decode from base64
        let decoded = EncryptedData::from_base64(&encoded).unwrap();

        assert_eq!(encrypted.nonce, decoded.nonce);
        assert_eq!(encrypted.ciphertext, decoded.ciphertext);
    }

    #[test]
    fn test_different_nonces_produce_different_ciphertext() {
        let password = "test_password_123";
        let salt = MasterKey::generate_salt();
        let master_key = MasterKey::derive_from_password(password, &salt).unwrap();

        let plaintext = "Same message";

        // Encrypt twice
        let encrypted1 = VaultCrypto::encrypt(&master_key, plaintext.as_bytes()).unwrap();
        let encrypted2 = VaultCrypto::encrypt(&master_key, plaintext.as_bytes()).unwrap();

        // Different nonces should produce different ciphertext
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // But both should decrypt to same plaintext
        let decrypted1 = VaultCrypto::decrypt(&master_key, &encrypted1).unwrap();
        let decrypted2 = VaultCrypto::decrypt(&master_key, &encrypted2).unwrap();
        assert_eq!(decrypted1, decrypted2);
    }

    #[test]
    fn test_wrong_key_fails_decryption() {
        let password1 = "password1";
        let password2 = "password2";
        let salt = MasterKey::generate_salt();

        let key1 = MasterKey::derive_from_password(password1, &salt).unwrap();
        let key2 = MasterKey::derive_from_password(password2, &salt).unwrap();

        let plaintext = "Secret message";

        // Encrypt with key1
        let encrypted = VaultCrypto::encrypt(&key1, plaintext.as_bytes()).unwrap();

        // Try to decrypt with key2 (should fail)
        let result = VaultCrypto::decrypt(&key2, &encrypted);
        assert!(result.is_err());
    }
}
