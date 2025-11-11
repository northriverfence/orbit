use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command execution record for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    pub original_input: String,
    pub suggested_command: Option<String>,
    pub executed_command: String,
    pub result: ExecutionResult,
    pub execution_time_ms: Option<i64>,
    pub exit_code: Option<i32>,
    pub context: CommandContext,
    pub provider: Option<String>,
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum ExecutionResult {
    #[sqlx(rename = "success")]
    Success,
    #[sqlx(rename = "failed")]
    Failed,
    #[sqlx(rename = "rejected")]
    Rejected,
    #[sqlx(rename = "edited")]
    Edited,
}

/// Command context for analysis
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandContext {
    pub cwd: String,
    pub shell: String,
    pub git_repo: Option<String>,
    pub project_type: Option<String>,
}

/// Ranked suggestion with learning-based scoring
#[derive(Debug, Clone)]
pub struct RankedSuggestion {
    pub command: String,
    pub score: f64,
    pub reasons: Vec<String>,
}

/// Detected pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub pattern: String,
    pub frequency: i64,
    pub success_rate: f64,
    pub preferred_translation: Option<String>,
    pub last_used: i64,
}

/// Learning insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Insight {
    pub category: InsightCategory,
    pub insight: String,
    pub confidence: f64,
    pub created_at: i64,
}

/// Insight categories
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
pub enum InsightCategory {
    #[sqlx(rename = "usage")]
    Usage,
    #[sqlx(rename = "preference")]
    Preference,
    #[sqlx(rename = "pattern")]
    Pattern,
    #[sqlx(rename = "error")]
    Error,
    #[sqlx(rename = "optimization")]
    Optimization,
}

/// Analytics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSummary {
    pub total_commands: i64,
    pub ai_suggestions: i64,
    pub accepted: i64,
    pub rejected: i64,
    pub success_rate: f64,
    pub top_patterns: Vec<Pattern>,
    pub insights: Vec<Insight>,
}

/// Learning export format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningExport {
    pub version: String,
    pub exported_at: i64,
    pub preferences: HashMap<String, String>,
    pub patterns: Vec<Pattern>,
    pub analytics_summary: AnalyticsSummary,
}
