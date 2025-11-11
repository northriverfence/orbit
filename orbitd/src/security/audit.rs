// Audit logging system for Orbit AI Terminal
//
// This module provides comprehensive audit logging for:
// - Command executions and results
// - AI provider queries and responses
// - User actions (accept/reject/modify)
// - Configuration changes
// - Security events and violations

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, Row};
use std::path::Path;
use std::sync::OnceLock;
use chrono::{DateTime, Utc};

/// Audit logger for security and compliance
pub struct AuditLogger {
    pool: Pool<Sqlite>,
}

impl AuditLogger {
    /// Create a new audit logger
    ///
    /// # Arguments
    /// * `db_path` - Path to audit log database
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_url = format!("sqlite:{}", db_path.as_ref().display());

        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let logger = Self { pool };

        // Initialize database schema
        logger.init_schema().await?;

        Ok(logger)
    }

    /// Initialize the audit log database schema
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS audit_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_type TEXT NOT NULL,
                user TEXT NOT NULL,
                command TEXT,
                result TEXT,
                details TEXT,
                severity TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                session_id TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for efficient querying
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_log(timestamp DESC)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_log(event_type)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_user ON audit_log(user)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_log(severity)",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Log an audit event
    ///
    /// # Arguments
    /// * `event` - The audit event to log
    pub async fn log_event(&self, event: AuditEvent) -> Result<i64> {
        let details_json = serde_json::to_string(&event.details)?;

        let result = sqlx::query(
            r#"
            INSERT INTO audit_log (event_type, user, command, result, details, severity, timestamp, session_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(event.event_type.to_string())
        .bind(&event.user)
        .bind(&event.command)
        .bind(&event.result)
        .bind(&details_json)
        .bind(event.severity.to_string())
        .bind(event.timestamp.timestamp())
        .bind(&event.session_id)
        .execute(&self.pool)
        .await?;

        Ok(result.last_insert_rowid())
    }

    /// Query audit logs with filters
    ///
    /// # Arguments
    /// * `filter` - Filter criteria for the query
    pub async fn query_logs(&self, filter: LogFilter) -> Result<Vec<AuditEvent>> {
        let mut query = String::from("SELECT * FROM audit_log WHERE 1=1");

        if let Some(start) = filter.start_time {
            query.push_str(&format!(" AND timestamp >= {}", start.timestamp()));
        }

        if let Some(end) = filter.end_time {
            query.push_str(&format!(" AND timestamp <= {}", end.timestamp()));
        }

        if let Some(ref event_type) = filter.event_type {
            query.push_str(&format!(" AND event_type = '{}'", event_type));
        }

        if let Some(ref user) = filter.user {
            query.push_str(&format!(" AND user = '{}'", user));
        }

        if let Some(ref severity) = filter.severity {
            query.push_str(&format!(" AND severity = '{}'", severity));
        }

        query.push_str(&format!(" ORDER BY timestamp DESC LIMIT {}", filter.limit));

        let rows = sqlx::query(&query).fetch_all(&self.pool).await?;

        let events: Vec<AuditEvent> = rows
            .into_iter()
            .filter_map(|row| {
                Some(AuditEvent {
                    event_type: row.get::<String, _>("event_type").parse().ok()?,
                    user: row.get("user"),
                    command: row.get("command"),
                    result: row.get("result"),
                    details: serde_json::from_str(row.get::<String, _>("details")).ok()?,
                    severity: row.get::<String, _>("severity").parse().ok()?,
                    timestamp: DateTime::from_timestamp(row.get::<i64, _>("timestamp"), 0)?,
                    session_id: row.get("session_id"),
                })
            })
            .collect();

        Ok(events)
    }

    /// Get audit statistics
    pub async fn stats(&self) -> Result<AuditStats> {
        let total_events: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&self.pool)
            .await?;

        let security_events: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE event_type = 'SecurityEvent'",
        )
        .fetch_one(&self.pool)
        .await?;

        let commands_executed: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE event_type = 'CommandExecuted'",
        )
        .fetch_one(&self.pool)
        .await?;

        let commands_rejected: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE event_type = 'CommandRejected'",
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(AuditStats {
            total_events: total_events as usize,
            security_events: security_events as usize,
            commands_executed: commands_executed as usize,
            commands_rejected: commands_rejected as usize,
            rejection_rate: if commands_executed + commands_rejected > 0 {
                (commands_rejected as f64 / (commands_executed + commands_rejected) as f64) * 100.0
            } else {
                0.0
            },
        })
    }

    /// Delete old audit logs (for retention policy)
    pub async fn cleanup_old_logs(&self, days: u64) -> Result<u64> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);

        let result = sqlx::query("DELETE FROM audit_log WHERE timestamp < ?")
            .bind(cutoff.timestamp())
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected())
    }

    /// Export audit logs to JSON
    pub async fn export_logs(&self, filter: LogFilter) -> Result<String> {
        let events = self.query_logs(filter).await?;
        Ok(serde_json::to_string_pretty(&events)?)
    }
}

