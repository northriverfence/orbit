// Async operation optimizations for Orbit AI Terminal
//
// This module provides optimized async operations including:
// - Parallel provider queries with first-success semantics
// - Smart routing with fallback strategies
// - Non-blocking context detection
// - Concurrent learning system queries

use crate::context::Context;
use crate::providers::{Provider, ProviderResponse};
use anyhow::{anyhow, Result};
use futures::stream::{FuturesUnordered, StreamExt};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Semaphore;

/// Request sent to AI providers
#[derive(Clone)]
pub struct Request {
    pub prompt: String,
    pub context: Context,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}

/// Manager for optimized provider queries
pub struct OptimizedProviderManager {
    providers: Vec<(String, Arc<dyn Provider + Send + Sync>)>,
    primary_provider: Option<String>,
    max_concurrent: usize,
    semaphore: Arc<Semaphore>,
}

impl OptimizedProviderManager {
    /// Create a new optimized provider manager
    ///
    /// # Arguments
    /// * `max_concurrent` - Maximum concurrent provider queries
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            providers: Vec::new(),
            primary_provider: None,
            max_concurrent,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    /// Register a provider
    pub fn add_provider(&mut self, name: String, provider: Arc<dyn Provider + Send + Sync>) {
        self.providers.push((name, provider));
    }

    /// Set the primary provider for smart routing
    pub fn set_primary(&mut self, name: String) {
        self.primary_provider = Some(name);
    }

    /// Query multiple providers in parallel and return the first successful response
    ///
    /// This races all enabled providers and returns as soon as any provider succeeds,
    /// canceling the remaining queries for efficiency.
    ///
    /// # Arguments
    /// * `request` - The request to send to providers
    ///
    /// # Returns
    /// The provider name and response from the first provider to succeed
    pub async fn query_parallel(&self, request: &Request) -> Result<(String, ProviderResponse)> {
        if self.providers.is_empty() {
            return Err(anyhow!("No providers registered"));
        }

        let mut tasks = FuturesUnordered::new();

        for (name, provider) in &self.providers {
            let req = request.clone();
            let provider = Arc::clone(provider);
            let name = name.clone();
            let semaphore = Arc::clone(&self.semaphore);

            tasks.push(tokio::spawn(async move {
                // Acquire permit for rate limiting
                let _permit = semaphore.acquire().await.ok()?;

                let start = Instant::now();
                let result = provider
                    .complete(&req.prompt, &req.context)
                    .await
                    .ok()?;

                let duration = start.elapsed();

                Some((name, result, duration))
            }));
        }

        // Return first successful response
        while let Some(result) = tasks.next().await {
            if let Ok(Some((name, response, duration))) = result {
                // Record metrics
                crate::observability::get_metrics().record_ai_query(
                    &name,
                    duration.as_secs_f64(),
                    0, // Token counting would happen in provider
                    0,
                );

                return Ok((
                    name,
                    ProviderResponse {
                        content: response,
                    },
                ));
            }
        }

        Err(anyhow!("All providers failed"))
    }

    /// Query providers with smart routing strategy
    ///
    /// This tries the primary provider first for low latency, then falls back
    /// to parallel queries if the primary fails.
    ///
    /// # Arguments
    /// * `request` - The request to send to providers
    ///
    /// # Returns
    /// The provider name and response
    pub async fn query_with_routing(&self, request: &Request) -> Result<(String, ProviderResponse)> {
        // Try primary provider first
        if let Some(primary_name) = &self.primary_provider {
            if let Some((name, provider)) = self
                .providers
                .iter()
                .find(|(n, _)| n == primary_name)
            {
                let _permit = self.semaphore.acquire().await?;
                let start = Instant::now();

                if let Ok(response) = provider.complete(&request.prompt, &request.context).await {
                    let duration = start.elapsed();

                    // Record metrics
                    crate::observability::get_metrics().record_ai_query(
                        name,
                        duration.as_secs_f64(),
                        0,
                        0,
                    );

                    return Ok((
                        name.clone(),
                        ProviderResponse {
                            content: response,
                        },
                    ));
                }
            }
        }

        // Fall back to parallel query of all providers
        self.query_parallel(request).await
    }

