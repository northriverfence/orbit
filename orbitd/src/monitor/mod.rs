pub mod git;

use anyhow::Result;
use chrono::{Datelike, Timelike, Utc};
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tracing::{debug, info};

use crate::config::Config;
use crate::learning::LearningEngine;
use git::{analyze_repo, find_git_repos, GitSuggestion};

#[derive(Clone)]
pub struct ProactiveMonitor {
    config: Arc<Config>,
    learning_engine: Arc<LearningEngine>,
}

impl ProactiveMonitor {
    pub async fn new(config: Arc<Config>, learning_engine: Arc<LearningEngine>) -> Result<Self> {
        Ok(Self {
            config,
            learning_engine,
        })
    }

    pub async fn run(&self) -> Result<()> {
        let mut interval_timer =
            interval(Duration::from_secs(self.config.monitoring.interval_seconds));

        info!("Proactive monitor started");

        loop {
            interval_timer.tick().await;

            // Monitor tasks
            if self.config.monitoring.watch_git_repos {
                self.check_git_status().await;
            }

            if self.config.monitoring.watch_system {
                self.check_system_conditions().await;
            }

            // Check temporal patterns
            self.check_temporal_patterns().await;
        }
    }

    async fn check_git_status(&self) {
        debug!("Checking git status");

        // Find git repositories in common locations
        let search_paths = vec![
            dirs::home_dir().unwrap_or_default().join("projects"),
            dirs::home_dir().unwrap_or_default().join("dev"),
            dirs::home_dir().unwrap_or_default().join("workspace"),
            std::env::current_dir().unwrap_or_default(),
        ];

        let repos = find_git_repos(search_paths).await;

        debug!("Found {} git repositories", repos.len());

        for repo in repos {
            let suggestions = analyze_repo(&repo);

            for suggestion in suggestions {
                // Show suggestion to user (via desktop notification or inline)
                self.show_suggestion(&suggestion).await;
            }
        }
    }

    async fn check_system_conditions(&self) {
        debug!("Checking system conditions");

        // Check disk space
        if let Ok(disk_usage) = Self::get_disk_usage() {
            if disk_usage > 90.0 {
                self.show_notification(
                    "⚠️  Disk space warning",
                    &format!("Disk usage is at {:.1}%", disk_usage),
                    Some("df -h".to_string()),
                )
                .await;
            }
        }

        // TODO: Check long-running processes
        // TODO: Check system load
    }

    async fn check_temporal_patterns(&self) {
        let now = Utc::now();
        let hour = now.hour() as i32;
        let day = now.weekday().num_days_from_monday() as i32;

        // Get patterns for this time
        if let Ok(patterns) = self.learning_engine.get_temporal_patterns(hour, day).await {
            for pattern in patterns {
                if pattern.frequency >= 3 && pattern.should_suggest() {
                    self.show_notification(
                        "💡 Routine suggestion",
                        &format!("You usually run '{}' around this time", pattern.command),
                        Some(pattern.command.clone()),
                    )
                    .await;
                }
            }
        }
    }

    async fn show_suggestion(&self, suggestion: &GitSuggestion) {
        let message = suggestion.message();
        let command = suggestion.command();

        debug!("Git suggestion: {}", message);

        if self.config.monitoring.desktop_notifications {
            self.show_notification("Orbit - Git Status", &message, command)
                .await;
        }

        // TODO: Also show inline in terminal if active session
    }

    async fn show_notification(&self, title: &str, message: &str, command: Option<String>) {
        if !self.config.monitoring.desktop_notifications {
            return;
        }

        #[cfg(not(target_os = "windows"))]
        {
            let mut notification = notify_rust::Notification::new();
            notification.summary(title).body(message);

            if let Some(cmd) = command {
                notification.action("execute", &format!("Run: {}", cmd));
            }

            if let Err(e) = notification.show() {
                debug!("Failed to show notification: {}", e);
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows notifications would go here
            debug!("Notification: {} - {}", title, message);
        }
    }

    fn get_disk_usage() -> Result<f32> {
        // Use sysinfo to get disk usage
        use sysinfo::Disks;

        let disks = Disks::new_with_refreshed_list();
        if let Some(disk) = disks.first() {
            let total = disk.total_space() as f64;
            let available = disk.available_space() as f64;
            let used = total - available;
            let usage_percent = (used / total * 100.0) as f32;

            Ok(usage_percent)
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_monitor() -> ProactiveMonitor {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path().to_path_buf();
        let config_path = temp_path.join("config.yaml");
        let data_dir = temp_path.join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        std::fs::write(
            &config_path,
            format!(
                r#"
license:
  key: "test-key"
daemon:
  socket_path: "{}/orbit.sock"
provider_mode: manual
default_provider: "test"
providers:
  test:
    api_key: "test"
learning:
  enabled: true
  confidence_threshold: 0.7
monitoring:
  interval_seconds: 300
  watch_git_repos: true
  watch_system: true
  desktop_notifications: false
classification:
  confidence_threshold: 0.7
execution:
  confirm_destructive: true
context:
  working_dir_history_size: 50
  recent_commands_count: 20
ui:
  color: true
"#,
                data_dir.display()
            ),
        )
        .unwrap();

        std::env::set_var("ORBIT_CONFIG", config_path.to_str().unwrap());
        std::env::set_var("ORBIT_DATA_DIR", data_dir.to_str().unwrap());
        std::env::set_var("ORBIT_DEV_MODE", "1");
        std::mem::forget(temp_dir);

        let config = Arc::new(crate::config::Config::load().await.unwrap());

        // Pre-create database file for learning engine
        let db_path = data_dir.join("learning.db");
        std::fs::create_dir_all(db_path.parent().unwrap()).unwrap();
        std::fs::File::create(&db_path).unwrap();

        let learning_engine = Arc::new(
            crate::learning::LearningEngine::new(config.clone())
                .await
                .unwrap(),
        );

        ProactiveMonitor::new(config, learning_engine)
            .await
            .unwrap()
    }

    // ========== Initialization Tests ==========

    #[tokio::test]
    async fn test_monitor_initialization() {
        let monitor = create_test_monitor().await;

        assert_eq!(monitor.config.monitoring.interval_seconds, 300);
        assert!(monitor.config.monitoring.watch_git_repos);
        assert!(monitor.config.monitoring.watch_system);
    }

    // ========== Disk Usage Tests ==========

    #[test]
    fn test_get_disk_usage() {
        let usage = ProactiveMonitor::get_disk_usage().unwrap();

        // Should return a valid percentage
        assert!(
            usage >= 0.0,
            "Disk usage should be non-negative, got {}",
            usage
        );
        assert!(
            usage <= 100.0,
            "Disk usage should be <= 100%, got {}",
            usage
        );
    }

    #[test]
    fn test_get_disk_usage_returns_reasonable_value() {
        let usage = ProactiveMonitor::get_disk_usage().unwrap();

        // Most systems will have some disk usage
        // but this is a sanity check
        assert!(
            usage >= 0.0 && usage <= 100.0,
            "Disk usage {} should be in valid range",
            usage
        );
    }
}