/// Audit event to be logged
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: EventType,
    pub user: String,
    pub command: Option<String>,
    pub result: Option<String>,
    pub details: serde_json::Value,
    pub severity: AuditSeverity,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<String>,
}

impl AuditEvent {
    /// Create a command execution event
    pub fn command_executed(
        user: String,
        command: String,
        result: String,
        duration_ms: u64,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_type: EventType::CommandExecuted,
            user,
            command: Some(command),
            result: Some(result),
            details: serde_json::json!({
                "duration_ms": duration_ms,
            }),
            severity: AuditSeverity::Info,
            timestamp: Utc::now(),
            session_id,
        }
    }

    /// Create a command rejection event
    pub fn command_rejected(
        user: String,
        command: String,
        reason: String,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_type: EventType::CommandRejected,
            user,
            command: Some(command),
            result: None,
            details: serde_json::json!({
                "reason": reason,
            }),
            severity: AuditSeverity::Warning,
            timestamp: Utc::now(),
            session_id,
        }
    }

    /// Create an AI query event
    pub fn ai_query(
        user: String,
        provider: String,
        prompt: String,
        response_length: usize,
        duration_ms: u64,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_type: EventType::AIQuery,
            user,
            command: None,
            result: None,
            details: serde_json::json!({
                "provider": provider,
                "prompt_length": prompt.len(),
                "response_length": response_length,
                "duration_ms": duration_ms,
            }),
            severity: AuditSeverity::Info,
            timestamp: Utc::now(),
            session_id,
        }
    }

    /// Create a security event
    pub fn security_event(
        user: String,
        event_description: String,
        severity: AuditSeverity,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_type: EventType::SecurityEvent,
            user,
            command: None,
            result: None,
            details: serde_json::json!({
                "description": event_description,
            }),
            severity,
            timestamp: Utc::now(),
            session_id,
        }
    }

    /// Create a configuration change event
    pub fn config_change(
        user: String,
        setting: String,
        old_value: String,
        new_value: String,
        session_id: Option<String>,
    ) -> Self {
        Self {
            event_type: EventType::ConfigChange,
            user,
            command: None,
            result: None,
            details: serde_json::json!({
                "setting": setting,
                "old_value": old_value,
                "new_value": new_value,
            }),
            severity: AuditSeverity::Info,
            timestamp: Utc::now(),
            session_id,
        }
    }
}

/// Type of audit event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    CommandExecuted,
    CommandRejected,
    AIQuery,
    SecurityEvent,
    ConfigChange,
    UserAction,
}

impl std::fmt::Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::CommandExecuted => write!(f, "CommandExecuted"),
            EventType::CommandRejected => write!(f, "CommandRejected"),
            EventType::AIQuery => write!(f, "AIQuery"),
            EventType::SecurityEvent => write!(f, "SecurityEvent"),
            EventType::ConfigChange => write!(f, "ConfigChange"),
            EventType::UserAction => write!(f, "UserAction"),
        }
    }
}

impl std::str::FromStr for EventType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CommandExecuted" => Ok(EventType::CommandExecuted),
            "CommandRejected" => Ok(EventType::CommandRejected),
            "AIQuery" => Ok(EventType::AIQuery),
            "SecurityEvent" => Ok(EventType::SecurityEvent),
            "ConfigChange" => Ok(EventType::ConfigChange),
            "UserAction" => Ok(EventType::UserAction),
            _ => Err(format!("Unknown event type: {}", s)),
        }
    }
}

