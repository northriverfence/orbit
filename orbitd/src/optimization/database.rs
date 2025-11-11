// Database optimization utilities for Orbit AI Terminal
//
// This module provides:
// - Connection pooling for SQLite
// - Batch insert operations
// - Optimized query execution
// - WAL mode and performance tuning

use anyhow::{Context as _, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Pool, Sqlite, Transaction};
use std::path::Path;
use std::time::Instant;

/// Database connection pool with optimizations
pub struct DatabasePool {
    pool: Pool<Sqlite>,
}

impl DatabasePool {
    /// Create a new optimized database pool
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file
    /// * `max_connections` - Maximum number of connections in pool
    ///
    /// # Example
    /// ```
    /// let pool = DatabasePool::new("orbit.db", 10).await?;
    /// ```
    pub async fn new<P: AsRef<Path>>(db_path: P, max_connections: u32) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path.as_ref().display());

        let options = SqliteConnectOptions::new()
            .filename(db_path)
            .create_if_missing(true)
            // Enable WAL mode for better concurrency
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            // Optimize synchronous mode
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            // Increase cache size (10MB = 10000 pages)
            .pragma("cache_size", "10000")
            // Enable foreign keys
            .pragma("foreign_keys", "ON")
            // Set busy timeout
            .busy_timeout(std::time::Duration::from_secs(5));

        let pool = SqlitePoolOptions::new()
            .max_connections(max_connections)
            .min_connections(1)
            .acquire_timeout(std::time::Duration::from_secs(10))
            .connect_with(options)
            .await
            .context("Failed to create database pool")?;

        Ok(Self { pool })
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &Pool<Sqlite> {
        &self.pool
    }

    /// Begin a transaction
    pub async fn begin(&self) -> Result<Transaction<'_, Sqlite>> {
        Ok(self.pool.begin().await?)
    }

    /// Execute a query and return number of affected rows
    pub async fn execute(&self, query: &str) -> Result<u64> {
        let start = Instant::now();
        let result = sqlx::query(query).execute(&self.pool).await?;
        let duration = start.elapsed();

        // Log slow queries
        if duration.as_millis() > 100 {
            tracing::warn!(
                query = query,
                duration_ms = duration.as_millis(),
                "Slow database query detected"
            );
        }

        Ok(result.rows_affected())
    }

    /// Create optimized indexes for Orbit database
    pub async fn create_indexes(&self) -> Result<()> {
        let indexes = vec![
            // Command analytics indexes
            "CREATE INDEX IF NOT EXISTS idx_command_analytics_timestamp
             ON command_analytics(timestamp DESC)",

            "CREATE INDEX IF NOT EXISTS idx_command_analytics_result
             ON command_analytics(result)",

            "CREATE INDEX IF NOT EXISTS idx_command_analytics_original
             ON command_analytics(original_input)",

            // Command patterns indexes
            "CREATE INDEX IF NOT EXISTS idx_command_patterns_frequency
             ON command_patterns(frequency DESC)",

            "CREATE INDEX IF NOT EXISTS idx_command_patterns_pattern
             ON command_patterns(pattern)",

            // Context cache indexes
            "CREATE INDEX IF NOT EXISTS idx_context_cache_context_hash
             ON context_cache(context_hash)",

            "CREATE INDEX IF NOT EXISTS idx_context_cache_created_at
             ON context_cache(created_at DESC)",
        ];

        for index_sql in indexes {
            self.execute(index_sql).await?;
        }

        tracing::info!("Created database indexes");
        Ok(())
    }

    /// Optimize database (VACUUM, ANALYZE)
    pub async fn optimize(&self) -> Result<()> {
        tracing::info!("Running database optimization");

        // Analyze tables for query planner
        self.execute("ANALYZE").await?;

        // Note: VACUUM cannot be run in a transaction or with a connection pool
        // It should be run separately during maintenance windows

        tracing::info!("Database optimization complete");
        Ok(())
    }

    /// Get database statistics
    pub async fn stats(&self) -> Result<DatabaseStats> {
        let page_count: i64 = sqlx::query_scalar("PRAGMA page_count")
            .fetch_one(&self.pool)
            .await?;

        let page_size: i64 = sqlx::query_scalar("PRAGMA page_size")
            .fetch_one(&self.pool)
            .await?;

        let freelist_count: i64 = sqlx::query_scalar("PRAGMA freelist_count")
            .fetch_one(&self.pool)
            .await?;

        Ok(DatabaseStats {
            total_pages: page_count as usize,
            page_size_bytes: page_size as usize,
            free_pages: freelist_count as usize,
            size_bytes: (page_count * page_size) as usize,
            pool_connections: self.pool.size() as usize,
            pool_idle_connections: self.pool.num_idle() as usize,
        })
    }

    /// Close the pool
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_pages: usize,
    pub page_size_bytes: usize,
    pub free_pages: usize,
    pub size_bytes: usize,
    pub pool_connections: usize,
    pub pool_idle_connections: usize,
}

impl DatabaseStats {
    /// Get database size in megabytes
    pub fn size_mb(&self) -> f64 {
        self.size_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get fragmentation percentage
    pub fn fragmentation_pct(&self) -> f64 {
        if self.total_pages == 0 {
            0.0
        } else {
            (self.free_pages as f64 / self.total_pages as f64) * 100.0
        }
    }
}

/// Batch inserter for efficient bulk operations
pub struct BatchInserter {
    batch_size: usize,
    items: Vec<BatchItem>,
}

#[derive(Clone)]
pub struct BatchItem {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<String>,
}

impl BatchInserter {
    /// Create a new batch inserter
    ///
    /// # Arguments
    /// * `batch_size` - Number of items to batch before executing
    pub fn new(batch_size: usize) -> Self {
        Self {
            batch_size,
            items: Vec::with_capacity(batch_size),
        }
    }

