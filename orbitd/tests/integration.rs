mod common;

use common::*;
use orbitd::config::Config;
use serial_test::serial;
use tempfile::TempDir;

#[tokio::test]
async fn test_config_loading() {
    setup_test_env();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.yaml");
    let data_dir = temp_dir.path().join("data");
    std::fs::create_dir_all(&data_dir).unwrap();

    // Write minimal config
    std::fs::write(
        &config_path,
        format!(
            r#"
license:
  key: "test-key"
daemon:
  socket_path: "{}/orbit.sock"
provider_mode: manual
default_provider: "anthropic"
providers:
  anthropic:
    api_key: "test-anthropic"
  openai:
    api_key: "test-openai"
learning:
  enabled: true
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
  emoji: true
"#,
            data_dir.display()
        ),
    )
    .unwrap();

    std::env::set_var("ORBIT_CONFIG", config_path.to_str().unwrap());
    std::env::set_var("ORBIT_DEV_MODE", "1");
    std::env::set_var("ORBIT_DATA_DIR", data_dir.to_str().unwrap());

    let config = Config::load().await;
    assert!(config.is_ok(), "Failed to load config: {:?}", config.err());

    let config = config.unwrap();
    assert!(
        config.providers.contains_key("anthropic"),
        "Config missing anthropic provider"
    );
    assert!(
        config.providers.contains_key("openai"),
        "Config missing openai provider"
    );
    assert!(config.is_development_mode());
}

#[tokio::test]
#[serial_test::serial]
async fn test_learning_engine_initialization() {
    setup_test_env();

    let config = create_test_config().await;

    // Pre-create the database file to help SQLite in test environment
    let data_dir = orbitd::config::Config::data_dir().unwrap();
    let db_path = data_dir.join("learning.db");
    std::fs::File::create(&db_path).expect("Failed to create test db file");

    let learning_engine = orbitd::learning::LearningEngine::new(config).await;

    assert!(
        learning_engine.is_ok(),
        "Failed to initialize learning engine: {:?}",
        learning_engine.err()
    );
}

#[tokio::test]
#[serial_test::serial]
async fn test_command_classification() {
    setup_test_env();

    let config = create_test_config().await;

    // Pre-create the database file for SQLite in test environment
    let data_dir = orbitd::config::Config::data_dir().unwrap();
    let db_path = data_dir.join("learning.db");
    std::fs::File::create(&db_path).expect("Failed to create test db file");

    let learning_engine = orbitd::learning::LearningEngine::new(config.clone())
        .await
        .expect("Failed to initialize learning engine for classification test");
    let classifier = orbitd::classifier::CommandClassifier::new(
        config.clone(),
        std::sync::Arc::new(learning_engine),
    )
    .await;

    assert!(
        classifier.is_ok(),
        "Failed to initialize classifier: {:?}",
        classifier.err()
    );
}

#[tokio::test]
async fn test_context_engine() {
    setup_test_env();

    let config = create_test_config().await;
    let context_engine = orbitd::context::ContextEngine::new(config).await;

    assert!(
        context_engine.is_ok(),
        "Failed to initialize context engine: {:?}",
        context_engine.err()
    );

    let context_engine = context_engine.unwrap();
    let context = context_engine.get_context().await;

    assert!(
        context.is_ok(),
        "Failed to get context: {:?}",
        context.err()
    );
}

#[tokio::test]
async fn test_executor_initialization() {
    setup_test_env();

    let config = create_test_config().await;
    let executor = orbitd::executor::Executor::new(config).await;

    assert!(
        executor.is_ok(),
        "Failed to initialize executor: {:?}",
        executor.err()
    );
}
