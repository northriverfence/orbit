// Unit tests for the Learning System
// Tests pattern detection, suggestion ranking, preferences, and export/import

use orbitd::learning::{LearningSystem, Pattern, ExecutionResult};
use tempfile::tempdir;
use std::path::PathBuf;

#[tokio::test]
async fn test_learning_system_initialization() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let result = LearningSystem::new(&db_path).await;
    assert!(result.is_ok(), "Learning system should initialize successfully");

    let learning = result.unwrap();
    assert!(db_path.exists(), "Database file should be created");
}

#[tokio::test]
async fn test_record_execution_success() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    let result = learning.record_execution(
        "list files",
        Some("ls -la"),
        "ls -la",
        ExecutionResult::Success,
        100, // 100ms execution time
    ).await;

    assert!(result.is_ok(), "Should record execution successfully");
}

#[tokio::test]
async fn test_record_execution_rejected() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    let result = learning.record_execution(
        "remove everything",
        Some("rm -rf /"),
        "",
        ExecutionResult::Rejected,
        0,
    ).await;

    assert!(result.is_ok(), "Should record rejection successfully");
}

#[tokio::test]
async fn test_pattern_detection_single_pattern() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record the same pattern multiple times
    for _ in 0..5 {
        learning.record_execution(
            "list files",
            Some("ls -la"),
            "ls -la",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    let patterns = learning.detect_patterns().await.unwrap();

    // Should detect "list files" -> "ls -la" pattern
    assert!(patterns.len() > 0, "Should detect at least one pattern");

    let list_pattern = patterns.iter()
        .find(|p| p.input.contains("list files"));

    assert!(list_pattern.is_some(), "Should detect 'list files' pattern");

    let pattern = list_pattern.unwrap();
    assert!(pattern.frequency >= 5, "Pattern frequency should be at least 5");
    assert_eq!(pattern.success_rate, 1.0, "Success rate should be 100%");
}

#[tokio::test]
async fn test_pattern_detection_multiple_patterns() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record multiple different patterns
    let patterns_to_record = vec![
        ("list files", "ls -la", 5),
        ("show git status", "git status", 3),
        ("find large files", "du -sh * | sort -rh", 4),
    ];

    for (input, command, count) in patterns_to_record {
        for _ in 0..count {
            learning.record_execution(
                input,
                Some(command),
                command,
                ExecutionResult::Success,
                100,
            ).await.unwrap();
        }
    }

    let detected_patterns = learning.detect_patterns().await.unwrap();

    assert!(detected_patterns.len() >= 3, "Should detect at least 3 patterns");

    // Verify each pattern was detected
    assert!(detected_patterns.iter().any(|p| p.input.contains("list files")));
    assert!(detected_patterns.iter().any(|p| p.input.contains("git status")));
    assert!(detected_patterns.iter().any(|p| p.input.contains("find large")));
}

