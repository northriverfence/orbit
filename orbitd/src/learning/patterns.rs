use anyhow::Result;
use sqlx::SqlitePool;
use std::collections::HashMap;

use super::types::*;

/// Pattern recognition and suggestion ranking
pub struct PatternRecognition {
    db: SqlitePool,
    pattern_cache: HashMap<String, CachedPattern>,
}

struct CachedPattern {
    pattern: Pattern,
    cached_at: i64,
}

impl PatternRecognition {
    pub async fn new(db: SqlitePool) -> Result<Self> {
        Ok(Self {
            db,
            pattern_cache: HashMap::new(),
        })
    }

    /// Update pattern based on execution result
    pub async fn update_pattern(&mut self, execution: &CommandExecution) -> Result<()> {
        // Check if pattern exists
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM command_patterns WHERE pattern = ?1)"
        )
        .bind(&execution.original_input)
        .fetch_one(&self.db)
        .await?;

        if exists {
            // Update existing pattern
            match execution.result {
                ExecutionResult::Success => {
                    sqlx::query(
                        r#"
                        UPDATE command_patterns
                        SET success_count = success_count + 1,
                            frequency = frequency + 1,
                            last_used = ?1
                        WHERE pattern = ?2
                        "#,
                    )
                    .bind(chrono::Utc::now().timestamp())
                    .bind(&execution.original_input)
                    .execute(&self.db)
                    .await?;
                }
                ExecutionResult::Failed => {
                    sqlx::query(
                        r#"
                        UPDATE command_patterns
                        SET failure_count = failure_count + 1,
                            frequency = frequency + 1,
                            last_used = ?1
                        WHERE pattern = ?2
                        "#,
                    )
                    .bind(chrono::Utc::now().timestamp())
                    .bind(&execution.original_input)
                    .execute(&self.db)
                    .await?;
                }
                _ => {}
            }
        } else if matches!(execution.result, ExecutionResult::Success) {
            // Create new pattern on first success
            sqlx::query(
                r#"
                INSERT INTO command_patterns (pattern, frequency, success_count, preferred_translation)
                VALUES (?1, 1, 1, ?2)
                "#,
            )
            .bind(&execution.original_input)
            .bind(&execution.executed_command)
            .execute(&self.db)
            .await?;
        }

        // Invalidate cache
        self.pattern_cache.remove(&execution.original_input);

        Ok(())
    }

    /// Rank suggestions based on learning data
    pub async fn rank_suggestions(
        &self,
        input: &str,
        suggestions: Vec<String>,
        context: &CommandContext,
    ) -> Result<Vec<RankedSuggestion>> {
        let mut ranked = Vec::new();

        for suggestion in suggestions {
            let score = self.calculate_score(input, &suggestion, context).await?;
            let reasons = self.generate_reasons(input, &suggestion, score).await?;

            ranked.push(RankedSuggestion {
                command: suggestion,
                score,
                reasons,
            });
        }

        // Sort by score descending
        ranked.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        Ok(ranked)
    }

    /// Calculate score for a suggestion
    async fn calculate_score(
        &self,
        input: &str,
        suggestion: &str,
        _context: &CommandContext,
    ) -> Result<f64> {
        // Base score
        let mut score = 0.5;

        // Check if this exact pattern was successful before
        let pattern_data: Option<(i64, i64, i64)> = sqlx::query_as(
            "SELECT frequency, success_count, failure_count FROM command_patterns WHERE pattern = ?1"
        )
        .bind(input)
        .fetch_optional(&self.db)
        .await?;

        if let Some((freq, success, failure)) = pattern_data {
            // Frequency bonus (max 0.2)
            score += (freq as f64 * 0.01).min(0.2);

            // Success rate bonus (max 0.3)
            let total = success + failure;
            if total > 0 {
                let success_rate = success as f64 / total as f64;
                score += success_rate * 0.3;
            }
        }

        // Check if this specific suggestion was executed before
        let suggestion_data: Option<i64> = sqlx::query_scalar(
            "SELECT COUNT(*) FROM command_analytics WHERE executed_command = ?1 AND result = 'success'"
        )
        .bind(suggestion)
        .fetch_optional(&self.db)
        .await?;

        if let Some(count) = suggestion_data {
            if count > 0 {
                score += 0.2; // Bonus for previously executed successfully
            }
        }

        Ok(score.min(1.0))
    }

    /// Generate reasons for the score
    async fn generate_reasons(
        &self,
        input: &str,
        suggestion: &str,
        score: f64,
    ) -> Result<Vec<String>> {
        let mut reasons = Vec::new();

        // Check pattern history
        let pattern_data: Option<(i64, i64, i64)> = sqlx::query_as(
            "SELECT frequency, success_count, failure_count FROM command_patterns WHERE pattern = ?1"
        )
        .bind(input)
        .fetch_optional(&self.db)
        .await?;

        if let Some((freq, success, _failure)) = pattern_data {
            if freq > 5 {
                reasons.push(format!("Frequently used ({} times)", freq));
            }

            let total = success + _failure;
            if total > 0 {
                let success_rate = (success as f64 / total as f64) * 100.0;
                if success_rate > 80.0 {
                    reasons.push(format!("High success rate ({:.0}%)", success_rate));
                }
            }
        }

        // Check suggestion execution history
        let exec_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM command_analytics WHERE executed_command = ?1 AND result = 'success'"
        )
        .bind(suggestion)
        .fetch_one(&self.db)
        .await?;

        if exec_count > 0 {
            reasons.push("Previously executed successfully".to_string());
        }

        if score > 0.8 {
            reasons.push("Highly recommended".to_string());
        }

        if reasons.is_empty() {
            reasons.push("AI suggestion".to_string());
        }

        Ok(reasons)
    }

    /// Get top patterns
    pub async fn get_top_patterns(&self, limit: usize) -> Result<Vec<Pattern>> {
        let rows = sqlx::query_as::<_, (String, i64, i64, i64, Option<String>, i64)>(
            r#"
            SELECT pattern, frequency, success_count, failure_count, preferred_translation, last_used
            FROM command_patterns
            ORDER BY frequency DESC
            LIMIT ?1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(pattern, freq, success, failure, trans, last_used)| {
                let total = success + failure;
                let success_rate = if total > 0 {
                    success as f64 / total as f64
                } else {
                    0.0
                };

                Pattern {
                    pattern,
                    frequency: freq,
                    success_rate,
                    preferred_translation: trans,
                    last_used,
                }
            })
            .collect())
    }

    /// Detect new patterns from analytics data
    pub async fn detect_new_patterns(&mut self) -> Result<()> {
        // Find commands executed multiple times with same input
        let candidates: Vec<(String, String, i64)> = sqlx::query_as(
            r#"
            SELECT original_input, executed_command, COUNT(*) as count
            FROM command_analytics
            WHERE timestamp >= ?1 AND result = 'success'
            GROUP BY original_input, executed_command
            HAVING count >= 3
            "#,
        )
        .bind(chrono::Utc::now().timestamp() - 86400 * 7) // Last 7 days
        .fetch_all(&self.db)
        .await?;

        for (input, cmd, count) in candidates {
            // Check if pattern already exists
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM command_patterns WHERE pattern = ?1)"
            )
            .bind(&input)
            .fetch_one(&self.db)
            .await?;

            if !exists {
                // Create new pattern
                sqlx::query(
                    r#"
                    INSERT INTO command_patterns (pattern, frequency, success_count, preferred_translation)
                    VALUES (?1, ?2, ?2, ?3)
                    "#,
                )
                .bind(&input)
                .bind(count)
                .bind(&cmd)
                .execute(&self.db)
                .await?;

                tracing::info!("Detected new pattern: '{}' -> '{}'", input, cmd);
            }
        }

        Ok(())
    }

    /// Export patterns
    pub async fn export_patterns(&self) -> Result<Vec<Pattern>> {
        self.get_top_patterns(1000).await
    }

    /// Import patterns
    pub async fn import_patterns(&mut self, patterns: Vec<Pattern>) -> Result<()> {
        for pattern in patterns {
            sqlx::query(
                r#"
                INSERT INTO command_patterns (pattern, frequency, success_count, failure_count, preferred_translation, last_used)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                ON CONFLICT(pattern) DO UPDATE SET
                    frequency = frequency + excluded.frequency,
                    success_count = success_count + excluded.success_count,
                    failure_count = failure_count + excluded.failure_count
                "#,
            )
            .bind(&pattern.pattern)
            .bind(pattern.frequency)
            .bind((pattern.success_rate * pattern.frequency as f64) as i64)
            .bind(((1.0 - pattern.success_rate) * pattern.frequency as f64) as i64)
            .bind(&pattern.preferred_translation)
            .bind(pattern.last_used)
            .execute(&self.db)
            .await?;
        }

        // Clear cache after import
        self.pattern_cache.clear();

        Ok(())
    }
}
