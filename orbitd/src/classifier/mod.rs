use anyhow::Result;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::debug;

use crate::config::Config;
use crate::context::Context;
use crate::learning::{LearnedCommand, LearningEngine};

pub struct CommandClassifier {
    config: Arc<Config>,
    known_commands: HashSet<String>,
    learning_engine: Arc<LearningEngine>,
}

#[derive(Debug, Clone)]
pub enum CommandType {
    Known,
    LearnedPattern(LearnedCommand),
    NaturalLanguage,
    Ambiguous,
}

impl CommandClassifier {
    pub async fn new(config: Arc<Config>, learning_engine: Arc<LearningEngine>) -> Result<Self> {
        let mut classifier = Self {
            config,
            known_commands: HashSet::new(),
            learning_engine,
        };

        // Build cache of known commands
        classifier.build_known_commands_cache().await?;

        Ok(classifier)
    }

    pub async fn classify(&self, input: &str, context: &Context) -> Result<CommandType> {
        let first_word = input.split_whitespace().next().unwrap_or("");

        // 1. Check if it's a known command
        if self.is_known_command(first_word) {
            debug!("Classified as: Known command");
            return Ok(CommandType::Known);
        }

        // 2. Check if we have a learned pattern
        if let Some(pattern) = self.learning_engine.find_similar(input, context).await? {
            if pattern.confidence > self.config.learning.confidence_threshold {
                debug!(
                    "Classified as: Learned pattern (confidence: {})",
                    pattern.confidence
                );
                return Ok(CommandType::LearnedPattern(pattern));
            }
        }

        // 3. Check if it looks like natural language
        if self.looks_like_natural_language(input) {
            debug!("Classified as: Natural language");
            return Ok(CommandType::NaturalLanguage);
        }

        // 4. Ambiguous - might be typo or natural language
        debug!("Classified as: Ambiguous");
        Ok(CommandType::Ambiguous)
    }

    fn is_known_command(&self, cmd: &str) -> bool {
        // Check cache
        if self.known_commands.contains(cmd) {
            return true;
        }

        // Check shell builtins
        if self.is_shell_builtin(cmd) {
            return true;
        }

        // Check if it's a path (./script or /usr/bin/app)
        if cmd.starts_with("./") || cmd.starts_with('/') {
            return true;
        }

        false
    }

    fn is_shell_builtin(&self, cmd: &str) -> bool {
        // Common shell builtins
        matches!(
            cmd,
            "cd" | "export"
                | "alias"
                | "source"
                | "."
                | "echo"
                | "pwd"
                | "exit"
                | "history"
                | "jobs"
                | "fg"
                | "bg"
                | "kill"
                | "wait"
                | "read"
                | "test"
                | "["
                | "eval"
                | "exec"
                | "set"
                | "unset"
                | "shift"
                | "return"
                | "break"
                | "continue"
                | "trap"
                | "ulimit"
                | "umask"
                | "type"
                | "command"
                | "builtin"
                | "enable"
                | "help"
                | "let"
                | "local"
                | "declare"
                | "typeset"
                | "readonly"
                | "unalias"
        )
    }

    fn looks_like_natural_language(&self, input: &str) -> bool {
        let input_lower = input.to_lowercase();

        // Question words
        let question_words = [
            "what", "how", "why", "when", "where", "who", "tell", "show", "find", "list", "get",
            "explain", "describe", "can you",
        ];

        // Check for question words at start
        for word in &question_words {
            if input_lower.starts_with(word) {
                return true;
            }
        }

        // Contains question mark
        if input.contains('?') {
            return true;
        }

        // Multiple words with spaces (> 4 words)
        if input.split_whitespace().count() > 4 {
            return true;
        }

        // Contains common conversational phrases
        let conversational = [
            "i want",
            "i need",
            "please",
            "could you",
            "would you",
            "can you",
            "help me",
            "show me",
            "tell me",
            "give me",
        ];

        for phrase in &conversational {
            if input_lower.contains(phrase) {
                return true;
            }
        }

        false
    }