    /// Add an item to the batch
    pub fn add(&mut self, item: BatchItem) {
        self.items.push(item);
    }

    /// Check if batch is ready to execute
    pub fn is_ready(&self) -> bool {
        self.items.len() >= self.batch_size
    }

    /// Execute the batch insert
    ///
    /// This groups items by table and executes multi-row inserts for efficiency
    pub async fn execute(&mut self, pool: &DatabasePool) -> Result<usize> {
        if self.items.is_empty() {
            return Ok(0);
        }

        let items = std::mem::take(&mut self.items);
        let mut total_inserted = 0;

        // Group by table
        let mut by_table: std::collections::HashMap<String, Vec<BatchItem>> =
            std::collections::HashMap::new();

        for item in items {
            by_table
                .entry(item.table.clone())
                .or_default()
                .push(item);
        }

        // Execute batch inserts per table
        let mut tx = pool.begin().await?;

        for (table, items) in by_table {
            if items.is_empty() {
                continue;
            }

            // Build multi-row insert
            let columns = &items[0].columns;
            let mut sql = format!(
                "INSERT INTO {} ({}) VALUES ",
                table,
                columns.join(", ")
            );

            let placeholders: Vec<String> = items
                .iter()
                .enumerate()
                .map(|(idx, _)| {
                    let start = idx * columns.len();
                    let params: Vec<String> = (0..columns.len())
                        .map(|i| format!("${}", start + i + 1))
                        .collect();
                    format!("({})", params.join(", "))
                })
                .collect();

            sql.push_str(&placeholders.join(", "));

            // Flatten all values
            let all_values: Vec<String> = items
                .iter()
                .flat_map(|item| item.values.clone())
                .collect();

            // Execute with all values
            let mut query = sqlx::query(&sql);
            for value in &all_values {
                query = query.bind(value);
            }

            let result = query.execute(&mut *tx).await?;
            total_inserted += result.rows_affected() as usize;
        }

        tx.commit().await?;
        Ok(total_inserted)
    }

    /// Force execute even if batch isn't full
    pub async fn flush(&mut self, pool: &DatabasePool) -> Result<usize> {
        self.execute(pool).await
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

/// Query optimizer that provides hints for better performance
pub struct QueryOptimizer;

impl QueryOptimizer {
    /// Analyze a query and provide optimization suggestions
    pub fn analyze(query: &str) -> Vec<OptimizationHint> {
        let mut hints = Vec::new();

        // Check for missing WHERE clause on large tables
        if query.contains("SELECT") && !query.contains("WHERE") && !query.contains("LIMIT") {
            hints.push(OptimizationHint::MissingWhereClause);
        }

        // Check for SELECT *
        if query.contains("SELECT *") {
            hints.push(OptimizationHint::SelectStar);
        }

        // Check for missing indexes on JOIN conditions
        if query.contains("JOIN") && !query.contains("INDEX") {
            hints.push(OptimizationHint::ConsiderIndex);
        }

        // Check for ORDER BY without LIMIT
        if query.contains("ORDER BY") && !query.contains("LIMIT") {
            hints.push(OptimizationHint::OrderByWithoutLimit);
        }

        hints
    }
}

#[derive(Debug, PartialEq)]
pub enum OptimizationHint {
    MissingWhereClause,
    SelectStar,
    ConsiderIndex,
    OrderByWithoutLimit,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_database_pool_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = DatabasePool::new(&db_path, 5).await;
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_database_stats() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let pool = DatabasePool::new(&db_path, 5).await.unwrap();

        // Create a test table
        pool.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, value TEXT)")
            .await
            .unwrap();

        let stats = pool.stats().await.unwrap();
        assert!(stats.total_pages > 0);
        assert!(stats.page_size_bytes > 0);
    }

    #[test]
    fn test_batch_inserter() {
        let mut inserter = BatchInserter::new(3);

        assert_eq!(inserter.len(), 0);
        assert!(inserter.is_empty());

        inserter.add(BatchItem {
            table: "test".to_string(),
            columns: vec!["col1".to_string()],
            values: vec!["val1".to_string()],
        });

        assert_eq!(inserter.len(), 1);
        assert!(!inserter.is_ready());

        inserter.add(BatchItem {
            table: "test".to_string(),
            columns: vec!["col1".to_string()],
            values: vec!["val2".to_string()],
        });

        inserter.add(BatchItem {
            table: "test".to_string(),
            columns: vec!["col1".to_string()],
            values: vec!["val3".to_string()],
        });

        assert!(inserter.is_ready());
    }

    #[test]
    fn test_query_optimizer() {
        let hints = QueryOptimizer::analyze("SELECT * FROM users");
        assert!(hints.contains(&OptimizationHint::SelectStar));

        let hints = QueryOptimizer::analyze("SELECT id FROM users ORDER BY created_at");
        assert!(hints.contains(&OptimizationHint::OrderByWithoutLimit));

        let hints = QueryOptimizer::analyze("SELECT id FROM users WHERE id = 1 LIMIT 10");
        assert!(hints.is_empty());
    }

    #[test]
    fn test_database_stats_calculations() {
        let stats = DatabaseStats {
            total_pages: 1000,
            page_size_bytes: 4096,
            free_pages: 100,
            size_bytes: 4096000,
            pool_connections: 5,
            pool_idle_connections: 3,
        };

        assert_eq!(stats.size_mb(), 3.90625);
        assert_eq!(stats.fragmentation_pct(), 10.0);
    }
}
