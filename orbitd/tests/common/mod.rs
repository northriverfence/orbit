use std::sync::Arc;
use tempfile::TempDir;

/// Setup test environment with proper configuration
pub fn setup_test_env() {
    // Set development mode
    std::env::set_var("ORBIT_DEV_MODE", "1");

    // Initialize test logging (only once)
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_max_level(tracing::Level::DEBUG)
        .try_init();
}

/// Create a test configuration
pub async fn create_test_config() -> Arc<orbitd::config::Config> {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Write test config
    std::fs::write(
        &config_path,
        format!(
            r#"
license:
  key: "test-key-12345"
daemon:
  socket_path: "{}/orbit.sock"
  log_level: "debug"
provider_mode: manual
default_provider: "anthropic"
providers:
  anthropic:
    api_key: "test-anthropic-key"
  openai:
    api_key: "test-openai-key"
  gemini:
    api_key: "test-gemini-key"
learning:
  enabled: true
  similarity_threshold: 0.8
monitoring:
  interval_seconds: 300
  watch_git_repos: false
  watch_system: false
  desktop_notifications: false
classification:
  confidence_threshold: 0.7
execution:
  dry_run: false
  confirm_destructive: true
context:
  working_dir_history_size: 50
  recent_commands_count: 20
ui:
  color: true
  emoji: true
"#,
            data_dir.display()
        ),
    )
    .unwrap();

    std::env::set_var("ORBIT_CONFIG", config_path.to_str().unwrap());
    std::env::set_var("ORBIT_DEV_MODE", "1");
    std::env::set_var("ORBIT_DATA_DIR", data_dir.to_str().unwrap());

    let config = orbitd::config::Config::load()
        .await
        .expect("Failed to load test config");

    // Leak temp_dir to keep it alive for duration of test
    std::mem::forget(temp_dir);

    Arc::new(config)
}

/// Create a temporary database for testing
pub fn create_test_db() -> TempDir {
    TempDir::new().expect("Failed to create temp dir")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup() {
        setup_test_env();
        assert_eq!(std::env::var("ORBIT_DEV_MODE").unwrap(), "1");
    }
}
