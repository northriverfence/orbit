// BLAKE3 Hash Validation for File Transfers

use blake3::Hasher;

/// Hash validator using BLAKE3
#[derive(Debug)]
pub struct HashValidator {
    hasher: Hasher,
}

impl HashValidator {
    /// Create a new hash validator
    pub fn new() -> Self {
        Self {
            hasher: Hasher::new(),
        }
    }

    /// Update hasher with new data
    pub fn update(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Finalize and get the hash as hex string
    pub fn finalize_hex(&self) -> String {
        self.hasher.finalize().to_hex().to_string()
    }

    /// Finalize and get the hash as bytes
    pub fn finalize(&self) -> [u8; 32] {
        *self.hasher.finalize().as_bytes()
    }

    /// Reset the hasher
    pub fn reset(&mut self) {
        self.hasher = Hasher::new();
    }
}

impl Default for HashValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute BLAKE3 hash of data and return as hex string
pub fn hash_data(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Compute BLAKE3 hash of data and return as bytes
pub fn hash_data_bytes(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Verify that computed hash matches expected hash
pub fn verify_hash(data: &[u8], expected_hash: &str) -> bool {
    let computed = hash_data(data);
    computed.eq_ignore_ascii_case(expected_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_data() {
        let data = b"Hello, world!";
        let hash = hash_data(data);
        assert_eq!(hash.len(), 64); // BLAKE3 produces 256-bit (32 byte) hash = 64 hex chars
    }

    #[test]
    fn test_verify_hash() {
        let data = b"Hello, world!";
        let hash = hash_data(data);
        assert!(verify_hash(data, &hash));
        assert!(!verify_hash(data, "0000000000000000000000000000000000000000000000000000000000000000"));
    }

    #[test]
    fn test_hash_validator() {
        let mut validator = HashValidator::new();
        validator.update(b"Hello, ");
        validator.update(b"world!");
        let hash = validator.finalize_hex();

        let expected = hash_data(b"Hello, world!");
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"Test data";

        // Hash using hash_data
        let hash1 = hash_data(data);

        // Hash using HashValidator
        let mut validator = HashValidator::new();
        validator.update(data);
        let hash2 = validator.finalize_hex();

        assert_eq!(hash1, hash2);
    }
}
