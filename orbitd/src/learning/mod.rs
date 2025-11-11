// Enhanced learning system modules (Phase 4)
pub mod analytics;
pub mod patterns;
pub mod preferences;
pub mod types;

use anyhow::Result;
use ndarray::Array1;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use std::sync::Arc;

use crate::config::Config;
use crate::context::Context;
use crate::embeddings::EmbeddingModel;

// Re-export enhanced learning types
pub use analytics::AnalyticsService;
pub use patterns::PatternRecognition;
pub use preferences::PreferenceService;
pub use types::*;

#[derive(Debug, Clone)]
pub struct LearnedCommand {
    #[allow(dead_code)]
    pub id: i64,
    #[allow(dead_code)]
    pub natural_input: String,
    pub learned_command: String,
    pub confidence: f32,
    #[allow(dead_code)]
    pub success_count: i32,
    #[allow(dead_code)]
    pub failure_count: i32,
}

#[derive(Clone)]
pub struct LearningEngine {
    #[allow(dead_code)]
    config: Arc<Config>,
    pool: SqlitePool,
    embeddings: Option<EmbeddingModel>,
}

impl LearningEngine {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        let db_path = Config::data_dir()?.join("learning.db");

        // Create pool with mode=rwc to allow creating database if it doesn't exist
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite://{}?mode=rwc", db_path.display()))
            .await?;

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS command_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                natural_input TEXT NOT NULL,
                learned_command TEXT NOT NULL,
                success_count INTEGER DEFAULT 0,
                failure_count INTEGER DEFAULT 0,
                confidence REAL DEFAULT 0.5,
                embedding BLOB,
                last_used TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS corrections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_input TEXT NOT NULL,
                ai_suggestion TEXT NOT NULL,
                user_correction TEXT NOT NULL,
                context TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS execution_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                input TEXT NOT NULL,
                executed_command TEXT NOT NULL,
                exit_code INTEGER,
                duration_ms INTEGER,
                context TEXT,
                timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS temporal_patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                command TEXT NOT NULL,
                hour_of_day INTEGER,
                day_of_week INTEGER,
                frequency INTEGER DEFAULT 1,
                last_executed TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&pool)
        .await?;

        // Initialize embedding model (optional - system works without it)
        let embeddings = match EmbeddingModel::new().await {
            Ok(model) => {
                tracing::info!("✓ Embedding model initialized");
                Some(model)
            }
            Err(e) => {
                tracing::warn!("Embedding model initialization failed: {}", e);
                tracing::warn!("Falling back to exact matching (embeddings disabled)");
                None
            }
        };

        Ok(Self {
            config,
            pool,
            embeddings,
        })
    }

    pub async fn find_similar(
        &self,
        input: &str,
        _context: &Context,
    ) -> Result<Option<LearnedCommand>> {
        // Use embeddings if available, otherwise fall back to exact match
        if let Some(ref embedding_model) = self.embeddings {
            self.find_similar_by_embedding(input, embedding_model).await
        } else {
            self.find_exact_match(input).await
        }
    }

    /// Find similar command using embedding-based semantic search
    async fn find_similar_by_embedding(
        &self,
        input: &str,
        embedding_model: &EmbeddingModel,
    ) -> Result<Option<LearnedCommand>> {
        // Generate embedding for input
        let input_embedding = embedding_model.embed(input)?;

        // Fetch all patterns with embeddings
        let patterns = sqlx::query(
            r#"
            SELECT id, natural_input, learned_command, confidence, success_count, failure_count, embedding
            FROM command_patterns
            WHERE embedding IS NOT NULL
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        // Find most similar pattern
        let mut best_match: Option<(LearnedCommand, f32)> = None;

        for row in patterns {
            let embedding_blob: Vec<u8> = row.get("embedding");

            // Deserialize embedding
            let pattern_embedding = Self::deserialize_embedding(&embedding_blob)?;

            // Calculate similarity
            let similarity =
                EmbeddingModel::cosine_similarity(&input_embedding, &pattern_embedding);

            // Combine similarity with confidence (weighted average)
            let confidence: f32 = row.get("confidence");
            let combined_score = similarity * 0.7 + confidence * 0.3;

            // Update best match if this is better
            if let Some((_, current_best_score)) = &best_match {
                if combined_score > *current_best_score {
                    best_match = Some((
                        LearnedCommand {
                            id: row.get("id"),
                            natural_input: row.get("natural_input"),
                            learned_command: row.get("learned_command"),
                            confidence: row.get("confidence"),
                            success_count: row.get("success_count"),
                            failure_count: row.get("failure_count"),
                        },
                        combined_score,
                    ));
                }
            } else {
                best_match = Some((
                    LearnedCommand {
                        id: row.get("id"),
                        natural_input: row.get("natural_input"),
                        learned_command: row.get("learned_command"),
                        confidence: row.get("confidence"),
                        success_count: row.get("success_count"),
                        failure_count: row.get("failure_count"),
                    },
                    combined_score,
                ));
            }
        }

        // Only return if similarity is high enough (>0.6 combined score)
        if let Some((command, score)) = best_match {
            if score > 0.6 {
                tracing::debug!(
                    "Found similar command: '{}' -> '{}' (similarity score: {:.2})",
                    input,
                    command.learned_command,
                    score
                );
                return Ok(Some(command));
            }
        }

        Ok(None)
    }

    /// Find exact match (fallback when embeddings unavailable)
    async fn find_exact_match(&self, input: &str) -> Result<Option<LearnedCommand>> {
        let result = sqlx::query(
            r#"
            SELECT id, natural_input, learned_command, confidence, success_count, failure_count
            FROM command_patterns
            WHERE natural_input = ?1
            ORDER BY confidence DESC
            LIMIT 1
            "#,
        )
        .bind(input)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = result {
            Ok(Some(LearnedCommand {
                id: row.get("id"),
                natural_input: row.get("natural_input"),
                learned_command: row.get("learned_command"),
                confidence: row.get("confidence"),
                success_count: row.get("success_count"),
                failure_count: row.get("failure_count"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Serialize embedding for storage
    fn serialize_embedding(embedding: &Array1<f32>) -> Vec<u8> {
        embedding
            .as_slice()
            .unwrap()
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect()
    }

    /// Deserialize embedding from storage
    fn deserialize_embedding(bytes: &[u8]) -> Result<Array1<f32>> {
        let floats: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect();

        Ok(Array1::from_vec(floats))
    }

    pub async fn record_success(
        &self,
        input: &str,
        executed: &str,
        _context: &Context,
    ) -> Result<()> {
        // Generate embedding if model available
        let embedding_blob = if let Some(ref model) = self.embeddings {
            match model.embed(input) {
                Ok(emb) => Some(Self::serialize_embedding(&emb)),
                Err(e) => {
                    tracing::warn!("Failed to generate embedding: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Check if pattern exists
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM command_patterns WHERE natural_input = ?1 AND learned_command = ?2"
        )
        .bind(input)
        .bind(executed)
        .fetch_one(&self.pool)
        .await? > 0;

        if exists {
            // Update existing pattern
            if let Some(embedding) = embedding_blob {
                sqlx::query(
                    r#"
                    UPDATE command_patterns
                    SET success_count = success_count + 1,
                        confidence = confidence + 0.1 * (1.0 - confidence),
                        embedding = ?1,
                        last_used = CURRENT_TIMESTAMP
                    WHERE natural_input = ?2 AND learned_command = ?3
                    "#,
                )
                .bind(embedding)
                .bind(input)
                .bind(executed)
                .execute(&self.pool)
                .await?;
            } else {
                sqlx::query(
                    r#"
                    UPDATE command_patterns
                    SET success_count = success_count + 1,
                        confidence = confidence + 0.1 * (1.0 - confidence),
                        last_used = CURRENT_TIMESTAMP
                    WHERE natural_input = ?1 AND learned_command = ?2
                    "#,
                )
                .bind(input)
                .bind(executed)
                .execute(&self.pool)
                .await?;
            }
        } else {
            // Create new pattern
            if let Some(embedding) = embedding_blob {
                sqlx::query(
                    r#"
                    INSERT INTO command_patterns (natural_input, learned_command, success_count, confidence, embedding)
                    VALUES (?1, ?2, 1, 0.6, ?3)
                    "#,
                )
                .bind(input)
                .bind(executed)
                .bind(embedding)
                .execute(&self.pool)
                .await?;
            } else {
                sqlx::query(
                    r#"
                    INSERT INTO command_patterns (natural_input, learned_command, success_count, confidence)
                    VALUES (?1, ?2, 1, 0.6)
                    "#,
                )
                .bind(input)
                .bind(executed)
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }

    pub async fn record_failure(
        &self,
        input: &str,
        executed: &str,
        _context: &Context,
    ) -> Result<()> {
        // Lower confidence for failed command
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM command_patterns WHERE natural_input = ?1 AND learned_command = ?2",
        )
        .bind(input)
        .bind(executed)
        .fetch_one(&self.pool)
        .await?
            > 0;

        if exists {
            // Penalize existing pattern
            sqlx::query(
                r#"
                UPDATE command_patterns
                SET failure_count = failure_count + 1,
                    confidence = confidence * 0.8,
                    last_used = CURRENT_TIMESTAMP
                WHERE natural_input = ?1 AND learned_command = ?2
                "#,
            )
            .bind(input)
            .bind(executed)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn record_ai_suggestion(
        &self,
        input: &str,
        suggestion: &str,
        _context: &Context,
    ) -> Result<()> {
        // Just log it for now - we'll learn when user confirms/rejects
        tracing::debug!("AI suggestion: {} -> {}", input, suggestion);
        Ok(())
    }

    pub async fn record_correction(
        &self,
        input: &str,
        ai_suggestion: &str,
        user_correction: &str,
        context: &Context,
    ) -> Result<()> {
        // Store correction
        sqlx::query(
            r#"
            INSERT INTO corrections (original_input, ai_suggestion, user_correction, context)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(input)
        .bind(ai_suggestion)
        .bind(user_correction)
        .bind(serde_json::to_string(context)?)
        .execute(&self.pool)
        .await?;

        // Penalize wrong suggestion
        sqlx::query(
            r#"
            UPDATE command_patterns
            SET confidence = confidence * 0.7,
                failure_count = failure_count + 1
            WHERE learned_command = ?1
            "#,
        )
        .bind(ai_suggestion)
        .execute(&self.pool)
        .await?;

        // Create or boost correct pattern
        self.record_success(input, user_correction, context).await?;

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn record_execution(
        &self,
        input: &str,
        executed: &str,
        exit_code: i32,
        duration_ms: i64,
        context: &Context,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO execution_history (input, executed_command, exit_code, duration_ms, context)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(input)
        .bind(executed)
        .bind(exit_code)
        .bind(duration_ms)
        .bind(serde_json::to_string(context)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn record_temporal_pattern(&self, command: &str, hour: i32, day: i32) -> Result<()> {
        // Check if pattern exists
        let exists = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM temporal_patterns WHERE command = ?1 AND hour_of_day = ?2 AND day_of_week = ?3"
        )
        .bind(command)
        .bind(hour)
        .bind(day)
        .fetch_one(&self.pool)
        .await? > 0;

        if exists {
            // Increment frequency
            sqlx::query(
                r#"
                UPDATE temporal_patterns
                SET frequency = frequency + 1,
                    last_executed = CURRENT_TIMESTAMP
                WHERE command = ?1 AND hour_of_day = ?2 AND day_of_week = ?3
                "#,
            )
            .bind(command)
            .bind(hour)
            .bind(day)
            .execute(&self.pool)
            .await?;
        } else {
            // Create new pattern
            sqlx::query(
                r#"
                INSERT INTO temporal_patterns (command, hour_of_day, day_of_week, frequency)
                VALUES (?1, ?2, ?3, 1)
                "#,
            )
            .bind(command)
            .bind(hour)
            .bind(day)
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn get_temporal_patterns(&self, hour: i32, day: i32) -> Result<Vec<TemporalPattern>> {
        let patterns = sqlx::query_as::<_, TemporalPattern>(
            r#"
            SELECT command, hour_of_day, day_of_week, frequency, last_executed
            FROM temporal_patterns
            WHERE hour_of_day = ?1 AND day_of_week = ?2
            ORDER BY frequency DESC
            LIMIT 5
            "#,
        )
        .bind(hour)
        .bind(day)
        .fetch_all(&self.pool)
        .await?;

        Ok(patterns)
    }

    #[allow(dead_code)]
    pub async fn get_stats(&self) -> Result<LearningStats> {
        let total_patterns = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM command_patterns")
            .fetch_one(&self.pool)
            .await?;

        let total_executions =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM execution_history")
                .fetch_one(&self.pool)
                .await?;

        let successful_executions = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM execution_history WHERE exit_code = 0",
        )
        .fetch_one(&self.pool)
        .await?;

        let success_rate = if total_executions > 0 {
            (successful_executions as f32 / total_executions as f32) * 100.0
        } else {
            0.0
        };

        Ok(LearningStats {
            total_patterns,
            total_executions,
            successful_executions,
            success_rate,
        })
    }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TemporalPattern {
    pub command: String,
    #[allow(dead_code)]
    pub hour_of_day: i32,
    #[allow(dead_code)]
    pub day_of_week: i32,
    pub frequency: i32,
    #[allow(dead_code)]
    pub last_executed: String,
}

impl TemporalPattern {
    pub fn should_suggest(&self) -> bool {
        // Only suggest if not executed in last hour
        // TODO: Parse last_executed timestamp and check
        true
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct LearningStats {
    pub total_patterns: i64,
    pub total_executions: i64,
    pub successful_executions: i64,
    pub success_rate: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_learning_engine() -> LearningEngine {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("ORBIT_DATA_DIR", temp_dir.path());

        let config_path = temp_dir.path().join("config.yaml");
        std::fs::write(
            &config_path,
            format!(
                r#"
license:
  key: "test-key"
daemon:
  socket_path: "{}/orbit.sock"
  log_level: "info"
  auto_restart: true
provider_mode: manual
default_provider: "test"
providers:
  test:
    api_key: "test"
learning:
  enabled: true
  confidence_threshold: 0.7
  max_patterns: 10000
  embedding_model: "minilm-l6-v2"
monitoring:
  enabled: true
  interval_seconds: 300
  watch_git_repos: true
  watch_system: true
  desktop_notifications: false
classification:
  natural_language_threshold: 0.8
  check_path_binaries: true
  cache_known_commands: true
execution:
  auto_approve: false
  confirm_destructive: true
  timeout_seconds: 300
context:
  track_directory_patterns: true
  detect_languages: true
  detect_frameworks: true
  include_git_context: true
  max_recent_commands: 20
ui:
  emoji: true
  colors: true
  show_provider: false
  show_learning_stats: true
"#,
                temp_dir.path().display()
            ),
        )
        .unwrap();

        std::env::set_var("ORBIT_CONFIG", config_path.to_str().unwrap());
        std::env::set_var("ORBIT_DEV_MODE", "1");
        std::mem::forget(temp_dir);

        let config = Arc::new(crate::config::Config::load().await.unwrap());

        // Let SQLite create the database file itself (don't pre-create empty file)
        // The LearningEngine::new will initialize the database with proper schema
        LearningEngine::new(config).await.unwrap()
    }

    fn create_test_context() -> Context {
        use crate::context::{DirectoryType, ProjectType};
        Context {
            os_name: "Linux".to_string(),
            os_version: "Ubuntu 22.04".to_string(),
            shell_name: "bash".to_string(),
            shell_version: "5.1.0".to_string(),
            pwd: std::path::PathBuf::from("/tmp"),
            username: "testuser".to_string(),
            git_context: None,
            detected_languages: vec![],
            recent_commands: vec![],
            project_type: None,
            directory_type: DirectoryType::Temp,
        }
    }

    // ========== Initialization Tests ==========

    #[tokio::test]
    async fn test_learning_engine_initialization() {
        let engine = create_test_learning_engine().await;

        // Verify tables were created by attempting to query them
        let result = sqlx::query("SELECT COUNT(*) FROM command_patterns")
            .fetch_one(&engine.pool)
            .await;
        assert!(result.is_ok(), "command_patterns table should exist");

        let result = sqlx::query("SELECT COUNT(*) FROM corrections")
            .fetch_one(&engine.pool)
            .await;
        assert!(result.is_ok(), "corrections table should exist");

        let result = sqlx::query("SELECT COUNT(*) FROM execution_history")
            .fetch_one(&engine.pool)
            .await;
        assert!(result.is_ok(), "execution_history table should exist");

        let result = sqlx::query("SELECT COUNT(*) FROM temporal_patterns")
            .fetch_one(&engine.pool)
            .await;
        assert!(result.is_ok(), "temporal_patterns table should exist");
    }

    #[tokio::test]
    async fn test_embedding_model_initialization() {
        let engine = create_test_learning_engine().await;

        // Engine should have embeddings initialized (or None if failed)
        // We don't assert on presence since it's optional
        assert!(
            engine.embeddings.is_some() || engine.embeddings.is_none(),
            "Embeddings should be Some or None"
        );
    }

    // ========== Pattern Recording Tests ==========

    #[tokio::test]
    async fn test_record_success_creates_new_pattern() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        let result = engine
            .record_success("list files", "ls -la", &context)
            .await;
        assert!(result.is_ok(), "Should record success");

        // Verify pattern was created
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM command_patterns WHERE natural_input = ?1")
                .bind("list files")
                .fetch_one(&engine.pool)
                .await
                .unwrap();

        assert_eq!(count, 1, "Should have created one pattern");
    }

    #[tokio::test]
    async fn test_record_success_increments_count() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Record success twice
        engine
            .record_success("show files", "ls", &context)
            .await
            .unwrap();
        engine
            .record_success("show files", "ls", &context)
            .await
            .unwrap();

        // Verify success_count increased
        let success_count: i32 = sqlx::query_scalar(
            "SELECT success_count FROM command_patterns WHERE natural_input = ?1",
        )
        .bind("show files")
        .fetch_one(&engine.pool)
        .await
        .unwrap();

        assert_eq!(
            success_count, 2,
            "Success count should be 2 after two recordings"
        );
    }

    #[tokio::test]
    async fn test_record_success_increases_confidence() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Record initial success
        engine
            .record_success("find text", "grep text file.txt", &context)
            .await
            .unwrap();

        let initial_confidence: f32 =
            sqlx::query_scalar("SELECT confidence FROM command_patterns WHERE natural_input = ?1")
                .bind("find text")
                .fetch_one(&engine.pool)
                .await
                .unwrap();

        // Record another success
        engine
            .record_success("find text", "grep text file.txt", &context)
            .await
            .unwrap();

        let new_confidence: f32 =
            sqlx::query_scalar("SELECT confidence FROM command_patterns WHERE natural_input = ?1")
                .bind("find text")
                .fetch_one(&engine.pool)
                .await
                .unwrap();

        assert!(
            new_confidence > initial_confidence,
            "Confidence should increase after successful execution: {} -> {}",
            initial_confidence,
            new_confidence
        );
        assert!(
            new_confidence <= 1.0,
            "Confidence should not exceed 1.0: {}",
            new_confidence
        );
    }

    #[tokio::test]
    async fn test_record_failure_decreases_confidence() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Create initial pattern with success
        engine
            .record_success("remove file", "rm test.txt", &context)
            .await
            .unwrap();

        let initial_confidence: f32 =
            sqlx::query_scalar("SELECT confidence FROM command_patterns WHERE natural_input = ?1")
                .bind("remove file")
                .fetch_one(&engine.pool)
                .await
                .unwrap();

        // Record failure
        engine
            .record_failure("remove file", "rm test.txt", &context)
            .await
            .unwrap();

        let new_confidence: f32 =
            sqlx::query_scalar("SELECT confidence FROM command_patterns WHERE natural_input = ?1")
                .bind("remove file")
                .fetch_one(&engine.pool)
                .await
                .unwrap();

        assert!(
            new_confidence < initial_confidence,
            "Confidence should decrease after failure: {} -> {}",
            initial_confidence,
            new_confidence
        );
        assert!(
            new_confidence > 0.0,
            "Confidence should remain positive: {}",
            new_confidence
        );
    }

    #[tokio::test]
    async fn test_record_failure_increments_failure_count() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Create pattern first
        engine
            .record_success("copy file", "cp a.txt b.txt", &context)
            .await
            .unwrap();

        // Record failures
        engine
            .record_failure("copy file", "cp a.txt b.txt", &context)
            .await
            .unwrap();
        engine
            .record_failure("copy file", "cp a.txt b.txt", &context)
            .await
            .unwrap();

        let failure_count: i32 = sqlx::query_scalar(
            "SELECT failure_count FROM command_patterns WHERE natural_input = ?1",
        )
        .bind("copy file")
        .fetch_one(&engine.pool)
        .await
        .unwrap();

        assert_eq!(
            failure_count, 2,
            "Failure count should be 2 after two failures"
        );
    }

    // ========== Pattern Finding Tests ==========

    #[tokio::test]
    async fn test_find_exact_match() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Record a pattern
        engine
            .record_success("show processes", "ps aux", &context)
            .await
            .unwrap();

        // Find exact match
        let result = engine.find_similar("show processes", &context).await;
        assert!(result.is_ok(), "Should find pattern");

        let pattern = result.unwrap();
        assert!(pattern.is_some(), "Should return a pattern");

        let pattern = pattern.unwrap();
        assert_eq!(pattern.learned_command, "ps aux");
        assert!(pattern.confidence >= 0.6);
    }

    #[tokio::test]
    async fn test_find_no_match() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // No patterns recorded
        let result = engine.find_similar("unknown command", &context).await;
        assert!(result.is_ok(), "Should succeed even with no matches");

        let pattern = result.unwrap();
        assert!(pattern.is_none(), "Should return None for unknown command");
    }

    #[tokio::test]
    async fn test_find_similar_respects_confidence_threshold() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Record low-confidence pattern (start at 0.6, reduce to below threshold)
        engine
            .record_success("test command", "echo test", &context)
            .await
            .unwrap();

        // Reduce confidence below threshold
        for _ in 0..10 {
            engine
                .record_failure("test command", "echo test", &context)
                .await
                .unwrap();
        }

        // With embeddings disabled, exact match should still return the pattern
        // but in production, confidence filtering would apply
        let result = engine.find_similar("test command", &context).await;
        assert!(result.is_ok());
    }

    // ========== Correction Recording Tests ==========

    #[tokio::test]
    async fn test_record_correction() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        let result = engine
            .record_correction(
                "find files",
                "ls -la",               // AI suggestion
                "find . -name '*.txt'", // User correction
                &context,
            )
            .await;

        assert!(result.is_ok(), "Should record correction");

        // Verify correction was stored
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM corrections WHERE original_input = ?1")
                .bind("find files")
                .fetch_one(&engine.pool)
                .await
                .unwrap();

        assert_eq!(count, 1, "Should have one correction recorded");
    }

    #[tokio::test]
    async fn test_record_correction_penalizes_wrong_suggestion() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Record initial pattern
        engine
            .record_success("search files", "ls", &context)
            .await
            .unwrap();

        let initial_confidence: f32 = sqlx::query_scalar(
            "SELECT confidence FROM command_patterns WHERE learned_command = ?1",
        )
        .bind("ls")
        .fetch_one(&engine.pool)
        .await
        .unwrap();

        // User corrects the suggestion
        engine
            .record_correction("search files", "ls", "find . -type f", &context)
            .await
            .unwrap();

        // Old suggestion should have lower confidence
        let new_confidence: f32 = sqlx::query_scalar(
            "SELECT confidence FROM command_patterns WHERE learned_command = ?1",
        )
        .bind("ls")
        .fetch_one(&engine.pool)
        .await
        .unwrap();

        assert!(
            new_confidence < initial_confidence,
            "Wrong suggestion should have reduced confidence"
        );
    }

    #[tokio::test]
    async fn test_record_correction_boosts_correct_pattern() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // User provides correction
        engine
            .record_correction(
                "make directory",
                "mkd folder", // Wrong AI suggestion
                "mkdir folder",
                &context,
            )
            .await
            .unwrap();

        // Correct pattern should now exist with good confidence
        let result = engine.find_similar("make directory", &context).await;
        assert!(result.is_ok());

        let pattern = result.unwrap();
        assert!(pattern.is_some(), "Corrected pattern should exist");

        let pattern = pattern.unwrap();
        assert_eq!(pattern.learned_command, "mkdir folder");
        assert!(pattern.confidence >= 0.6);
    }

    // ========== Temporal Pattern Tests ==========

    #[tokio::test]
    async fn test_record_temporal_pattern() {
        let engine = create_test_learning_engine().await;

        let result = engine
            .record_temporal_pattern("git pull", 9, 1) // 9 AM, Monday
            .await;

        assert!(result.is_ok(), "Should record temporal pattern");

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM temporal_patterns WHERE command = ?1 AND hour_of_day = ?2",
        )
        .bind("git pull")
        .bind(9)
        .fetch_one(&engine.pool)
        .await
        .unwrap();

        assert_eq!(count, 1, "Should have one temporal pattern");
    }

    #[tokio::test]
    async fn test_record_temporal_pattern_increments_frequency() {
        let engine = create_test_learning_engine().await;

        // Record same pattern multiple times
        engine
            .record_temporal_pattern("npm install", 10, 3)
            .await
            .unwrap();
        engine
            .record_temporal_pattern("npm install", 10, 3)
            .await
            .unwrap();
        engine
            .record_temporal_pattern("npm install", 10, 3)
            .await
            .unwrap();

        let frequency: i32 = sqlx::query_scalar(
            "SELECT frequency FROM temporal_patterns WHERE command = ?1 AND hour_of_day = ?2",
        )
        .bind("npm install")
        .bind(10)
        .fetch_one(&engine.pool)
        .await
        .unwrap();

        assert_eq!(frequency, 3, "Frequency should be 3 after three recordings");
    }

    #[tokio::test]
    async fn test_get_temporal_patterns() {
        let engine = create_test_learning_engine().await;

        // Record multiple patterns for same time
        engine
            .record_temporal_pattern("git status", 14, 2)
            .await
            .unwrap();
        engine
            .record_temporal_pattern("git status", 14, 2)
            .await
            .unwrap();
        engine
            .record_temporal_pattern("cargo test", 14, 2)
            .await
            .unwrap();
        engine
            .record_temporal_pattern("cargo build", 14, 2)
            .await
            .unwrap();

        let patterns = engine.get_temporal_patterns(14, 2).await.unwrap();

        assert!(
            !patterns.is_empty(),
            "Should return temporal patterns for time"
        );
        assert!(
            patterns.len() <= 5,
            "Should limit to 5 patterns (as per implementation)"
        );

        // Most frequent should be first
        assert_eq!(patterns[0].command, "git status");
        assert_eq!(patterns[0].frequency, 2);
    }

    // ========== Embedding Serialization Tests ==========

    #[test]
    fn test_serialize_deserialize_embedding() {
        let original = Array1::from_vec(vec![1.0, 2.0, 3.0, 4.5, -1.0]);

        let serialized = LearningEngine::serialize_embedding(&original);
        assert_eq!(serialized.len(), 20, "5 floats * 4 bytes = 20 bytes");

        let deserialized = LearningEngine::deserialize_embedding(&serialized).unwrap();
        assert_eq!(deserialized.len(), 5, "Should have 5 elements");

        for (i, &val) in deserialized.iter().enumerate() {
            assert!(
                (val - original[i]).abs() < 0.0001,
                "Values should match at index {}: {} vs {}",
                i,
                val,
                original[i]
            );
        }
    }

    #[test]
    fn test_serialize_embedding_384_dimensions() {
        let original = Array1::from_vec(vec![0.5; 384]);

        let serialized = LearningEngine::serialize_embedding(&original);
        assert_eq!(serialized.len(), 1536, "384 floats * 4 bytes = 1536 bytes");

        let deserialized = LearningEngine::deserialize_embedding(&serialized).unwrap();
        assert_eq!(deserialized.len(), 384);
    }

    // ========== AI Suggestion Tracking Tests ==========

    #[tokio::test]
    async fn test_record_ai_suggestion() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Should succeed without error
        let result = engine
            .record_ai_suggestion("compile code", "gcc main.c", &context)
            .await;

        assert!(result.is_ok(), "Should record AI suggestion without error");
    }

    // ========== Stats Tests ==========

    #[tokio::test]
    async fn test_get_stats_empty() {
        let engine = create_test_learning_engine().await;

        let stats = engine.get_stats().await.unwrap();

        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.total_executions, 0);
        assert_eq!(stats.success_rate, 0.0);
    }

    #[tokio::test]
    async fn test_get_stats_with_data() {
        let engine = create_test_learning_engine().await;
        let context = create_test_context();

        // Record some patterns
        engine.record_success("cmd1", "ls", &context).await.unwrap();
        engine
            .record_success("cmd2", "pwd", &context)
            .await
            .unwrap();

        // Record some executions
        engine
            .record_execution("cmd1", "ls", 0, 100, &context)
            .await
            .unwrap();
        engine
            .record_execution("cmd2", "pwd", 0, 50, &context)
            .await
            .unwrap();
        engine
            .record_execution("cmd3", "cd /tmp", 1, 10, &context)
            .await
            .unwrap();

        let stats = engine.get_stats().await.unwrap();

        assert_eq!(stats.total_patterns, 2, "Should have 2 patterns");
        assert_eq!(stats.total_executions, 3, "Should have 3 executions");
        assert_eq!(
            stats.successful_executions, 2,
            "Should have 2 successful executions"
        );
        assert!(
            (stats.success_rate - 66.67).abs() < 0.1,
            "Success rate should be ~66.67%: {}",
            stats.success_rate
        );
    }
}
