// Unit tests for Context Detection System
// Tests Git detection, project type detection, directory classification, and language detection

use orbitd::context::{Context, GitContext, ProjectType, DirectoryType};
use std::path::PathBuf;
use tempfile::tempdir;
use std::fs;

#[tokio::test]
async fn test_context_detection_basic() {
    let temp_dir = tempdir().unwrap();
    let context = Context::detect(temp_dir.path()).await;

    assert!(context.is_ok(), "Context detection should succeed");

    let ctx = context.unwrap();
    assert_eq!(ctx.cwd, temp_dir.path());
    assert!(!ctx.username.is_empty(), "Username should be detected");
    assert!(!ctx.shell.is_empty(), "Shell should be detected");
}

#[tokio::test]
async fn test_git_detection_no_repo() {
    let temp_dir = tempdir().unwrap();
    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.git.is_none(), "Should not detect Git in non-repo directory");
}

#[tokio::test]
async fn test_git_detection_with_repo() {
    let temp_dir = tempdir().unwrap();

    // Initialize Git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.git.is_some(), "Should detect Git repo");

    let git = context.git.unwrap();
    assert!(git.current_branch.is_some(), "Should detect branch");
    assert_eq!(git.has_uncommitted_changes, false, "Should detect no uncommitted changes");
}

#[tokio::test]
async fn test_git_detection_with_changes() {
    let temp_dir = tempdir().unwrap();

    // Initialize Git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Create a file
    fs::write(temp_dir.path().join("test.txt"), "test content").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.git.is_some());
    let git = context.git.unwrap();
    assert_eq!(git.has_uncommitted_changes, true, "Should detect uncommitted changes");
}

#[tokio::test]
async fn test_git_detection_remote_url() {
    let temp_dir = tempdir().unwrap();

    // Initialize Git repo
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test User"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@example.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Add remote
    std::process::Command::new("git")
        .args(&["remote", "add", "origin", "https://github.com/test/repo.git"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.git.is_some());
    let git = context.git.unwrap();
    assert!(git.remote_url.is_some(), "Should detect remote URL");
    assert!(git.remote_url.unwrap().contains("github.com/test/repo"));
}

#[tokio::test]
async fn test_project_type_rust() {
    let temp_dir = tempdir().unwrap();

    // Create Cargo.toml
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.project_type.is_some());
    assert!(matches!(context.project_type.unwrap(), ProjectType::Rust));
}

#[tokio::test]
async fn test_project_type_nodejs() {
    let temp_dir = tempdir().unwrap();

    // Create package.json
    fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.project_type.is_some());
    let project_type = context.project_type.unwrap();
    assert!(matches!(project_type, ProjectType::Node));
}

#[tokio::test]
async fn test_project_type_python() {
    let temp_dir = tempdir().unwrap();

    // Create requirements.txt
    fs::write(temp_dir.path().join("requirements.txt"), "requests==2.28.0\n").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.project_type.is_some());
    assert!(matches!(context.project_type.unwrap(), ProjectType::Python));
}

#[tokio::test]
async fn test_project_type_go() {
    let temp_dir = tempdir().unwrap();

    // Create go.mod
    fs::write(temp_dir.path().join("go.mod"), "module test\n\ngo 1.20\n").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.project_type.is_some());
    assert!(matches!(context.project_type.unwrap(), ProjectType::Go));
}

#[tokio::test]
async fn test_project_type_docker() {
    let temp_dir = tempdir().unwrap();

    // Create Dockerfile
    fs::write(temp_dir.path().join("Dockerfile"), "FROM ubuntu:latest\n").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.project_type.is_some());
    assert!(matches!(context.project_type.unwrap(), ProjectType::Docker));
}

#[tokio::test]
async fn test_project_type_multiple() {
    let temp_dir = tempdir().unwrap();

    // Create multiple project markers
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    fs::write(temp_dir.path().join("package.json"), r#"{"name": "test"}"#).unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.project_type.is_some());
    // Should detect mixed project type or prioritize one
}

#[tokio::test]
async fn test_project_type_unknown() {
    let temp_dir = tempdir().unwrap();

    // No project markers
    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Should be None or Unknown
    assert!(context.project_type.is_none() || matches!(context.project_type.unwrap(), ProjectType::Unknown));
}

#[tokio::test]
async fn test_directory_type_home() {
    let home_dir = dirs::home_dir().unwrap();
    let context = Context::detect(&home_dir).await.unwrap();

    assert!(matches!(context.directory_type, DirectoryType::Home));
}

#[tokio::test]
async fn test_directory_type_root() {
    // Skip if not root
    if !nix::unistd::Uid::current().is_root() {
        return;
    }

    let root_path = PathBuf::from("/");
    let context = Context::detect(&root_path).await.unwrap();

    assert!(matches!(context.directory_type, DirectoryType::Root));
}

#[tokio::test]
async fn test_directory_type_temp() {
    let temp_dir = tempdir().unwrap();
    let context = Context::detect(temp_dir.path()).await.unwrap();

    // /tmp directories should be detected as Temp
    if temp_dir.path().starts_with("/tmp") {
        assert!(matches!(context.directory_type, DirectoryType::Temp));
    }
}

#[tokio::test]
async fn test_directory_type_project() {
    let temp_dir = tempdir().unwrap();

    // Create project marker
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Directory with project markers should be classified as Project
    assert!(matches!(context.directory_type, DirectoryType::Project));
}

#[tokio::test]
async fn test_language_detection_rust() {
    let temp_dir = tempdir().unwrap();

    // Create Rust files
    fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join("lib.rs"), "pub fn test() {}").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.languages.contains(&"Rust".to_string()));
}

