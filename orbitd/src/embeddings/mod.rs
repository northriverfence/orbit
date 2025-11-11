use anyhow::Result;
use ndarray::Array1;
use tracing::info;

/// Sentence embedding model using MiniLM-L6-V2
///
/// NOTE: This is currently a simplified implementation.
/// Full ONNX Runtime integration with MiniLM-L6-V2 is pending due to API compatibility.
/// The architecture is in place - embeddings are generated, stored, and used for similarity search.
///
/// TODO: Complete ONNX Runtime integration when ort crate API stabilizes
pub struct EmbeddingModel {
    // Placeholder for now - will hold ONNX session and tokenizer
    _embedding_dim: usize,
}

impl EmbeddingModel {
    /// Initialize the embedding model
    ///
    /// NOTE: Simplified implementation using TF-IDF style approach
    /// Full ONNX/transformer model integration pending
    pub async fn new() -> Result<Self> {
        info!("✓ Embedding model initialized (simplified TF-IDF implementation)");
        info!("  Full transformer model integration is pending");

        Ok(Self {
            _embedding_dim: 384, // MiniLM-L6-V2 dimension
        })
    }

    /// Generate embedding for a single text
    ///
    /// Current implementation uses a hash-based TF-IDF style embedding
    /// TODO: Replace with actual MiniLM-L6-V2 ONNX model
    pub fn embed(&self, text: &str) -> Result<Array1<f32>> {
        // Simple word-based embedding (TF-IDF style)
        let lowercase_text = text.to_lowercase();
        let words: Vec<&str> = lowercase_text
            .split_whitespace()
            .filter(|w| w.len() > 2) // Skip short words
            .collect();

        // Create a fixed-size embedding vector
        let mut embedding = Array1::zeros(384);

        // Hash each word and set corresponding dimensions
        for (i, word) in words.iter().enumerate() {
            let hash = Self::hash_word(word);
            let idx = (hash % 384) as usize;

            // TF-IDF style: count occurrences
            embedding[idx] += 1.0;

            // Add positional information
            if i < words.len() {
                let pos_idx = ((hash + i as u64) % 384) as usize;
                embedding[pos_idx] += 0.5;
            }
        }

        // Normalize
        let normalized = Self::normalize(&embedding);

        Ok(normalized)
    }

    /// Simple hash function for words
    fn hash_word(word: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        word.hash(&mut hasher);
        hasher.finish()
    }

    /// L2 normalize a vector
    fn normalize(vec: &Array1<f32>) -> Array1<f32> {
        let norm = vec.mapv(|x| x * x).sum().sqrt().max(1e-12);
        vec / norm
    }

    /// Calculate cosine similarity between two embeddings
    pub fn cosine_similarity(a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        // Since embeddings are normalized, dot product = cosine similarity
        a.dot(b)
    }
}

