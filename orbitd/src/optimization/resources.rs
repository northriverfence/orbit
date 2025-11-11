// Resource management and limiting for Orbit AI Terminal
//
// This module provides:
// - Memory usage limits and monitoring
// - CPU throttling and rate limiting
// - Concurrent request limits
// - Graceful degradation under resource pressure

use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{System, SystemExt, ProcessExt};
use tokio::sync::Semaphore;

/// Resource limiter for controlling system resource usage
pub struct ResourceLimiter {
    max_memory_bytes: usize,
    max_concurrent_requests: usize,
    semaphore: Arc<Semaphore>,
    active_requests: Arc<AtomicUsize>,
    total_requests: Arc<AtomicUsize>,
    rejected_requests: Arc<AtomicUsize>,
    start_time: Instant,
}

impl ResourceLimiter {
    /// Create a new resource limiter
    ///
    /// # Arguments
    /// * `max_memory_mb` - Maximum memory usage in megabytes
    /// * `max_concurrent_requests` - Maximum concurrent requests
    ///
    /// # Example
    /// ```
    /// let limiter = ResourceLimiter::new(500, 100); // 500MB, 100 concurrent
    /// ```
    pub fn new(max_memory_mb: usize, max_concurrent_requests: usize) -> Self {
        Self {
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            max_concurrent_requests,
            semaphore: Arc::new(Semaphore::new(max_concurrent_requests)),
            active_requests: Arc::new(AtomicUsize::new(0)),
            total_requests: Arc::new(AtomicUsize::new(0)),
            rejected_requests: Arc::new(AtomicUsize::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Check if memory usage is within limits
    pub fn check_memory(&self) -> Result<()> {
        let mut sys = System::new_all();
        sys.refresh_all();

        let pid = sysinfo::get_current_pid().map_err(|e| anyhow!("Failed to get PID: {}", e))?;

        if let Some(process) = sys.process(pid) {
            let memory_bytes = process.memory() as usize * 1024; // memory() returns KB

            if memory_bytes > self.max_memory_bytes {
                crate::observability::get_metrics().record_error("memory_limit");
                return Err(anyhow!(
                    "Memory limit exceeded: {} MB / {} MB",
                    memory_bytes / (1024 * 1024),
                    self.max_memory_bytes / (1024 * 1024)
                ));
            }

            // Update metrics
            crate::observability::get_metrics().update_memory_usage(memory_bytes);

            // Warn at 80% utilization
            if memory_bytes > (self.max_memory_bytes * 80 / 100) {
                tracing::warn!(
                    memory_mb = memory_bytes / (1024 * 1024),
                    limit_mb = self.max_memory_bytes / (1024 * 1024),
                    "High memory usage detected"
                );
            }
        }

        Ok(())
    }

    /// Acquire a permit for request processing
    ///
    /// This blocks until a permit is available or returns an error if
    /// resources are exhausted
    pub async fn acquire_permit(&self) -> Result<RequestPermit> {
        // Check memory before acquiring permit
        self.check_memory()?;

        // Try to acquire with timeout
        let permit = tokio::time::timeout(
            Duration::from_secs(30),
            self.semaphore.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            self.rejected_requests.fetch_add(1, Ordering::SeqCst);
            crate::observability::get_metrics().record_error("request_timeout");
            anyhow!("Request timeout: too many concurrent requests")
        })?
        .map_err(|e| anyhow!("Failed to acquire permit: {}", e))?;

        self.active_requests.fetch_add(1, Ordering::SeqCst);
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        // Update metrics
        let active = self.active_requests.load(Ordering::SeqCst);
        crate::observability::get_metrics().update_active_connections(active);

        Ok(RequestPermit {
            _permit: permit,
            active_requests: Arc::clone(&self.active_requests),
        })
    }

    /// Try to acquire a permit without blocking
    pub fn try_acquire_permit(&self) -> Result<RequestPermit> {
        // Check memory first
        self.check_memory()?;

        let permit = self.semaphore.clone().try_acquire_owned().map_err(|_| {
            self.rejected_requests.fetch_add(1, Ordering::SeqCst);
            crate::observability::get_metrics().record_error("request_rejected");
            anyhow!("Too many concurrent requests")
        })?;

        self.active_requests.fetch_add(1, Ordering::SeqCst);
        self.total_requests.fetch_add(1, Ordering::SeqCst);

        let active = self.active_requests.load(Ordering::SeqCst);
        crate::observability::get_metrics().update_active_connections(active);

        Ok(RequestPermit {
            _permit: permit,
            active_requests: Arc::clone(&self.active_requests),
        })
    }

    /// Get resource statistics
    pub fn stats(&self) -> ResourceStats {
        let active = self.active_requests.load(Ordering::SeqCst);
        let total = self.total_requests.load(Ordering::SeqCst);
        let rejected = self.rejected_requests.load(Ordering::SeqCst);

        ResourceStats {
            active_requests: active,
            total_requests: total,
            rejected_requests: rejected,
            max_concurrent: self.max_concurrent_requests,
            max_memory_bytes: self.max_memory_bytes,
            uptime_seconds: self.start_time.elapsed().as_secs(),
            utilization_pct: (active as f64 / self.max_concurrent_requests as f64) * 100.0,
        }
    }

    /// Check if system is under heavy load
    pub fn is_overloaded(&self) -> bool {
        let active = self.active_requests.load(Ordering::SeqCst);
        let utilization = (active as f64 / self.max_concurrent_requests as f64) * 100.0;

        utilization > 80.0
    }

    /// Get recommended action based on current load
    pub fn load_action(&self) -> LoadAction {
        let utilization = self.stats().utilization_pct;

        if utilization > 90.0 {
            LoadAction::Reject
        } else if utilization > 75.0 {
            LoadAction::Throttle
        } else {
            LoadAction::Accept
        }
    }
}

impl Clone for ResourceLimiter {
    fn clone(&self) -> Self {
        Self {
            max_memory_bytes: self.max_memory_bytes,
            max_concurrent_requests: self.max_concurrent_requests,
            semaphore: Arc::clone(&self.semaphore),
            active_requests: Arc::clone(&self.active_requests),
            total_requests: Arc::clone(&self.total_requests),
            rejected_requests: Arc::clone(&self.rejected_requests),
            start_time: self.start_time,
        }
    }
}

/// Request permit that releases on drop
pub struct RequestPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    active_requests: Arc<AtomicUsize>,
}

impl Drop for RequestPermit {
    fn drop(&mut self) {
        let active = self.active_requests.fetch_sub(1, Ordering::SeqCst) - 1;
        crate::observability::get_metrics().update_active_connections(active);
    }
}

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceStats {
    pub active_requests: usize,
    pub total_requests: usize,
    pub rejected_requests: usize,
    pub max_concurrent: usize,
    pub max_memory_bytes: usize,
    pub uptime_seconds: u64,
    pub utilization_pct: f64,
}

impl ResourceStats {
    /// Get rejection rate
    pub fn rejection_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.rejected_requests as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Get requests per second
    pub fn requests_per_second(&self) -> f64 {
        if self.uptime_seconds == 0 {
            0.0
        } else {
            self.total_requests as f64 / self.uptime_seconds as f64
        }
    }
}

/// Recommended action based on system load
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadAction {
    /// Accept request normally
    Accept,
    /// Throttle request (add delay)
    Throttle,
    /// Reject request (system overloaded)
    Reject,
}

/// Rate limiter using token bucket algorithm
pub struct RateLimiter {
    tokens: Arc<AtomicUsize>,
    max_tokens: usize,
    refill_rate: usize,
    last_refill: Arc<parking_lot::Mutex<Instant>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_tokens` - Maximum number of tokens (burst size)
    /// * `refill_rate` - Number of tokens to add per second
    pub fn new(max_tokens: usize, refill_rate: usize) -> Self {
        Self {
            tokens: Arc::new(AtomicUsize::new(max_tokens)),
            max_tokens,
            refill_rate,
            last_refill: Arc::new(parking_lot::Mutex::new(Instant::now())),
        }
    }

    /// Try to consume a token
    pub fn try_acquire(&self) -> bool {
        self.refill();

        let mut current = self.tokens.load(Ordering::SeqCst);

        loop {
            if current == 0 {
                return false;
            }

            match self.tokens.compare_exchange(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    /// Wait until a token is available
    pub async fn acquire(&self) {
        loop {
            if self.try_acquire() {
                return;
            }

            // Wait a bit before trying again
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&self) {
        let mut last_refill = self.last_refill.lock();
        let elapsed = last_refill.elapsed();

        if elapsed < Duration::from_millis(100) {
            return; // Don't refill too frequently
        }

        let tokens_to_add = (elapsed.as_secs_f64() * self.refill_rate as f64) as usize;

        if tokens_to_add > 0 {
            let current = self.tokens.load(Ordering::SeqCst);
            let new_tokens = (current + tokens_to_add).min(self.max_tokens);
            self.tokens.store(new_tokens, Ordering::SeqCst);
            *last_refill = Instant::now();
        }
    }

    /// Get current token count
    pub fn available_tokens(&self) -> usize {
        self.refill();
        self.tokens.load(Ordering::SeqCst)
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            tokens: Arc::clone(&self.tokens),
            max_tokens: self.max_tokens,
            refill_rate: self.refill_rate,
            last_refill: Arc::clone(&self.last_refill),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_limiter_creation() {
        let limiter = ResourceLimiter::new(500, 100);
        let stats = limiter.stats();

        assert_eq!(stats.max_concurrent, 100);
        assert_eq!(stats.max_memory_bytes, 500 * 1024 * 1024);
        assert_eq!(stats.active_requests, 0);
    }

    #[tokio::test]
    async fn test_acquire_permit() {
        let limiter = ResourceLimiter::new(1000, 2);

        // Should acquire successfully
        let permit1 = limiter.acquire_permit().await;
        assert!(permit1.is_ok());

        let permit2 = limiter.acquire_permit().await;
        assert!(permit2.is_ok());

        // Third should timeout (2 is the limit)
        let result = tokio::time::timeout(
            Duration::from_millis(100),
            limiter.acquire_permit()
        ).await;

        assert!(result.is_err()); // Timeout
    }

    #[test]
    fn test_resource_stats_calculations() {
        let limiter = ResourceLimiter::new(500, 100);

        limiter.total_requests.store(1000, Ordering::SeqCst);
        limiter.rejected_requests.store(50, Ordering::SeqCst);
        limiter.active_requests.store(80, Ordering::SeqCst);

        let stats = limiter.stats();

        assert_eq!(stats.rejection_rate(), 5.0);
        assert_eq!(stats.utilization_pct, 80.0);
    }

    #[test]
    fn test_load_action() {
        let limiter = ResourceLimiter::new(500, 100);

        // Low load
        assert_eq!(limiter.load_action(), LoadAction::Accept);

        // Medium load
        limiter.active_requests.store(80, Ordering::SeqCst);
        assert_eq!(limiter.load_action(), LoadAction::Throttle);

        // High load
        limiter.active_requests.store(95, Ordering::SeqCst);
        assert_eq!(limiter.load_action(), LoadAction::Reject);
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(10, 5);

        // Should have full tokens initially
        assert_eq!(limiter.available_tokens(), 10);

        // Consume all tokens
        for _ in 0..10 {
            assert!(limiter.try_acquire());
        }

        // Should be empty
        assert_eq!(limiter.available_tokens(), 0);
        assert!(!limiter.try_acquire());
    }

    #[tokio::test]
    async fn test_rate_limiter_refill() {
        let limiter = RateLimiter::new(5, 10); // 10 tokens per second

        // Consume all tokens
        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }

        assert_eq!(limiter.available_tokens(), 0);

        // Wait for refill (100ms = 1 token at 10/sec)
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should have refilled
        assert!(limiter.available_tokens() > 0);
    }
}