#[tokio::test]
async fn test_pattern_with_failures() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record pattern with 70% success rate (7 success, 3 failures)
    for _ in 0..7 {
        learning.record_execution(
            "complex operation",
            Some("complicated command"),
            "complicated command",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    for _ in 0..3 {
        learning.record_execution(
            "complex operation",
            Some("complicated command"),
            "complicated command",
            ExecutionResult::Failed,
            100,
        ).await.unwrap();
    }

    let patterns = learning.detect_patterns().await.unwrap();

    let complex_pattern = patterns.iter()
        .find(|p| p.input.contains("complex operation"));

    assert!(complex_pattern.is_some(), "Should detect pattern despite failures");

    let pattern = complex_pattern.unwrap();
    assert!(pattern.success_rate < 1.0 && pattern.success_rate >= 0.7,
            "Success rate should be around 70%");
}

#[tokio::test]
async fn test_suggestion_ranking_by_frequency() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record "ls -la" many times
    for _ in 0..10 {
        learning.record_execution(
            "list files",
            Some("ls -la"),
            "ls -la",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    // Record "ls" only once
    learning.record_execution(
        "list files",
        Some("ls"),
        "ls",
        ExecutionResult::Success,
        100,
    ).await.unwrap();

    let suggestions = vec![
        "ls".to_string(),
        "ls -la".to_string(),
        "find".to_string(),
    ];

    let ranked = learning.rank_suggestions("list files", suggestions).await.unwrap();

    // "ls -la" should be ranked highest due to frequency
    assert_eq!(ranked[0].command, "ls -la", "Most frequent command should rank first");
    assert!(ranked[0].score > 0.8, "Top suggestion should have high score");
    assert!(ranked[0].reasons.iter().any(|r| r.contains("frequently used") || r.contains("High success rate")));
}

#[tokio::test]
async fn test_suggestion_ranking_by_success_rate() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Command A: high frequency but low success rate (5 success, 5 failures)
    for _ in 0..5 {
        learning.record_execution(
            "test command",
            Some("command_a"),
            "command_a",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }
    for _ in 0..5 {
        learning.record_execution(
            "test command",
            Some("command_a"),
            "command_a",
            ExecutionResult::Failed,
            100,
        ).await.unwrap();
    }

    // Command B: lower frequency but perfect success rate (3 success, 0 failures)
    for _ in 0..3 {
        learning.record_execution(
            "test command",
            Some("command_b"),
            "command_b",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    let suggestions = vec![
        "command_a".to_string(),
        "command_b".to_string(),
    ];

    let ranked = learning.rank_suggestions("test command", suggestions).await.unwrap();

    // Command B should rank higher due to better success rate
    assert_eq!(ranked[0].command, "command_b", "Higher success rate should win");
}

#[tokio::test]
async fn test_preference_storage_and_retrieval() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Set multiple preferences
    learning.set_preference("verbosity", "high").await.unwrap();
    learning.set_preference("provider", "claude").await.unwrap();
    learning.set_preference("learning_mode", "adaptive").await.unwrap();

    // Retrieve preferences
    let verbosity = learning.get_preference("verbosity").await.unwrap();
    let provider = learning.get_preference("provider").await.unwrap();
    let learning_mode = learning.get_preference("learning_mode").await.unwrap();

    assert_eq!(verbosity, Some("high".to_string()));
    assert_eq!(provider, Some("claude".to_string()));
    assert_eq!(learning_mode, Some("adaptive".to_string()));
}

#[tokio::test]
async fn test_preference_update() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Set initial preference
    learning.set_preference("theme", "dark").await.unwrap();
    let initial = learning.get_preference("theme").await.unwrap();
    assert_eq!(initial, Some("dark".to_string()));

    // Update preference
    learning.set_preference("theme", "light").await.unwrap();
    let updated = learning.get_preference("theme").await.unwrap();
    assert_eq!(updated, Some("light".to_string()));
}

#[tokio::test]
async fn test_preference_nonexistent() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let learning = LearningSystem::new(&db_path).await.unwrap();

    let result = learning.get_preference("nonexistent").await.unwrap();
    assert_eq!(result, None, "Non-existent preference should return None");
}

#[tokio::test]
async fn test_list_all_preferences() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Set multiple preferences
    learning.set_preference("pref1", "value1").await.unwrap();
    learning.set_preference("pref2", "value2").await.unwrap();
    learning.set_preference("pref3", "value3").await.unwrap();

    let all_prefs = learning.list_preferences().await.unwrap();

    assert_eq!(all_prefs.len(), 3, "Should have 3 preferences");
    assert!(all_prefs.contains_key("pref1"));
    assert!(all_prefs.contains_key("pref2"));
    assert!(all_prefs.contains_key("pref3"));
}

#[tokio::test]
async fn test_export_learning_data() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Add some data
    learning.record_execution(
        "test command",
        Some("echo test"),
        "echo test",
        ExecutionResult::Success,
        50,
    ).await.unwrap();

    learning.set_preference("test_pref", "test_value").await.unwrap();

    // Export data
    let export = learning.export().await.unwrap();

    assert!(export.command_analytics.len() > 0, "Should export command analytics");
    assert!(export.preferences.len() > 0, "Should export preferences");
}

#[tokio::test]
async fn test_import_learning_data() {
    let temp_dir = tempdir().unwrap();
    let db_path1 = temp_dir.path().join("test1.db");
    let db_path2 = temp_dir.path().join("test2.db");

    // Create first learning system and add data
    let mut learning1 = LearningSystem::new(&db_path1).await.unwrap();

    learning1.record_execution(
        "import test",
        Some("echo import"),
        "echo import",
        ExecutionResult::Success,
        75,
    ).await.unwrap();

    learning1.set_preference("import_pref", "import_value").await.unwrap();

    // Export from first system
    let export = learning1.export().await.unwrap();

    // Create second learning system and import
    let mut learning2 = LearningSystem::new(&db_path2).await.unwrap();
    learning2.import(export).await.unwrap();

    // Verify data was imported
    let patterns = learning2.detect_patterns().await.unwrap();
    assert!(patterns.len() > 0, "Should have imported patterns");

    let pref = learning2.get_preference("import_pref").await.unwrap();
    assert_eq!(pref, Some("import_value".to_string()), "Should have imported preferences");
}