#[tokio::test]
async fn test_language_detection_javascript() {
    let temp_dir = tempdir().unwrap();

    // Create JavaScript files
    fs::write(temp_dir.path().join("app.js"), "console.log('test');").unwrap();
    fs::write(temp_dir.path().join("utils.js"), "export function test() {}").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.languages.contains(&"JavaScript".to_string()));
}

#[tokio::test]
async fn test_language_detection_typescript() {
    let temp_dir = tempdir().unwrap();

    // Create TypeScript files
    fs::write(temp_dir.path().join("app.ts"), "const x: number = 5;").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.languages.contains(&"TypeScript".to_string()));
}

#[tokio::test]
async fn test_language_detection_python() {
    let temp_dir = tempdir().unwrap();

    // Create Python files
    fs::write(temp_dir.path().join("main.py"), "def test():\n    pass").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.languages.contains(&"Python".to_string()));
}

#[tokio::test]
async fn test_language_detection_multiple() {
    let temp_dir = tempdir().unwrap();

    // Create files in multiple languages
    fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();
    fs::write(temp_dir.path().join("app.js"), "console.log('test');").unwrap();
    fs::write(temp_dir.path().join("script.py"), "print('test')").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert!(context.languages.len() >= 3, "Should detect multiple languages");
    assert!(context.languages.contains(&"Rust".to_string()));
    assert!(context.languages.contains(&"JavaScript".to_string()));
    assert!(context.languages.contains(&"Python".to_string()));
}

#[tokio::test]
async fn test_language_detection_empty_dir() {
    let temp_dir = tempdir().unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    assert_eq!(context.languages.len(), 0, "Empty directory should have no languages");
}

#[tokio::test]
async fn test_context_hash_generation() {
    let temp_dir = tempdir().unwrap();

    let context1 = Context::detect(temp_dir.path()).await.unwrap();
    let hash1 = context1.generate_hash();

    let context2 = Context::detect(temp_dir.path()).await.unwrap();
    let hash2 = context2.generate_hash();

    assert_eq!(hash1, hash2, "Same context should generate same hash");
}

#[tokio::test]
async fn test_context_hash_different_dirs() {
    let temp_dir1 = tempdir().unwrap();
    let temp_dir2 = tempdir().unwrap();

    let context1 = Context::detect(temp_dir1.path()).await.unwrap();
    let hash1 = context1.generate_hash();

    let context2 = Context::detect(temp_dir2.path()).await.unwrap();
    let hash2 = context2.generate_hash();

    assert_ne!(hash1, hash2, "Different contexts should generate different hashes");
}

#[tokio::test]
async fn test_context_serialization() {
    let temp_dir = tempdir().unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Serialize to JSON
    let json = serde_json::to_string(&context);
    assert!(json.is_ok(), "Context should be serializable");

    // Deserialize
    let deserialized: Result<Context, _> = serde_json::from_str(&json.unwrap());
    assert!(deserialized.is_ok(), "Context should be deserializable");
}

#[tokio::test]
async fn test_context_enrichment() {
    let temp_dir = tempdir().unwrap();

    // Create a Rust project with Git
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@test.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();
    fs::write(temp_dir.path().join("main.rs"), "fn main() {}").unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Should have rich context
    assert!(context.git.is_some(), "Should detect Git");
    assert!(context.project_type.is_some(), "Should detect project type");
    assert!(context.languages.len() > 0, "Should detect languages");

    // Test context enrichment for AI prompt
    let enriched_prompt = context.enrich_prompt("test command");
    assert!(enriched_prompt.contains("Git"));
    assert!(enriched_prompt.contains("Rust"));
}

#[tokio::test]
async fn test_context_caching() {
    let temp_dir = tempdir().unwrap();

    // First detection
    let start1 = std::time::Instant::now();
    let _context1 = Context::detect(temp_dir.path()).await.unwrap();
    let duration1 = start1.elapsed();

    // Second detection (should use cache if implemented)
    let start2 = std::time::Instant::now();
    let _context2 = Context::detect(temp_dir.path()).await.unwrap();
    let duration2 = start2.elapsed();

    // Second detection might be faster due to caching
    // This is just a sanity check
    assert!(duration2 <= duration1 * 2);
}

#[tokio::test]
async fn test_environment_variables() {
    let temp_dir = tempdir().unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Should capture some environment variables
    assert!(context.environment_vars.len() > 0, "Should capture environment variables");

    // Common variables that should be present
    assert!(context.environment_vars.contains_key("PATH") ||
            context.environment_vars.contains_key("HOME") ||
            context.environment_vars.contains_key("USER"));
}

#[tokio::test]
async fn test_shell_detection() {
    let temp_dir = tempdir().unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Should detect current shell
    assert!(!context.shell.is_empty());
    assert!(
        context.shell == "bash" ||
        context.shell == "zsh" ||
        context.shell == "fish" ||
        context.shell == "sh" ||
        context.shell.contains("unknown")
    );
}

#[tokio::test]
async fn test_username_detection() {
    let temp_dir = tempdir().unwrap();

    let context = Context::detect(temp_dir.path()).await.unwrap();

    // Should detect username
    assert!(!context.username.is_empty());
    assert!(context.username.len() > 0);
}
