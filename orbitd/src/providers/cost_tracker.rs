use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::collections::HashMap;

/// Cost tracking service
pub struct CostTracker {
    db: SqlitePool,
}

impl CostTracker {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Record provider usage
    pub async fn record_usage(
        &self,
        provider: &str,
        model: &str,
        tokens: u32,
        cost: f64,
        success: bool,
        user_input: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO provider_usage (
                provider_name, model, timestamp, tokens_used, cost,
                request_type, success, error_message, user_input, response_length
            ) VALUES (?1, ?2, ?3, ?4, ?5, 'completion', ?6, ?7, ?8, ?9)
            "#,
        )
        .bind(provider)
        .bind(model)
        .bind(timestamp)
        .bind(tokens as i64)
        .bind(cost)
        .bind(if success { 1 } else { 0 })
        .bind(error_message)
        .bind(user_input)
        .bind(tokens as i64) // response_length approximation
        .execute(&self.db)
        .await?;

        // Update provider stats
        self.update_provider_stats(provider, tokens, cost, success).await?;

        // Check budget limits
        self.check_budget_limits(provider).await?;

        Ok(())
    }

    /// Update provider statistics
    async fn update_provider_stats(
        &self,
        provider: &str,
        tokens: u32,
        cost: f64,
        success: bool,
    ) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO provider_stats (
                provider_name, total_requests, successful_requests, failed_requests,
                total_tokens, total_cost, last_updated
            ) VALUES (?1, 1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(provider_name) DO UPDATE SET
                total_requests = total_requests + 1,
                successful_requests = successful_requests + excluded.successful_requests,
                failed_requests = failed_requests + excluded.failed_requests,
                total_tokens = total_tokens + excluded.total_tokens,
                total_cost = total_cost + excluded.total_cost,
                last_updated = excluded.last_updated
            "#,
        )
        .bind(provider)
        .bind(if success { 1 } else { 0 })
        .bind(if success { 0 } else { 1 })
        .bind(tokens as i64)
        .bind(cost)
        .bind(timestamp)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Check budget limits and create alerts if needed
    async fn check_budget_limits(&self, provider: &str) -> Result<()> {
        // Get monthly budget limit
        let budget: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT limit_amount FROM cost_budgets
            WHERE budget_type = 'monthly'
            AND (provider_name IS NULL OR provider_name = ?1)
            ORDER BY provider_name DESC NULLS LAST
            LIMIT 1
            "#,
        )
        .bind(provider)
        .fetch_optional(&self.db)
        .await?;

        if let Some(limit) = budget {
            // Get current month's spending
            let current_spending = self.get_monthly_cost(provider).await?;

            let percent = (current_spending / limit) * 100.0;

            // Create alerts at 80% and 100%
            if percent >= 100.0 {
                self.create_alert(
                    "budget_exceeded",
                    Some(provider),
                    &format!(
                        "Monthly budget exceeded: ${:.2} / ${:.2}",
                        current_spending, limit
                    ),
                    Some(percent),
                )
                .await?;
            } else if percent >= 80.0 {
                self.create_alert(
                    "budget_warning",
                    Some(provider),
                    &format!(
                        "Approaching monthly budget: ${:.2} / ${:.2} ({:.0}%)",
                        current_spending, limit, percent
                    ),
                    Some(percent),
                )
                .await?;
            }
        }

        Ok(())
    }

    /// Create cost alert
    async fn create_alert(
        &self,
        alert_type: &str,
        provider: Option<&str>,
        message: &str,
        threshold: Option<f64>,
    ) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        // Check if similar unacknowledged alert exists (avoid spam)
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM cost_alerts
                WHERE alert_type = ?1
                AND COALESCE(provider_name, '') = COALESCE(?2, '')
                AND acknowledged = 0
                AND triggered_at > ?3
            )
            "#,
        )
        .bind(alert_type)
        .bind(provider)
        .bind(timestamp - 3600) // Don't create duplicate within 1 hour
        .fetch_one(&self.db)
        .await?;

        if !exists {
            sqlx::query(
                r#"
                INSERT INTO cost_alerts (alert_type, provider_name, message, threshold_percent)
                VALUES (?1, ?2, ?3, ?4)
                "#,
            )
            .bind(alert_type)
            .bind(provider)
            .bind(message)
            .bind(threshold)
            .execute(&self.db)
            .await?;
        }

        Ok(())
    }

    /// Get current month's cost for a provider
    async fn get_monthly_cost(&self, provider: &str) -> Result<f64> {
        let cost: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(cost), 0.0) FROM provider_usage
            WHERE provider_name = ?1
            AND strftime('%Y-%m', timestamp, 'unixepoch') = strftime('%Y-%m', 'now')
            "#,
        )
        .bind(provider)
        .fetch_one(&self.db)
        .await?;

        Ok(cost.unwrap_or(0.0))
    }

    /// Get monthly costs grouped by provider
    pub async fn get_monthly_costs(&self) -> Result<HashMap<String, f64>> {
        let rows: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT provider_name, COALESCE(SUM(cost), 0.0) as total_cost
            FROM provider_usage
            WHERE strftime('%Y-%m', timestamp, 'unixepoch') = strftime('%Y-%m', 'now')
            GROUP BY provider_name
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(rows.into_iter().collect())
    }

    /// Get total monthly cost
    pub async fn get_total_monthly_cost(&self) -> Result<f64> {
        let cost: Option<f64> = sqlx::query_scalar(
            r#"
            SELECT COALESCE(SUM(cost), 0.0) FROM provider_usage
            WHERE strftime('%Y-%m', timestamp, 'unixepoch') = strftime('%Y-%m', 'now')
            "#,
        )
        .fetch_one(&self.db)
        .await?;

        Ok(cost.unwrap_or(0.0))
    }

    /// Get monthly budget limit
    pub async fn get_monthly_budget(&self) -> Result<Option<f64>> {
        let budget: Option<f64> = sqlx::query_scalar(
            "SELECT limit_amount FROM cost_budgets WHERE budget_type = 'monthly' AND provider_name IS NULL"
        )
        .fetch_optional(&self.db)
        .await?;

        Ok(budget)
    }

    /// Set monthly budget limit
    pub async fn set_monthly_budget(&self, limit: f64) -> Result<()> {
        let timestamp = Utc::now().timestamp();

        sqlx::query(
            r#"
            INSERT INTO cost_budgets (budget_type, limit_amount, updated_at)
            VALUES ('monthly', ?1, ?2)
            ON CONFLICT(budget_type, COALESCE(provider_name, '')) DO UPDATE SET
                limit_amount = excluded.limit_amount,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(limit)
        .bind(timestamp)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get unacknowledged alerts
    pub async fn get_alerts(&self) -> Result<Vec<CostAlert>> {
        let alerts = sqlx::query_as::<_, CostAlert>(
            r#"
            SELECT id, alert_type, provider_name, message, threshold_percent, triggered_at
            FROM cost_alerts
            WHERE acknowledged = 0
            ORDER BY triggered_at DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(alerts)
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: i64) -> Result<()> {
        sqlx::query("UPDATE cost_alerts SET acknowledged = 1 WHERE id = ?1")
            .bind(alert_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Get provider statistics
    pub async fn get_provider_stats(&self) -> Result<Vec<ProviderStats>> {
        let stats = sqlx::query_as::<_, ProviderStats>(
            "SELECT * FROM provider_stats ORDER BY total_cost DESC"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(stats)
    }

    /// Get detailed cost report
    pub async fn get_cost_report(&self, days: u32) -> Result<CostReport> {
        let cutoff = Utc::now().timestamp() - (days as i64 * 86400);

        // Total costs by provider
        let by_provider: Vec<(String, f64)> = sqlx::query_as(
            r#"
            SELECT provider_name, COALESCE(SUM(cost), 0.0) as total_cost
            FROM provider_usage
            WHERE timestamp >= ?1
            GROUP BY provider_name
            ORDER BY total_cost DESC
            "#,
        )
        .bind(cutoff)
        .fetch_all(&self.db)
        .await?;

        // Total requests
        let total_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM provider_usage WHERE timestamp >= ?1"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // Successful requests
        let successful_requests: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM provider_usage WHERE timestamp >= ?1 AND success = 1"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // Total tokens
        let total_tokens: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(tokens_used), 0) FROM provider_usage WHERE timestamp >= ?1"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        // Total cost
        let total_cost: f64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(cost), 0.0) FROM provider_usage WHERE timestamp >= ?1"
        )
        .bind(cutoff)
        .fetch_one(&self.db)
        .await?;

        Ok(CostReport {
            days,
            total_requests: total_requests as u64,
            successful_requests: successful_requests as u64,
            total_tokens: total_tokens as u64,
            total_cost,
            by_provider: by_provider.into_iter().collect(),
        })
    }
}

/// Cost alert
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CostAlert {
    pub id: i64,
    pub alert_type: String,
    pub provider_name: Option<String>,
    pub message: String,
    pub threshold_percent: Option<f64>,
    pub triggered_at: i64,
}

/// Provider statistics
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProviderStats {
    pub provider_name: String,
    pub total_requests: i64,
    pub successful_requests: i64,
    pub failed_requests: i64,
    pub total_tokens: i64,
    pub total_cost: f64,
    pub last_updated: i64,
}

/// Cost report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostReport {
    pub days: u32,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub by_provider: HashMap<String, f64>,
}
