//! Encryption and decryption primitives

use anyhow::Result;
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};

pub struct EncryptionKey {
    cipher: ChaCha20Poly1305,
}

impl EncryptionKey {
    pub fn generate() -> Self {
        let key = ChaCha20Poly1305::generate_key(&mut OsRng);
        let cipher = ChaCha20Poly1305::new(&key);
        Self { cipher }
    }

    pub fn from_bytes(key: &[u8; 32]) -> Self {
        let cipher = ChaCha20Poly1305::new(key.into());
        Self { cipher }
    }
}

pub fn encrypt_chunk(key: &EncryptionKey, chunk: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
    let nonce = Nonce::from(*nonce_bytes);
    key.cipher
        .encrypt(&nonce, chunk)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))
}

pub fn decrypt_chunk(key: &EncryptionKey, ciphertext: &[u8], nonce_bytes: &[u8; 12]) -> Result<Vec<u8>> {
    let nonce = Nonce::from(*nonce_bytes);
    key.cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|e| anyhow::anyhow!("Decryption failed: {}", e))
}