/// Severity level for audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl std::fmt::Display for AuditSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditSeverity::Info => write!(f, "Info"),
            AuditSeverity::Warning => write!(f, "Warning"),
            AuditSeverity::Error => write!(f, "Error"),
            AuditSeverity::Critical => write!(f, "Critical"),
        }
    }
}

impl std::str::FromStr for AuditSeverity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Info" => Ok(AuditSeverity::Info),
            "Warning" => Ok(AuditSeverity::Warning),
            "Error" => Ok(AuditSeverity::Error),
            "Critical" => Ok(AuditSeverity::Critical),
            _ => Err(format!("Unknown severity: {}", s)),
        }
    }
}

/// Filter for querying audit logs
#[derive(Debug, Clone, Default)]
pub struct LogFilter {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub event_type: Option<String>,
    pub user: Option<String>,
    pub severity: Option<String>,
    pub limit: usize,
}

impl LogFilter {
    /// Create a new log filter with default limit
    pub fn new() -> Self {
        Self {
            limit: 100,
            ..Default::default()
        }
    }

    /// Set the time range for the filter
    pub fn time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_time = Some(start);
        self.end_time = Some(end);
        self
    }

    /// Filter by event type
    pub fn event_type(mut self, event_type: EventType) -> Self {
        self.event_type = Some(event_type.to_string());
        self
    }

    /// Filter by user
    pub fn user(mut self, user: String) -> Self {
        self.user = Some(user);
        self
    }

    /// Filter by severity
    pub fn severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = Some(severity.to_string());
        self
    }

    /// Set the result limit
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize)]
pub struct AuditStats {
    pub total_events: usize,
    pub security_events: usize,
    pub commands_executed: usize,
    pub commands_rejected: usize,
    pub rejection_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_audit_logger_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("audit.db");

        let logger = AuditLogger::new(&db_path).await;
        assert!(logger.is_ok());
    }

    #[tokio::test]
    async fn test_log_command_execution() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("audit.db");

        let logger = AuditLogger::new(&db_path).await.unwrap();

        let event = AuditEvent::command_executed(
            "testuser".to_string(),
            "ls -la".to_string(),
            "Success".to_string(),
            100,
            None,
        );

        let result = logger.log_event(event).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_query_logs() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("audit.db");

        let logger = AuditLogger::new(&db_path).await.unwrap();

        // Log some events
        for i in 0..5 {
            let event = AuditEvent::command_executed(
                "testuser".to_string(),
                format!("command {}", i),
                "Success".to_string(),
                100,
                None,
            );
            logger.log_event(event).await.unwrap();
        }

        // Query logs
        let filter = LogFilter::new().limit(10);
        let logs = logger.query_logs(filter).await.unwrap();

        assert_eq!(logs.len(), 5);
    }

    #[tokio::test]
    async fn test_audit_stats() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("audit.db");

        let logger = AuditLogger::new(&db_path).await.unwrap();

        // Log some events
        logger
            .log_event(AuditEvent::command_executed(
                "user".to_string(),
                "ls".to_string(),
                "Success".to_string(),
                100,
                None,
            ))
            .await
            .unwrap();

        logger
            .log_event(AuditEvent::command_rejected(
                "user".to_string(),
                "rm -rf /".to_string(),
                "Dangerous".to_string(),
                None,
            ))
            .await
            .unwrap();

        let stats = logger.stats().await.unwrap();
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.commands_executed, 1);
        assert_eq!(stats.commands_rejected, 1);
        assert_eq!(stats.rejection_rate, 50.0);
    }

    #[tokio::test]
    async fn test_event_type_parsing() {
        assert_eq!(
            "CommandExecuted".parse::<EventType>().unwrap(),
            EventType::CommandExecuted
        );
        assert_eq!(
            "SecurityEvent".parse::<EventType>().unwrap(),
            EventType::SecurityEvent
        );
        assert!("InvalidType".parse::<EventType>().is_err());
    }

    #[tokio::test]
    async fn test_log_filter_builder() {
        let filter = LogFilter::new()
            .event_type(EventType::CommandExecuted)
            .user("testuser".to_string())
            .severity(AuditSeverity::Warning)
            .limit(50);

        assert_eq!(filter.event_type, Some("CommandExecuted".to_string()));
        assert_eq!(filter.user, Some("testuser".to_string()));
        assert_eq!(filter.limit, 50);
    }
}