impl Clone for EmbeddingModel {
    fn clone(&self) -> Self {
        Self {
            _embedding_dim: self._embedding_dim,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_model_initialization() {
        let model = EmbeddingModel::new().await;
        assert!(model.is_ok(), "Failed to initialize embedding model");

        let model = model.unwrap();
        assert_eq!(
            model._embedding_dim, 384,
            "Embedding dimension should be 384"
        );
    }

    #[tokio::test]
    async fn test_embedding_generation() {
        let model = EmbeddingModel::new().await.unwrap();

        let embedding = model.embed("test command").unwrap();
        assert_eq!(embedding.len(), 384, "Embedding should have 384 dimensions");

        // Check that embedding is normalized (L2 norm should be ~1.0)
        let norm: f32 = embedding.mapv(|x| x * x).sum().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.01,
            "Embedding should be normalized, got norm: {}",
            norm
        );
    }

    #[tokio::test]
    async fn test_embedding_similarity() {
        let model = EmbeddingModel::new().await.unwrap();

        let emb1 = model.embed("show files").unwrap();
        let emb2 = model.embed("list files").unwrap();
        let emb3 = model.embed("delete database").unwrap();

        let sim_similar = EmbeddingModel::cosine_similarity(&emb1, &emb2);
        let sim_different = EmbeddingModel::cosine_similarity(&emb1, &emb3);

        // With word overlap, similar commands should have higher similarity
        assert!(
            sim_similar > sim_different,
            "Similar commands should have higher similarity: {} vs {}",
            sim_similar,
            sim_different
        );
        assert!(
            sim_similar > 0.3,
            "Similar commands should have reasonable similarity: {}",
            sim_similar
        );
    }

    #[tokio::test]
    async fn test_embedding_determinism() {
        let model = EmbeddingModel::new().await.unwrap();

        // Same input should produce same embedding
        let emb1 = model.embed("list files in directory").unwrap();
        let emb2 = model.embed("list files in directory").unwrap();

        let similarity = EmbeddingModel::cosine_similarity(&emb1, &emb2);
        assert!(
            (similarity - 1.0).abs() < 0.001,
            "Same input should produce identical embeddings, got similarity: {}",
            similarity
        );
    }

    #[tokio::test]
    async fn test_embedding_case_insensitivity() {
        let model = EmbeddingModel::new().await.unwrap();

        let emb1 = model.embed("Show Files").unwrap();
        let emb2 = model.embed("show files").unwrap();

        let similarity = EmbeddingModel::cosine_similarity(&emb1, &emb2);
        assert!(
            (similarity - 1.0).abs() < 0.001,
            "Case should not matter, got similarity: {}",
            similarity
        );
    }

    #[tokio::test]
    async fn test_embedding_word_order_sensitivity() {
        let model = EmbeddingModel::new().await.unwrap();

        let emb1 = model.embed("delete files quickly").unwrap();
        let emb2 = model.embed("quickly delete files").unwrap();

        let similarity = EmbeddingModel::cosine_similarity(&emb1, &emb2);
        // Word order matters due to positional encoding, but should still be very similar
        assert!(
            similarity > 0.7,
            "Word order variations should still be similar: {}",
            similarity
        );
    }

    #[test]
    fn test_hash_word_consistency() {
        // Same word should produce same hash
        let hash1 = EmbeddingModel::hash_word("test");
        let hash2 = EmbeddingModel::hash_word("test");
        assert_eq!(hash1, hash2, "Hash should be consistent");

        // Different words should (likely) produce different hashes
        let hash3 = EmbeddingModel::hash_word("different");
        assert_ne!(hash1, hash3, "Different words should have different hashes");
    }

    #[test]
    fn test_normalize_vector() {
        let vec = Array1::from_vec(vec![3.0, 4.0]);
        let normalized = EmbeddingModel::normalize(&vec);

        let norm: f32 = normalized.mapv(|x| x * x).sum().sqrt();
        assert!(
            (norm - 1.0).abs() < 0.001,
            "Normalized vector should have unit norm: {}",
            norm
        );

        // Check values
        assert!(
            (normalized[0] - 0.6).abs() < 0.01,
            "First element should be 0.6"
        );
        assert!(
            (normalized[1] - 0.8).abs() < 0.01,
            "Second element should be 0.8"
        );
    }

    #[test]
    fn test_normalize_zero_vector() {
        let vec = Array1::zeros(10);
        let normalized = EmbeddingModel::normalize(&vec);

        // Zero vector normalized should still be zero (with epsilon protection)
        assert!(
            normalized.iter().all(|&x| x.abs() < 1e-10),
            "Zero vector should normalize to near-zero"
        );
    }

    #[tokio::test]
    async fn test_empty_text_embedding() {
        let model = EmbeddingModel::new().await.unwrap();

        let emb = model.embed("").unwrap();
        assert_eq!(
            emb.len(),
            384,
            "Empty text should still produce 384-dim embedding"
        );

        // Should be all zeros (or near zero after normalization)
        let sum: f32 = emb.iter().map(|&x| x.abs()).sum();
        assert!(
            sum < 0.1,
            "Empty text embedding should be near zero: {}",
            sum
        );
    }

    #[tokio::test]
    async fn test_short_words_filtered() {
        let model = EmbeddingModel::new().await.unwrap();

        // Text with only short words (2 chars or less) should be mostly empty
        let emb1 = model.embed("a b c to in of").unwrap();

        // The sum of the embedding should be very small
        let sum: f32 = emb1.iter().map(|&x| x.abs()).sum();
        assert!(
            sum < 0.1,
            "Short words should be filtered, got sum: {}",
            sum
        );

        // Text with actual words should have higher sum
        let emb2 = model.embed("list files directory").unwrap();
        let sum2: f32 = emb2.iter().map(|&x| x.abs()).sum();
        assert!(
            sum2 > 0.5,
            "Real words should produce non-zero embedding: {}",
            sum2
        );
    }
}
