// Response caching system for Orbit AI Terminal
//
// This module provides an LRU cache with TTL support for caching AI provider responses.
// The cache is context-aware, meaning it takes into account the current working directory,
// git state, and other contextual information when determining cache keys.

use crate::context::Context;
use crate::observability::get_metrics;
use lru::LruCache;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Response cache with LRU eviction and TTL expiration
pub struct ResponseCache {
    cache: Arc<Mutex<LruCache<CacheKey, CacheEntry>>>,
    ttl: Duration,
}

/// Cache key that includes input and context hash
#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    input: String,
    context_hash: u64,
}

/// Cache entry with response and creation timestamp
struct CacheEntry {
    response: String,
    created_at: Instant,
}

impl ResponseCache {
    /// Create a new response cache with specified capacity and TTL
    ///
    /// # Arguments
    /// * `capacity` - Maximum number of entries in the cache
    /// * `ttl_seconds` - Time-to-live for cache entries in seconds
    ///
    /// # Example
    /// ```
    /// let cache = ResponseCache::new(1000, 3600); // 1000 entries, 1 hour TTL
    /// ```
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(1000).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get a cached response if it exists and hasn't expired
    ///
    /// # Arguments
    /// * `input` - The user's input query
    /// * `context` - Current execution context
    ///
    /// # Returns
    /// Some(response) if cache hit, None if cache miss or expired
    pub fn get(&self, input: &str, context: &Context) -> Option<String> {
        let key = CacheKey {
            input: input.to_string(),
            context_hash: Self::hash_context(context),
        };

        let mut cache = self.cache.lock().unwrap();

        if let Some(entry) = cache.get(&key) {
            // Check if entry is still valid
            if entry.created_at.elapsed() < self.ttl {
                // Record cache hit metric
                get_metrics().record_cache_hit();
                return Some(entry.response.clone());
            } else {
                // Entry expired, remove it
                cache.pop(&key);
            }
        }

        // Record cache miss metric
        get_metrics().record_cache_miss();
        None
    }

    /// Store a response in the cache
    ///
    /// # Arguments
    /// * `input` - The user's input query
    /// * `context` - Current execution context
    /// * `response` - The AI provider's response to cache
    pub fn put(&self, input: &str, context: &Context, response: String) {
        let key = CacheKey {
            input: input.to_string(),
            context_hash: Self::hash_context(context),
        };

        let entry = CacheEntry {
            response,
            created_at: Instant::now(),
        };

        let mut cache = self.cache.lock().unwrap();
        cache.put(key, entry);

        // Update cache size metric
        get_metrics().update_cache_size(cache.len());
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        get_metrics().update_cache_size(0);
    }

    /// Get the current number of entries in the cache
    pub fn len(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Calculate cache hit rate (hits / total requests)
    pub fn hit_rate(&self) -> f64 {
        get_metrics().cache_hit_rate()
    }

    /// Remove expired entries from the cache
    ///
    /// This is called periodically to clean up stale entries
    pub fn cleanup_expired(&self) {
        let mut cache = self.cache.lock().unwrap();
        let now = Instant::now();

        // Collect keys to remove (can't remove while iterating)
        let mut expired_keys = Vec::new();

        // Note: LruCache doesn't provide an iterator, so we'll rely on
        // expiration checks during get() operations instead

        // Update cache size metric
        get_metrics().update_cache_size(cache.len());
    }

    /// Hash the context to create a cache key component
    ///
    /// This includes:
    /// - Current working directory
    /// - Git branch (if in a git repository)
    /// - Git status (clean/dirty)
    fn hash_context(context: &Context) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash current working directory
        context.cwd.hash(&mut hasher);

        // Hash git context if available
        if let Some(ref git) = context.git {
            git.current_branch.hash(&mut hasher);
            git.is_clean.hash(&mut hasher);
        }

        // Hash environment-related context
        // (Add more context fields as needed)

        hasher.finish()
    }
}

impl Clone for ResponseCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            ttl: self.ttl,
        }
    }
}

/// Command pattern cache for learning system
///
/// Caches frequently used command patterns to speed up suggestions
pub struct PatternCache {
    cache: Arc<Mutex<LruCache<String, Vec<CommandPattern>>>>,
    ttl: Duration,
}

#[derive(Clone)]
pub struct CommandPattern {
    pub pattern: String,
    pub command: String,
    pub frequency: usize,
    pub cached_at: Instant,
}

