//! Merkle tree for chunk verification

use blake3::Hasher;

pub struct MerkleTree {
    root: String,
    #[allow(dead_code)]
    leaves: Vec<String>,
}

impl MerkleTree {
    pub fn new(chunk_hashes: Vec<String>) -> Self {
        let root = Self::compute_root(&chunk_hashes);
        Self {
            root,
            leaves: chunk_hashes,
        }
    }

    pub fn root(&self) -> &str {
        &self.root
    }

    fn compute_root(hashes: &[String]) -> String {
        if hashes.is_empty() {
            return String::new();
        }

        if hashes.len() == 1 {
            return hashes[0].clone();
        }

        // Simple implementation: hash all leaves together
        let mut hasher = Hasher::new();
        for hash in hashes {
            hasher.update(hash.as_bytes());
        }
        hasher.finalize().to_hex().to_string()
    }

    pub fn verify(&self, chunk_hashes: &[String]) -> bool {
        Self::compute_root(chunk_hashes) == self.root
    }
}
