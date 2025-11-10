//! File chunking utilities

use blake3::Hasher;

pub struct FileChunker {
    chunk_size: usize,
}

impl FileChunker {
    pub fn new(chunk_size: usize) -> Self {
        Self { chunk_size }
    }

    pub fn chunk_count(&self, file_size: u64) -> usize {
        ((file_size as f64) / (self.chunk_size as f64)).ceil() as usize
    }
}

#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub index: usize,
    pub offset: u64,
    pub size: usize,
    pub hash: String,
}

impl ChunkInfo {
    pub fn compute_hash(data: &[u8]) -> String {
        let mut hasher = Hasher::new();
        hasher.update(data);
        hasher.finalize().to_hex().to_string()
    }
}