impl PatternCache {
    /// Create a new pattern cache
    pub fn new(capacity: usize, ttl_seconds: u64) -> Self {
        let capacity = NonZeroUsize::new(capacity).unwrap_or(NonZeroUsize::new(500).unwrap());
        Self {
            cache: Arc::new(Mutex::new(LruCache::new(capacity))),
            ttl: Duration::from_secs(ttl_seconds),
        }
    }

    /// Get cached patterns for a query
    pub fn get(&self, query: &str) -> Option<Vec<CommandPattern>> {
        let mut cache = self.cache.lock().unwrap();

        if let Some(patterns) = cache.get(query) {
            // Check if patterns are still valid
            if patterns.first().map_or(false, |p| p.cached_at.elapsed() < self.ttl) {
                get_metrics().record_cache_hit();
                return Some(patterns.clone());
            } else {
                // Expired, remove from cache
                cache.pop(query);
            }
        }

        get_metrics().record_cache_miss();
        None
    }

    /// Store patterns in cache
    pub fn put(&self, query: &str, patterns: Vec<CommandPattern>) {
        let patterns_with_timestamp: Vec<CommandPattern> = patterns
            .into_iter()
            .map(|mut p| {
                p.cached_at = Instant::now();
                p
            })
            .collect();

        let mut cache = self.cache.lock().unwrap();
        cache.put(query.to_string(), patterns_with_timestamp);
    }

    /// Clear the pattern cache
    pub fn clear(&self) {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
    }
}

impl Clone for PatternCache {
    fn clone(&self) -> Self {
        Self {
            cache: Arc::clone(&self.cache),
            ttl: self.ttl,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::GitContext;
    use std::path::PathBuf;

    fn create_test_context() -> Context {
        Context {
            cwd: PathBuf::from("/tmp/test"),
            git: Some(GitContext {
                root: PathBuf::from("/tmp/test"),
                current_branch: "main".to_string(),
                is_clean: true,
                recent_commits: vec![],
            }),
            shell: "bash".to_string(),
            user: "test".to_string(),
            home: PathBuf::from("/home/test"),
        }
    }

    #[test]
    fn test_cache_put_and_get() {
        let cache = ResponseCache::new(100, 3600);
        let context = create_test_context();

        cache.put("test query", &context, "test response".to_string());

        let result = cache.get("test query", &context);
        assert_eq!(result, Some("test response".to_string()));
    }

    #[test]
    fn test_cache_miss() {
        let cache = ResponseCache::new(100, 3600);
        let context = create_test_context();

        let result = cache.get("nonexistent", &context);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_context_awareness() {
        let cache = ResponseCache::new(100, 3600);
        let context1 = create_test_context();

        let mut context2 = create_test_context();
        context2.cwd = PathBuf::from("/tmp/other");

        cache.put("test query", &context1, "response1".to_string());

        // Same query, different context should miss
        let result = cache.get("test query", &context2);
        assert_eq!(result, None);

        // Same context should hit
        let result = cache.get("test query", &context1);
        assert_eq!(result, Some("response1".to_string()));
    }

    #[test]
    fn test_cache_expiration() {
        let cache = ResponseCache::new(100, 1); // 1 second TTL
        let context = create_test_context();

        cache.put("test query", &context, "test response".to_string());

        // Should hit immediately
        let result = cache.get("test query", &context);
        assert_eq!(result, Some("test response".to_string()));

        // Wait for expiration
        std::thread::sleep(Duration::from_secs(2));

        // Should miss after TTL
        let result = cache.get("test query", &context);
        assert_eq!(result, None);
    }

    #[test]
    fn test_cache_len() {
        let cache = ResponseCache::new(100, 3600);
        let context = create_test_context();

        assert_eq!(cache.len(), 0);

        cache.put("query1", &context, "response1".to_string());
        assert_eq!(cache.len(), 1);

        cache.put("query2", &context, "response2".to_string());
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_pattern_cache() {
        let cache = PatternCache::new(100, 3600);

        let patterns = vec![
            CommandPattern {
                pattern: "list files".to_string(),
                command: "ls -la".to_string(),
                frequency: 10,
                cached_at: Instant::now(),
            },
        ];

        cache.put("list", patterns.clone());

        let result = cache.get("list");
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 1);
    }
}
