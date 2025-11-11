use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub license: LicenseConfig,
    pub daemon: DaemonConfig,
    pub provider_mode: ProviderMode,
    pub default_provider: String,
    pub providers: HashMap<String, ProviderConfig>,
    pub auto_routing: Option<AutoRoutingConfig>,
    pub learning: LearningConfig,
    pub monitoring: MonitoringConfig,
    pub classification: ClassificationConfig,
    pub execution: ExecutionConfig,
    pub context: ContextConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub key: Option<String>,
    pub company: Option<String>,
    pub user: Option<String>,
    #[serde(default = "default_validation_interval")]
    pub validation_interval_hours: u64,
}

fn default_validation_interval() -> u64 {
    48
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_true")]
    pub auto_restart: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProviderMode {
    Manual,
    Auto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_key: Option<String>,
    pub base_url: Option<String>,
    pub model: Option<String>,
    pub models: Option<Vec<ModelConfig>>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    pub cost: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub capabilities: Vec<String>,
    pub cost: String,
    pub context_window: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRoutingConfig {
    pub enabled: bool,
    pub prefer_cost: String,
    pub prefer_speed: Option<String>,
    pub fallback_chain: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f32,
    #[serde(default = "default_max_patterns")]
    pub max_patterns: usize,
    #[serde(default = "default_embedding_model")]
    pub embedding_model: String,
}

fn default_confidence_threshold() -> f32 {
    0.7
}

fn default_max_patterns() -> usize {
    10000
}

fn default_embedding_model() -> String {
    "minilm-l6-v2".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_interval")]
    pub interval_seconds: u64,
    #[serde(default = "default_true")]
    pub watch_git_repos: bool,
    #[serde(default = "default_true")]
    pub watch_system: bool,
    #[serde(default = "default_true")]
    pub desktop_notifications: bool,
}

fn default_interval() -> u64 {
    60
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationConfig {
    #[serde(default = "default_nl_threshold")]
    pub natural_language_threshold: f32,
    #[serde(default = "default_true")]
    pub check_path_binaries: bool,
    #[serde(default = "default_true")]
    pub cache_known_commands: bool,
}

fn default_nl_threshold() -> f32 {
    0.8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub auto_approve: bool,
    #[serde(default = "default_true")]
    pub confirm_destructive: bool,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
}

fn default_timeout() -> u64 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(default = "default_true")]
    pub track_directory_patterns: bool,
    #[serde(default = "default_true")]
    pub detect_languages: bool,
    #[serde(default = "default_true")]
    pub detect_frameworks: bool,
    #[serde(default = "default_true")]
    pub include_git_context: bool,
    #[serde(default = "default_recent_commands")]
    pub max_recent_commands: usize,
}

fn default_recent_commands() -> usize {
    20
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    #[serde(default = "default_true")]
    pub emoji: bool,
    #[serde(default = "default_true")]
    pub colors: bool,
    #[serde(default)]
    pub show_provider: bool,
    #[serde(default = "default_true")]
    pub show_learning_stats: bool,
}

impl Config {
    pub async fn load() -> Result<Self> {
        // Allow override for testing
        let config_path = if let Ok(override_path) = std::env::var("ORBIT_CONFIG") {
            PathBuf::from(override_path)
        } else {
            Self::config_path()?
        };

        if !config_path.exists() {
            return Self::default_config();
        }

        let content = tokio::fs::read_to_string(&config_path)
            .await
            .context("Failed to read config file")?;

        let config: Config =
            serde_yaml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to find config directory")?
            .join("orbit");

        std::fs::create_dir_all(&config_dir)?;

        Ok(config_dir.join("config.yaml"))
    }

    pub fn data_dir() -> Result<PathBuf> {
        // Allow override for testing
        if let Ok(override_dir) = std::env::var("ORBIT_DATA_DIR") {
            let data_dir = PathBuf::from(override_dir);
            std::fs::create_dir_all(&data_dir)?;
            return Ok(data_dir);
        }

        let data_dir = dirs::data_dir()
            .context("Failed to find data directory")?
            .join("orbit");

        std::fs::create_dir_all(&data_dir)?;

        Ok(data_dir)
    }

    pub fn is_development_mode(&self) -> bool {
        std::env::var("ORBIT_DEV_MODE").is_ok() || cfg!(debug_assertions)
    }

    fn default_config() -> Result<Self> {
        let home = dirs::home_dir().context("Failed to find home directory")?;
        let socket_path = home.join(".orbit").join("daemon.sock");

        Ok(Config {
            license: LicenseConfig {
                key: None,
                company: None,
                user: None,
                validation_interval_hours: 48,
            },
            daemon: DaemonConfig {
                socket_path,
                log_level: "info".to_string(),
                auto_restart: true,
            },
            provider_mode: ProviderMode::Auto,
            default_provider: "claude".to_string(),
            providers: HashMap::new(),
            auto_routing: Some(AutoRoutingConfig {
                enabled: true,
                prefer_cost: "medium".to_string(),
                prefer_speed: Some("fast".to_string()),
                fallback_chain: vec![
                    "claude".to_string(),
                    "gpt-4".to_string(),
                    "gemini".to_string(),
                ],
            }),
            learning: LearningConfig {
                enabled: true,
                confidence_threshold: 0.7,
                max_patterns: 10000,
                embedding_model: "minilm-l6-v2".to_string(),
            },
            monitoring: MonitoringConfig {
                enabled: true,
                interval_seconds: 60,
                watch_git_repos: true,
                watch_system: true,
                desktop_notifications: true,
            },
            classification: ClassificationConfig {
                natural_language_threshold: 0.8,
                check_path_binaries: true,
                cache_known_commands: true,
            },
            execution: ExecutionConfig {
                auto_approve: false,
                confirm_destructive: true,
                timeout_seconds: 300,
            },
            context: ContextConfig {
                track_directory_patterns: true,
                detect_languages: true,
                detect_frameworks: true,
                include_git_context: true,
                max_recent_commands: 20,
            },
            ui: UiConfig {
                emoji: true,
                colors: true,
                show_provider: false,
                show_learning_stats: true,
            },
        })
    }
}