#[tokio::test]
async fn test_export_import_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let db_path1 = temp_dir.path().join("test1.db");
    let db_path2 = temp_dir.path().join("test2.db");

    let mut learning1 = LearningSystem::new(&db_path1).await.unwrap();

    // Add diverse data
    let test_data = vec![
        ("list files", "ls -la", ExecutionResult::Success),
        ("show git status", "git status", ExecutionResult::Success),
        ("remove file", "rm dangerous", ExecutionResult::Rejected),
        ("find text", "grep pattern file", ExecutionResult::Success),
        ("count lines", "wc -l file", ExecutionResult::Failed),
    ];

    for (input, command, result) in test_data {
        learning1.record_execution(
            input,
            Some(command),
            command,
            result,
            100,
        ).await.unwrap();
    }

    learning1.set_preference("test1", "value1").await.unwrap();
    learning1.set_preference("test2", "value2").await.unwrap();

    // Export and import
    let export = learning1.export().await.unwrap();
    let mut learning2 = LearningSystem::new(&db_path2).await.unwrap();
    learning2.import(export).await.unwrap();

    // Verify all data survived the roundtrip
    let patterns1 = learning1.detect_patterns().await.unwrap();
    let patterns2 = learning2.detect_patterns().await.unwrap();

    assert_eq!(patterns1.len(), patterns2.len(), "Pattern count should match");

    let prefs1 = learning1.list_preferences().await.unwrap();
    let prefs2 = learning2.list_preferences().await.unwrap();

    assert_eq!(prefs1.len(), prefs2.len(), "Preference count should match");
}

#[tokio::test]
async fn test_get_analytics_summary() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record various executions
    for _ in 0..10 {
        learning.record_execution(
            "test",
            Some("cmd"),
            "cmd",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    for _ in 0..3 {
        learning.record_execution(
            "test",
            Some("cmd"),
            "cmd",
            ExecutionResult::Failed,
            100,
        ).await.unwrap();
    }

    for _ in 0..2 {
        learning.record_execution(
            "test",
            Some("cmd"),
            "cmd",
            ExecutionResult::Rejected,
            100,
        ).await.unwrap();
    }

    let summary = learning.get_analytics_summary(30).await.unwrap();

    assert_eq!(summary.total_commands, 15, "Should count all commands");
    assert_eq!(summary.successful_commands, 10, "Should count successful commands");
    assert_eq!(summary.failed_commands, 3, "Should count failed commands");
    assert_eq!(summary.rejected_commands, 2, "Should count rejected commands");

    let success_rate = summary.successful_commands as f64 / summary.total_commands as f64;
    assert!((success_rate - 0.666).abs() < 0.01, "Success rate should be ~66.6%");
}

#[tokio::test]
async fn test_command_analytics_with_context() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record execution with context hash
    learning.record_execution_with_context(
        "git status",
        Some("git status"),
        "git status",
        ExecutionResult::Success,
        50,
        "git_repo_context_hash",
        "claude",
    ).await.unwrap();

    // Verify it was recorded
    let patterns = learning.detect_patterns().await.unwrap();
    assert!(patterns.len() > 0);
}

#[tokio::test]
async fn test_learning_insights() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // Record patterns that should generate insights
    // User prefers verbose output
    for _ in 0..10 {
        learning.record_execution(
            "list files",
            Some("ls -la"),
            "ls -la",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    // User often works with Docker
    for _ in 0..5 {
        learning.record_execution(
            "docker containers",
            Some("docker ps"),
            "docker ps",
            ExecutionResult::Success,
            100,
        ).await.unwrap();
    }

    let insights = learning.generate_insights().await.unwrap();

    // Should detect user preferences
    assert!(insights.len() > 0, "Should generate insights from patterns");
}

#[tokio::test]
async fn test_concurrent_access() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let learning = LearningSystem::new(&db_path).await.unwrap();
    let learning = std::sync::Arc::new(tokio::sync::Mutex::new(learning));

    // Spawn multiple tasks that access the learning system concurrently
    let mut handles = vec![];

    for i in 0..10 {
        let learning_clone = learning.clone();
        let handle = tokio::spawn(async move {
            let mut learning = learning_clone.lock().await;
            learning.record_execution(
                &format!("test {}", i),
                Some(&format!("cmd {}", i)),
                &format!("cmd {}", i),
                ExecutionResult::Success,
                100,
            ).await
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }

    let learning = learning.lock().await;
    let patterns = learning.detect_patterns().await.unwrap();
    assert!(patterns.len() >= 10, "All concurrent writes should succeed");
}

#[tokio::test]
async fn test_delete_old_analytics() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let mut learning = LearningSystem::new(&db_path).await.unwrap();

    // This test would require manipulating timestamps
    // For now, just verify the method exists and runs
    let result = learning.delete_old_analytics(90).await;
    assert!(result.is_ok(), "Should be able to delete old analytics");
}
