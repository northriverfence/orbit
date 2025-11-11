// Performance optimization module for Orbit AI Terminal
//
// This module contains various optimizations including:
// - Async operation optimizations (parallel queries, smart routing)
// - Database optimizations (connection pooling, batch operations)
// - Resource management (memory limits, concurrency control)

pub mod async_ops;
pub mod database;
pub mod resources;

pub use async_ops::{OptimizedProviderManager, Request, detect_context_parallel};
pub use database::{DatabasePool, BatchInserter};
pub use resources::{ResourceLimiter, ResourceStats};