    async fn build_known_commands_cache(&mut self) -> Result<()> {
        // Get all executables in PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for path_dir in path_var.split(':') {
                if let Ok(entries) = std::fs::read_dir(path_dir) {
                    for entry in entries.flatten() {
                        if let Ok(file_name) = entry.file_name().into_string() {
                            // Check if executable
                            if entry.path().is_file() {
                                #[cfg(unix)]
                                {
                                    use std::os::unix::fs::PermissionsExt;
                                    if let Ok(metadata) = entry.metadata() {
                                        let permissions = metadata.permissions();
                                        // Check if owner has execute permission
                                        if permissions.mode() & 0o100 != 0 {
                                            self.known_commands.insert(file_name);
                                        }
                                    }
                                }

                                #[cfg(not(unix))]
                                {
                                    self.known_commands.insert(file_name);
                                }
                            }
                        }
                    }
                }
            }
        }

        debug!("Cached {} known commands", self.known_commands.len());

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::Context;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_classifier() -> CommandClassifier {
        // Use unique temp dir for each test to avoid parallel test conflicts
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

        // Set unique environment variables for this test instance
        std::env::set_var("ORBIT_CONFIG", config_path.to_str().unwrap());
        std::env::set_var("ORBIT_DATA_DIR", data_dir.to_str().unwrap());
        std::env::set_var("ORBIT_DEV_MODE", "1");

        // Keep temp_dir alive for the duration of the test
        std::mem::forget(temp_dir);

        let config = Arc::new(crate::config::Config::load().await.unwrap());

        // Let SQLite create the database file itself (don't pre-create empty file)
        // The learning engine will initialize the database with proper schema
        let learning_engine = Arc::new(
            crate::learning::LearningEngine::new(config.clone())
                .await
                .unwrap(),
        );

        CommandClassifier::new(config, learning_engine)
            .await
            .unwrap()
    }

    fn create_test_context() -> Context {
        use crate::context::{DirectoryType, ProjectType};
        Context {
            os_name: "linux".to_string(),
            os_version: "5.0.0".to_string(),
            shell_name: "bash".to_string(),
            shell_version: "5.0".to_string(),
            pwd: std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/")),
            username: "testuser".to_string(),
            git_context: None,
            detected_languages: vec![],
            recent_commands: vec![],
            project_type: None,
            directory_type: DirectoryType::Other,
        }
    }

    // ========== Initialization Tests ==========

    #[tokio::test]
    async fn test_classifier_initialization() {
        let classifier = create_test_classifier().await;

        assert!(
            classifier.config.learning.confidence_threshold > 0.0,
            "Classifier should have valid confidence threshold"
        );

        // Should have built known commands cache from PATH
        // We can't assert exact count because it depends on the system
        // but we can check that it found at least some common commands
        assert!(
            !classifier.known_commands.is_empty(),
            "Should have cached some commands from PATH"
        );
    }

    #[tokio::test]
    async fn test_build_known_commands_cache() {
        let classifier = create_test_classifier().await;

        // Common Unix commands that should be present
        let common_commands = ["ls", "cat", "grep", "sed", "awk"];

        let mut found_count = 0;
        for cmd in &common_commands {
            if classifier.known_commands.contains(*cmd) {
                found_count += 1;
            }
        }

        assert!(
            found_count > 0,
            "Should find at least some common Unix commands"
        );
    }

    // ========== Known Command Detection Tests ==========

    #[tokio::test]
    async fn test_is_known_command_shell_builtins() {
        let classifier = create_test_classifier().await;

        // Test common shell builtins
        assert!(
            classifier.is_known_command("cd"),
            "Should recognize 'cd' as builtin"
        );
        assert!(
            classifier.is_known_command("echo"),
            "Should recognize 'echo' as builtin"
        );
        assert!(
            classifier.is_known_command("pwd"),
            "Should recognize 'pwd' as builtin"
        );
        assert!(
            classifier.is_known_command("export"),
            "Should recognize 'export' as builtin"
        );
        assert!(
            classifier.is_known_command("alias"),
            "Should recognize 'alias' as builtin"
        );
    }

    #[tokio::test]
    async fn test_is_known_command_path_executables() {
        let classifier = create_test_classifier().await;

        // Path-based commands should be recognized
        assert!(
            classifier.is_known_command("./script.sh"),
            "Should recognize relative path commands"
        );
        assert!(
            classifier.is_known_command("/usr/bin/python3"),
            "Should recognize absolute path commands"
        );
        assert!(
            classifier.is_known_command("./test"),
            "Should recognize ./ prefix"
        );
        assert!(
            classifier.is_known_command("/bin/bash"),
            "Should recognize / prefix"
        );
    }

    #[tokio::test]
    async fn test_is_known_command_from_cache() {
        let classifier = create_test_classifier().await;

        // If ls is in PATH (it should be), it should be in the cache
        if classifier.known_commands.contains("ls") {
            assert!(
                classifier.is_known_command("ls"),
                "Should recognize cached command 'ls'"
            );
        }
    }

    #[tokio::test]
    async fn test_is_known_command_false_cases() {
        let classifier = create_test_classifier().await;

        // These should NOT be recognized as known commands
        assert!(
            !classifier.is_known_command("asdfqwerzxcv"),
            "Should not recognize random string as command"
        );
        assert!(
            !classifier.is_known_command("find files in directory"),
            "Should not recognize natural language as command"
        );
        assert!(
            !classifier.is_known_command("nonexistent_command_xyz"),
            "Should not recognize nonexistent command"
        );
    }

    // ========== Classification Tests ==========

    #[tokio::test]
    async fn test_classify_known_command() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // Test with shell builtin
        let result = classifier.classify("cd /tmp", &context).await.unwrap();
        assert!(
            matches!(result, CommandType::Known),
            "Should classify 'cd' as Known"
        );

        // Test with path command
        let result = classifier
            .classify("./script.sh arg1 arg2", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::Known),
            "Should classify './script.sh' as Known"
        );
    }

    #[tokio::test]
    async fn test_classify_natural_language_question_words() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // Test question words - avoiding words that are also commands
        let nl_inputs = [
            "what files are in this directory",
            "how do I list all files",
            "why is this not working",
            "when was this file created",
            "where is the config file",
            "explain this command to me",
            "describe the file contents",
            "can you help me with this",
        ];

        for input in &nl_inputs {
            let result = classifier.classify(input, &context).await.unwrap();
            assert!(
                matches!(result, CommandType::NaturalLanguage),
                "Should classify '{}' as NaturalLanguage",
                input
            );
        }
    }

    #[tokio::test]
    async fn test_classify_natural_language_question_mark() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        let result = classifier
            .classify("what is this?", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::NaturalLanguage),
            "Should classify question with '?' as NaturalLanguage"
        );

        let result = classifier
            .classify("how do I do this?", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::NaturalLanguage),
            "Should classify question ending with '?' as NaturalLanguage"
        );
    }

    #[tokio::test]
    async fn test_classify_natural_language_word_count() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // More than 4 words should be natural language
        let result = classifier
            .classify("this is a very long sentence", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::NaturalLanguage),
            "Should classify > 4 words as NaturalLanguage"
        );

        let result = classifier
            .classify("copy all files to backup directory", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::NaturalLanguage),
            "Should classify long phrase as NaturalLanguage"
        );
    }

    #[tokio::test]
    async fn test_classify_natural_language_conversational() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        let conversational = [
            "i want to see all files",
            "i need some help with this",
            "please show me the files",
            "could you list the files",
            "would you help me",
            "can you find this for me",
        ];

        for input in &conversational {
            let result = classifier.classify(input, &context).await.unwrap();
            assert!(
                matches!(result, CommandType::NaturalLanguage),
                "Should classify '{}' as NaturalLanguage",
                input
            );
        }
    }

    #[tokio::test]
    async fn test_classify_learned_pattern() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // First, add a learned pattern with high confidence to the learning engine
        // Use a phrase that doesn't start with a known command
        let natural_input = "search for text in files";
        let learned_command = "grep -r text .";

        classifier
            .learning_engine
            .record_success(natural_input, learned_command, &context)
            .await
            .unwrap();

        // Record multiple times to boost confidence above threshold
        for _ in 0..5 {
            classifier
                .learning_engine
                .record_success(natural_input, learned_command, &context)
                .await
                .unwrap();
        }

        // Now classify - should find the learned pattern
        let result = classifier.classify(natural_input, &context).await.unwrap();

        match result {
            CommandType::LearnedPattern(pattern) => {
                assert_eq!(pattern.natural_input, natural_input);
                assert_eq!(pattern.learned_command, learned_command);
                assert!(
                    pattern.confidence >= 0.7,
                    "Pattern confidence should be above threshold"
                );
            }
            _ => panic!("Expected LearnedPattern, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_classify_learned_pattern_below_threshold() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // Add a learned pattern with low confidence (below threshold)
        let natural_input = "do something random";
        let learned_command = "echo random";

        classifier
            .learning_engine
            .record_success(natural_input, learned_command, &context)
            .await
            .unwrap();

        // Don't boost confidence - it should be below threshold (0.6 initially, threshold is 0.7)

        // Should NOT classify as learned pattern because confidence is too low
        let result = classifier.classify(natural_input, &context).await.unwrap();

        assert!(
            !matches!(result, CommandType::LearnedPattern(_)),
            "Should not classify low-confidence pattern as LearnedPattern"
        );
    }

    #[tokio::test]
    async fn test_classify_ambiguous() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // Short phrases that don't match any criteria should be Ambiguous
        // Avoid words that are commands
        let ambiguous_inputs = [
            "unknown_command_xyz",
            "random_thing_abc",
            "do something quick",
            "perform action",
        ];

        for input in &ambiguous_inputs {
            let result = classifier.classify(input, &context).await.unwrap();

            // Could be Ambiguous or NaturalLanguage depending on heuristics
            // But should NOT be Known or LearnedPattern
            match result {
                CommandType::Known => panic!("Should not classify '{}' as Known", input),
                CommandType::LearnedPattern(_) => {
                    panic!("Should not classify '{}' as LearnedPattern", input)
                }
                CommandType::Ambiguous | CommandType::NaturalLanguage => {
                    // Expected - either is acceptable
                }
            }
        }
    }

    // ========== Natural Language Heuristics Tests ==========
    // These are tested implicitly through the classification tests above

    #[tokio::test]
    async fn test_natural_language_case_insensitive() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // Question words should work case-insensitively
        let result = classifier
            .classify("WHAT files are here", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::NaturalLanguage),
            "Uppercase question word should be detected"
        );

        let result = classifier
            .classify("How do I do this", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::NaturalLanguage),
            "Mixed case question word should be detected"
        );
    }

    #[tokio::test]
    async fn test_shell_builtin_completeness() {
        let classifier = create_test_classifier().await;

        // Test comprehensive list of builtins
        let builtins = [
            "cd", "export", "alias", "source", ".", "echo", "pwd", "exit", "history", "jobs", "fg",
            "bg", "kill", "wait", "read", "test", "[", "eval", "exec", "set", "unset", "shift",
            "return", "break", "continue", "trap", "ulimit", "umask", "type", "command", "builtin",
            "enable", "help", "let", "local", "declare", "typeset", "readonly", "unalias",
        ];

        for builtin in &builtins {
            assert!(
                classifier.is_known_command(builtin),
                "Should recognize '{}' as shell builtin",
                builtin
            );
        }
    }

    #[tokio::test]
    async fn test_shell_builtin_false_cases() {
        let classifier = create_test_classifier().await;

        // These should NOT be recognized as builtins (but might be in PATH cache)
        // We'll check against commands that definitely don't exist
        assert!(
            !classifier.is_known_command("nonexistent_xyz_123"),
            "'nonexistent_xyz_123' should not be recognized"
        );
        assert!(
            !classifier.is_known_command("random_command_abc"),
            "'random_command_abc' should not be recognized"
        );
    }

    // ========== Edge Cases ==========

    #[tokio::test]
    async fn test_classify_empty_input() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        let result = classifier.classify("", &context).await.unwrap();

        // Empty input should be Ambiguous (no first word to check)
        assert!(
            matches!(result, CommandType::Ambiguous),
            "Empty input should be Ambiguous"
        );
    }

    #[tokio::test]
    async fn test_classify_whitespace_only() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        let result = classifier.classify("   ", &context).await.unwrap();

        // Whitespace-only should be Ambiguous
        assert!(
            matches!(result, CommandType::Ambiguous),
            "Whitespace-only should be Ambiguous"
        );
    }

    #[tokio::test]
    async fn test_classify_with_arguments() {
        let classifier = create_test_classifier().await;
        let context = create_test_context();

        // Command with arguments should still be recognized
        let result = classifier
            .classify("cd /tmp/test/directory", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::Known),
            "Should classify 'cd' with arguments as Known"
        );

        let result = classifier
            .classify("echo hello world", &context)
            .await
            .unwrap();
        assert!(
            matches!(result, CommandType::Known),
            "Should classify 'echo' with arguments as Known"
        );
    }
}
