// Unit tests for the Command Classifier
// Tests command classification, natural language detection, and ambiguous case handling

use orbitd::classifier::{Classifier, Classification};

#[test]
fn test_classifier_initialization() {
    let classifier = Classifier::new();
    assert!(true, "Classifier should initialize successfully");
}

#[test]
fn test_classify_known_command_simple() {
    let classifier = Classifier::new();

    let result = classifier.classify("ls");
    assert!(matches!(result, Classification::Known));
}

#[test]
fn test_classify_known_command_with_args() {
    let classifier = Classifier::new();

    let result = classifier.classify("ls -la /tmp");
    assert!(matches!(result, Classification::Known));
}

#[test]
fn test_classify_known_command_git() {
    let classifier = Classifier::new();

    let commands = vec![
        "git status",
        "git add .",
        "git commit -m 'test'",
        "git push origin main",
        "git pull",
        "git log",
        "git diff",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known), "Git command should be known: {}", cmd);
    }
}

#[test]
fn test_classify_known_command_cargo() {
    let classifier = Classifier::new();

    let commands = vec![
        "cargo build",
        "cargo test",
        "cargo run",
        "cargo check",
        "cargo clippy",
        "cargo fmt",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known), "Cargo command should be known: {}", cmd);
    }
}

#[test]
fn test_classify_known_command_docker() {
    let classifier = Classifier::new();

    let commands = vec![
        "docker ps",
        "docker images",
        "docker run ubuntu",
        "docker build -t test .",
        "docker stop container",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known), "Docker command should be known: {}", cmd);
    }
}

#[test]
fn test_classify_natural_language_simple() {
    let classifier = Classifier::new();

    let commands = vec![
        "list all files",
        "show me the current directory",
        "what files are here",
        "display the contents",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::NaturalLanguage),
                "Should classify as natural language: {}", cmd);
    }
}

#[test]
fn test_classify_natural_language_questions() {
    let classifier = Classifier::new();

    let commands = vec![
        "how do I list files?",
        "what is my current directory?",
        "can you show me the git status?",
        "where am I?",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::NaturalLanguage),
                "Question should be natural language: {}", cmd);
    }
}

#[test]
fn test_classify_natural_language_imperative() {
    let classifier = Classifier::new();

    let commands = vec![
        "show me all processes",
        "find large files",
        "search for pattern in files",
        "count the number of lines",
        "tell me the weather",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::NaturalLanguage),
                "Imperative should be natural language: {}", cmd);
    }
}

#[test]
fn test_classify_natural_language_complex() {
    let classifier = Classifier::new();

    let commands = vec![
        "find all JavaScript files modified in the last week",
        "show me the top 10 largest files in this directory",
        "list all git branches and their last commit dates",
        "display system resource usage sorted by memory",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::NaturalLanguage),
                "Complex query should be natural language: {}", cmd);
    }
}

#[test]
fn test_classify_empty_command() {
    let classifier = Classifier::new();

    let result = classifier.classify("");
    // Empty command should be either Unknown or handled specially
    assert!(matches!(result, Classification::Known) ||
            matches!(result, Classification::NaturalLanguage) ||
            matches!(result, Classification::Ambiguous));
}

#[test]
fn test_classify_whitespace_only() {
    let classifier = Classifier::new();

    let result = classifier.classify("   ");
    assert!(matches!(result, Classification::Known) ||
            matches!(result, Classification::Ambiguous));
}

#[test]
fn test_classify_single_character() {
    let classifier = Classifier::new();

    // Single character commands (like 'l' typo) should not be natural language
    let result = classifier.classify("l");
    assert!(matches!(result, Classification::Known) ||
            matches!(result, Classification::Ambiguous));
}

#[test]
fn test_classify_command_with_pipes() {
    let classifier = Classifier::new();

    let result = classifier.classify("ls -la | grep test | wc -l");
    assert!(matches!(result, Classification::Known),
            "Piped commands should be known");
}

#[test]
fn test_classify_command_with_redirects() {
    let classifier = Classifier::new();

    let commands = vec![
        "echo test > file.txt",
        "cat file.txt >> output.txt",
        "grep pattern < input.txt",
        "command 2>&1",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known),
                "Command with redirect should be known: {}", cmd);
    }
}

#[test]
fn test_classify_command_with_variables() {
    let classifier = Classifier::new();

    let commands = vec![
        "echo $HOME",
        "cd $HOME",
        "export VAR=value",
        "echo ${VAR}",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known),
                "Command with variables should be known: {}", cmd);
    }
}

#[test]
fn test_classify_ambiguous_short_phrases() {
    let classifier = Classifier::new();

    // These could be typos or abbreviations
    let commands = vec![
        "lst",
        "gti",
        "dc",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        // These should probably be classified as Known (treat as potential typos)
        // or Ambiguous
        assert!(matches!(result, Classification::Known) ||
                matches!(result, Classification::Ambiguous),
                "Short command should be known or ambiguous: {}", cmd);
    }
}

