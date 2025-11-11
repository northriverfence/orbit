// Provider system for Orbit AI Terminal
pub mod cost_tracker;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::config::Config;
use crate::context::Context;

pub use cost_tracker::CostTracker;

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub api_key: String,
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f32,
    pub priority: u8,
    pub cost_per_1k_tokens: f64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: "claude".to_string(),
            enabled: true,
            api_key: String::new(),
            model: "claude-3-5-sonnet-20241022".to_string(),
            max_tokens: 1000,
            temperature: 0.7,
            priority: 5,
            cost_per_1k_tokens: 0.015,
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_minute: u32,
    pub tokens_per_minute: u32,
}

impl Default for RateLimit {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: 90000,
        }
    }
}

/// Command context for provider requests
#[derive(Debug, Clone, Default)]
pub struct ProviderContext {
    pub user_input: String,
    pub shell: String,
    pub cwd: String,
    pub git_context: Option<String>,
    pub project_type: Option<String>,
    pub recent_commands: Vec<String>,
    pub complexity: CommandComplexity,
}

/// Command complexity estimation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandComplexity {
    Simple,    // Single word commands, common operations
    Medium,    // Multi-word, flags/options
    Complex,   // Pipes, redirects, complex logic
}

impl Default for CommandComplexity {
    fn default() -> Self {
        Self::Medium
    }
}

impl CommandComplexity {
    /// Estimate complexity from user input
    pub fn estimate(input: &str) -> Self {
        let words = input.split_whitespace().count();
        let has_pipe = input.contains('|');
        let has_redirect = input.contains('>') || input.contains('<');
        let has_complex_syntax = input.contains("$(") || input.contains("&&") || input.contains("||");
        let has_flags = input.split_whitespace().any(|w| w.starts_with('-'));

        if has_complex_syntax || (has_pipe && has_redirect) {
            Self::Complex
        } else if words > 2 || has_pipe || has_redirect || has_flags {
            Self::Medium
        } else {
            Self::Simple
        }
    }
}

/// Provider response
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub suggestion: String,
    pub confidence: f32,
    pub tokens_used: u32,
    pub cost: f64,
    pub provider_name: String,
}

/// Provider router - manages AI provider selection and requests
pub struct ProviderRouter {
    config: Arc<Config>,
    cost_tracker: Option<CostTracker>,
}

impl ProviderRouter {
    /// Create new provider router
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self {
            config,
            cost_tracker: None,
        })
    }

    /// Create router with cost tracking
    pub async fn with_cost_tracking(config: Arc<Config>, db: SqlitePool) -> Result<Self> {
        Ok(Self {
            config,
            cost_tracker: Some(CostTracker::new(db)),
        })
    }

    /// Process natural language input and return shell command suggestion
    pub async fn process_natural_language(&self, input: &str, _context: &Context) -> Result<String> {
        // For now, return a placeholder
        // In production, this would call the actual AI provider (OpenAI, Claude, Gemini)
        // and use the context to provide intelligent suggestions

        // Simple pattern matching for demonstration
        let suggestion = if input.contains("list files") || input.contains("show files") {
            "ls -la".to_string()
        } else if input.contains("current directory") || input.contains("where am i") {
            "pwd".to_string()
        } else if input.contains("disk space") || input.contains("storage") {
            "df -h".to_string()
        } else if input.contains("processes") || input.contains("running") {
            "ps aux | head -20".to_string()
        } else {
            format!("echo \"AI provider ({}) not yet fully implemented. Input: {}\"", self.config.default_provider, input)
        };

        Ok(suggestion)
    }

    /// Get AI suggestion for user input (legacy method)
    pub async fn get_suggestion(&self, input: &str, _context: &ProviderContext) -> Result<String> {
        // For now, return a placeholder
        // In production, this would call the actual AI provider
        Ok(format!("# Command suggestion for: {}\n# Provider: {} not yet implemented\necho \"Provider system in development\"", input, self.config.default_provider))
    }

    /// Record usage for cost tracking
    pub async fn record_usage(
        &self,
        provider: &str,
        model: &str,
        tokens: u32,
        cost: f64,
        success: bool,
    ) -> Result<()> {
        if let Some(tracker) = &self.cost_tracker {
            tracker.record_usage(provider, model, tokens, cost, success, None, None).await?;
        }
        Ok(())
    }

    /// Get cost tracker
    pub fn cost_tracker(&self) -> Option<&CostTracker> {
        self.cost_tracker.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_simple() {
        assert_eq!(CommandComplexity::estimate("ls"), CommandComplexity::Simple);
        assert_eq!(CommandComplexity::estimate("pwd"), CommandComplexity::Simple);
        assert_eq!(CommandComplexity::estimate("cd .."), CommandComplexity::Simple);
    }

    #[test]
    fn test_complexity_medium() {
        assert_eq!(CommandComplexity::estimate("ls -la"), CommandComplexity::Medium);
        assert_eq!(CommandComplexity::estimate("find . -name test"), CommandComplexity::Medium);
        assert_eq!(CommandComplexity::estimate("ls | head"), CommandComplexity::Medium);
    }

    #[test]
    fn test_complexity_complex() {
        assert_eq!(
            CommandComplexity::estimate("find . | grep test > output.txt"),
            CommandComplexity::Complex
        );
        assert_eq!(
            CommandComplexity::estimate("echo $(date) && ls"),
            CommandComplexity::Complex
        );
    }
}
