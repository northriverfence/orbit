use anyhow::Result;
use sqlx::SqlitePool;

use super::types::*;

/// Analytics service for command execution tracking
pub struct AnalyticsService {
    db: SqlitePool,
}

impl AnalyticsService {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Record command execution
    pub async fn record(&self, execution: CommandExecution) -> Result<()> {
        let timestamp = chrono::Utc::now().timestamp();
        let result_str = match execution.result {
            ExecutionResult::Success => "success",
            ExecutionResult::Failed => "failed",
            ExecutionResult::Rejected => "rejected",
            ExecutionResult::Edited => "edited",
        };

        // Generate context hash for grouping related commands
        let context_hash = self.generate_context_hash(&execution.context);

        sqlx::query(
            r#"
            INSERT INTO command_analytics (
                original_input, suggested_command, executed_command, result,
                execution_time_ms, exit_code, timestamp, context_hash, provider, cwd, shell
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&execution.original_input)
        .bind(&execution.suggested_command)
        .bind(&execution.executed_command)
        .bind(result_str)
        .bind(execution.execution_time_ms)
        .bind(execution.exit_code)
        .bind(timestamp)
        .bind(&context_hash)
        .bind(&execution.provider)
        .bind(&execution.context.cwd)
        .bind(&execution.context.shell)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get analytics summary for the last N days
    pub async fn get_summary(&self, days: u32) -> Result<AnalyticsSummary> {
        let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 86400);

        // Total commands
        let total_commands: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM command_analytics WHERE timestamp >= ?1"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // AI suggestions (where suggested_command is not null)
        let ai_suggestions: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM command_analytics WHERE timestamp >= ?1 AND suggested_command IS NOT NULL"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // Accepted (success results)
        let accepted: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM command_analytics WHERE timestamp >= ?1 AND result = 'success'"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // Rejected
        let rejected: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM command_analytics WHERE timestamp >= ?1 AND result = 'rejected'"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // Success rate
        let success_rate = if total_commands > 0 {
            (accepted as f64 / total_commands as f64) * 100.0
        } else {
            0.0
        };

        // Top patterns
        let top_patterns = self.get_top_patterns_internal(10).await?;

        // Recent insights
        let insights = self.get_recent_insights(5).await?;

        Ok(AnalyticsSummary {
            total_commands,
            ai_suggestions,
            accepted,
            rejected,
            success_rate,
            top_patterns,
            insights,
        })
    }

    /// Get top patterns
    async fn get_top_patterns_internal(&self, limit: usize) -> Result<Vec<Pattern>> {
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

    /// Get insights
    pub async fn get_insights(&self) -> Result<Vec<Insight>> {
        self.get_recent_insights(10).await
    }

    /// Get recent insights
    async fn get_recent_insights(&self, limit: usize) -> Result<Vec<Insight>> {
        let rows = sqlx::query_as::<_, (String, String, f64, i64)>(
            r#"
            SELECT category, insight, confidence, created_at
            FROM insights
            WHERE acknowledged = 0
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )
        .bind(limit as i64)
        .fetch_all(&self.db)
        .await?;

        Ok(rows
            .into_iter()
            .map(|(cat_str, insight, confidence, created_at)| {
                let category = match cat_str.as_str() {
                    "usage" => InsightCategory::Usage,
                    "preference" => InsightCategory::Preference,
                    "pattern" => InsightCategory::Pattern,
                    "error" => InsightCategory::Error,
                    "optimization" => InsightCategory::Optimization,
                    _ => InsightCategory::Usage,
                };

                Insight {
                    category,
                    insight,
                    confidence,
                    created_at,
                }
            })
            .collect())
    }

    /// Generate insights based on analytics data
    pub async fn generate_insights(&self) -> Result<()> {
        // Find common error patterns
        let failing_commands: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT executed_command, COUNT(*) as count
            FROM command_analytics
            WHERE result = 'failed' AND timestamp >= ?1
            GROUP BY executed_command
            HAVING count >= 3
            ORDER BY count DESC
            LIMIT 5
            "#,
        )
        .bind(chrono::Utc::now().timestamp() - 86400 * 7) // Last 7 days
        .fetch_all(&self.db)
        .await?;

        for (cmd, count) in failing_commands {
            let insight = format!(
                "Command '{}' has failed {} times recently. Consider reviewing its usage.",
                cmd, count
            );
            self.add_insight(InsightCategory::Error, &insight, 0.8)
                .await?;
        }

        // Detect preferred patterns
        let preferred_patterns: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT executed_command, COUNT(*) as count
            FROM command_analytics
            WHERE result = 'success' AND timestamp >= ?1
            GROUP BY executed_command
            HAVING count >= 10
            ORDER BY count DESC
            LIMIT 3
            "#,
        )
        .bind(chrono::Utc::now().timestamp() - 86400 * 30) // Last 30 days
        .fetch_all(&self.db)
        .await?;

        for (cmd, count) in preferred_patterns {
            let insight = format!(
                "You frequently use '{}' ({} times). This has been added to your preferred patterns.",
                cmd, count
            );
            self.add_insight(InsightCategory::Pattern, &insight, 0.9)
                .await?;
        }

        Ok(())
    }

    /// Add an insight
    async fn add_insight(
        &self,
        category: InsightCategory,
        insight: &str,
        confidence: f64,
    ) -> Result<()> {
        let category_str = match category {
            InsightCategory::Usage => "usage",
            InsightCategory::Preference => "preference",
            InsightCategory::Pattern => "pattern",
            InsightCategory::Error => "error",
            InsightCategory::Optimization => "optimization",
        };

        let timestamp = chrono::Utc::now().timestamp();
        let relevant_until = timestamp + (86400 * 30); // 30 days

        sqlx::query(
            r#"
            INSERT INTO insights (category, insight, confidence, created_at, relevant_until)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
        )
        .bind(category_str)
        .bind(insight)
        .bind(confidence)
        .bind(timestamp)
        .bind(relevant_until)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Clean up old analytics data
    pub async fn cleanup_old_data(&self, keep_days: u32) -> Result<()> {
        let cutoff = chrono::Utc::now().timestamp() - (keep_days as i64 * 86400);

        // Delete old analytics
        sqlx::query("DELETE FROM command_analytics WHERE timestamp < ?1")
            .bind(cutoff)
            .execute(&self.db)
            .await?;

        // Delete old insights
        sqlx::query("DELETE FROM insights WHERE relevant_until < ?1")
            .bind(chrono::Utc::now().timestamp())
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Generate context hash for grouping
    fn generate_context_hash(&self, context: &CommandContext) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        context.cwd.hash(&mut hasher);
        context.git_repo.hash(&mut hasher);
        context.project_type.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}