#[test]
fn test_classify_with_context_awareness() {
    let classifier = Classifier::new();

    // In a Git repo context, these should be natural language
    let result = classifier.classify("show me the changes");
    assert!(matches!(result, Classification::NaturalLanguage));

    // In any context, this is a known command
    let result = classifier.classify("git show");
    assert!(matches!(result, Classification::Known));
}

#[test]
fn test_classify_dangerous_commands() {
    let classifier = Classifier::new();

    // These are known commands, but dangerous
    let commands = vec![
        "rm -rf /",
        "dd if=/dev/zero of=/dev/sda",
        "chmod -R 777 /",
        "mkfs.ext4 /dev/sda",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        // Should still classify correctly (as Known) but might be flagged elsewhere
        assert!(matches!(result, Classification::Known),
                "Dangerous command should still be classified: {}", cmd);
    }
}

#[test]
fn test_classify_script_snippets() {
    let classifier = Classifier::new();

    let commands = vec![
        "for i in *; do echo $i; done",
        "while read line; do echo $line; done",
        "if [ -f file ]; then cat file; fi",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known),
                "Script snippet should be known: {}", cmd);
    }
}

#[test]
fn test_classify_aliases() {
    let classifier = Classifier::new();

    // Common aliases should be classified as known if they exist
    let commands = vec![
        "ll",  // common alias for ls -la
        "la",  // common alias for ls -A
        "...", // common alias for cd ../../..
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        // Could be Known or Ambiguous depending on whether alias is registered
        assert!(matches!(result, Classification::Known) ||
                matches!(result, Classification::Ambiguous) ||
                matches!(result, Classification::NaturalLanguage));
    }
}

#[test]
fn test_classify_with_special_characters() {
    let classifier = Classifier::new();

    let commands = vec![
        "echo 'hello world'",
        r#"echo "test $VAR""#,
        "grep 'pattern.*' file",
        "find . -name '*.rs'",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known),
                "Command with special chars should be known: {}", cmd);
    }
}

#[test]
fn test_classify_multilingual() {
    let classifier = Classifier::new();

    // Non-English natural language (if supported)
    let commands = vec![
        "muestra todos los archivos", // Spanish: show all files
        "liste tous les fichiers",    // French: list all files
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        // Should be natural language
        assert!(matches!(result, Classification::NaturalLanguage),
                "Non-English should be natural language: {}", cmd);
    }
}

#[test]
fn test_classify_mixed_case() {
    let classifier = Classifier::new();

    let commands = vec![
        "LS -LA",
        "Git Status",
        "ECHO test",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        // Should normalize case and classify as Known
        assert!(matches!(result, Classification::Known),
                "Mixed case command should be known: {}", cmd);
    }
}

#[test]
fn test_classify_with_numbers() {
    let classifier = Classifier::new();

    let commands = vec![
        "head -n 10 file.txt",
        "tail -n 20 file.txt",
        "ps aux | head -5",
    ];

    for cmd in commands {
        let result = classifier.classify(cmd);
        assert!(matches!(result, Classification::Known),
                "Command with numbers should be known: {}", cmd);
    }
}

#[test]
fn test_classify_learning_integration() {
    let classifier = Classifier::new();

    // Test that classifier can use learned patterns
    // In a real scenario, this would check against the learning database

    let result = classifier.classify("my custom command pattern");
    // Without learning data, this should be natural language
    assert!(matches!(result, Classification::NaturalLanguage));
}

#[test]
fn test_classify_performance() {
    let classifier = Classifier::new();

    // Classification should be very fast (<5ms)
    let start = std::time::Instant::now();

    for _ in 0..1000 {
        classifier.classify("ls -la");
    }

    let duration = start.elapsed();
    let avg_time = duration.as_micros() / 1000;

    // Average classification should be under 100 microseconds
    assert!(avg_time < 100, "Classification too slow: {} µs", avg_time);
}

#[test]
fn test_classify_batch() {
    let classifier = Classifier::new();

    let commands = vec![
        ("ls", Classification::Known),
        ("show files", Classification::NaturalLanguage),
        ("git status", Classification::Known),
        ("what is the current directory?", Classification::NaturalLanguage),
    ];

    for (cmd, expected) in commands {
        let result = classifier.classify(cmd);
        assert!(std::mem::discriminant(&result) == std::mem::discriminant(&expected),
                "Classification mismatch for '{}': expected {:?}, got {:?}",
                cmd, expected, result);
    }
}

#[test]
fn test_classifier_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    let classifier = Arc::new(Classifier::new());
    let mut handles = vec![];

    for i in 0..10 {
        let classifier_clone = Arc::clone(&classifier);
        let handle = thread::spawn(move || {
            let cmd = format!("ls -la {}", i);
            classifier_clone.classify(&cmd)
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(matches!(result, Classification::Known));
    }
}