    /// Query with timeout and automatic retry
    ///
    /// # Arguments
    /// * `request` - The request to send to providers
    /// * `timeout_secs` - Timeout in seconds
    /// * `max_retries` - Maximum number of retries
    pub async fn query_with_retry(
        &self,
        request: &Request,
        timeout_secs: u64,
        max_retries: usize,
    ) -> Result<(String, ProviderResponse)> {
        for attempt in 0..=max_retries {
            let result = tokio::time::timeout(
                std::time::Duration::from_secs(timeout_secs),
                self.query_with_routing(request),
            )
            .await;

            match result {
                Ok(Ok(response)) => return Ok(response),
                Ok(Err(e)) if attempt == max_retries => return Err(e),
                Err(_) if attempt == max_retries => {
                    return Err(anyhow!("Query timeout after {} retries", max_retries))
                }
                _ => {
                    // Exponential backoff
                    let backoff = std::time::Duration::from_millis(100 * 2_u64.pow(attempt as u32));
                    tokio::time::sleep(backoff).await;
                }
            }
        }

        Err(anyhow!("Max retries exceeded"))
    }

    /// Get provider statistics
    pub fn stats(&self) -> ProviderStats {
        ProviderStats {
            total_providers: self.providers.len(),
            primary_provider: self.primary_provider.clone(),
            max_concurrent: self.max_concurrent,
        }
    }
}

/// Provider statistics
#[derive(Debug, Clone)]
pub struct ProviderStats {
    pub total_providers: usize,
    pub primary_provider: Option<String>,
    pub max_concurrent: usize,
}

/// Parallel context detection
///
/// Detects multiple context signals in parallel for faster startup
pub async fn detect_context_parallel() -> Result<Context> {
    let (cwd_result, git_result, shell_result, user_result) = tokio::join!(
        detect_cwd(),
        detect_git(),
        detect_shell(),
        detect_user(),
    );

    Ok(Context {
        cwd: cwd_result?,
        git: git_result.ok(),
        shell: shell_result?,
        user: user_result?,
        home: detect_home().await?,
    })
}

// Helper functions for parallel context detection
async fn detect_cwd() -> Result<std::path::PathBuf> {
    Ok(std::env::current_dir()?)
}

async fn detect_git() -> Result<crate::context::GitContext> {
    // This would integrate with git detection logic
    Err(anyhow!("Git detection not implemented"))
}

async fn detect_shell() -> Result<String> {
    Ok(std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string()))
}

async fn detect_user() -> Result<String> {
    Ok(std::env::var("USER").unwrap_or_else(|_| "unknown".to_string()))
}

async fn detect_home() -> Result<std::path::PathBuf> {
    Ok(dirs::home_dir().ok_or_else(|| anyhow!("Could not detect home directory"))?)
}

/// Batch operation executor for processing multiple operations efficiently
pub struct BatchExecutor<T> {
    batch_size: usize,
    items: Vec<T>,
}

impl<T> BatchExecutor<T> {
    /// Create a new batch executor
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            items: Vec::new(),
        }
    }

    /// Add an item to the batch
    pub fn add(&mut self, item: T) {
        self.items.push(item);
    }

    /// Check if batch is full
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.batch_size
    }

    /// Execute the batch with a given async function
    pub async fn execute<F, Fut, R>(&mut self, f: F) -> Vec<Result<R>>
    where
        F: Fn(T) -> Fut,
        Fut: std::future::Future<Output = Result<R>>,
        T: Clone,
    {
        let items = std::mem::take(&mut self.items);
        let mut tasks = FuturesUnordered::new();

        for item in items {
            tasks.push(f(item));
        }

        let mut results = Vec::new();
        while let Some(result) = tasks.next().await {
            results.push(result);
        }

        results
    }

    /// Get the number of pending items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if there are no pending items
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_context_detection() {
        let result = detect_context_parallel().await;
        assert!(result.is_ok());

        let context = result.unwrap();
        assert!(!context.cwd.as_os_str().is_empty());
        assert!(!context.shell.is_empty());
        assert!(!context.user.is_empty());
    }

    #[tokio::test]
    async fn test_batch_executor() {
        let mut executor = BatchExecutor::new(3);

        executor.add(1);
        executor.add(2);
        executor.add(3);

        assert!(executor.is_full());

        let results = executor
            .execute(|x| async move { Ok::<i32, anyhow::Error>(x * 2) })
            .await;

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_ok()));
    }

    #[test]
    fn test_batch_executor_size() {
        let mut executor = BatchExecutor::new(5);

        assert_eq!(executor.len(), 0);
        assert!(executor.is_empty());

        executor.add(1);
        assert_eq!(executor.len(), 1);
        assert!(!executor.is_empty());
        assert!(!executor.is_full());

        for i in 2..=5 {
            executor.add(i);
        }

        assert!(executor.is_full());
    }
}
