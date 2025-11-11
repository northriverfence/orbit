use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Arc;

use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    pub os_name: String,
    pub os_version: String,
    pub shell_name: String,
    pub shell_version: String,
    pub pwd: PathBuf,
    pub username: String,
    pub git_context: Option<GitContext>,
    pub detected_languages: Vec<String>,
    pub recent_commands: Vec<String>,
    pub project_type: Option<ProjectType>,
    pub directory_type: DirectoryType,
}

/// Detected project type based on files in directory
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProjectType {
    Rust,           // Cargo.toml
    Node,           // package.json
    Python,         // setup.py, requirements.txt, pyproject.toml
    Go,             // go.mod
    Java,           // pom.xml, build.gradle
    CSharp,         // *.csproj, *.sln
    Ruby,           // Gemfile
    Php,            // composer.json
    Docker,         // Dockerfile
    Kubernetes,     // *.yaml with k8s resources
    Terraform,      // *.tf
    Unknown,
}

/// Type of directory being worked in
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DirectoryType {
    Home,           // User home directory
    Root,           // System root
    Temp,           // Temporary directory
    System,         // /usr, /etc, /var, etc.
    Project,        // Contains project files
    Downloads,      // Downloads folder
    Documents,      // Documents folder
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub repo_name: String,
    pub current_branch: String,
    pub has_uncommitted_changes: bool,
    pub remote_url: Option<String>,
    pub ahead_behind: Option<(usize, usize)>, // (ahead, behind) commits
    pub total_commits: Option<usize>,
    pub last_commit_message: Option<String>,
}

pub struct ContextEngine {
    _config: Arc<Config>,
}

impl ContextEngine {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { _config: config })
    }

    pub async fn get_context(&self) -> Result<Context> {
        let pwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));

        let username = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "unknown".to_string());

        let shell_name = std::env::var("SHELL").unwrap_or_else(|_| "bash".to_string());

        let git_context = Self::detect_git_context(&pwd);
        let project_type = Self::detect_project_type(&pwd);
        let directory_type = Self::detect_directory_type(&pwd, &username);
        let detected_languages = Self::detect_languages(&pwd);

        Ok(Context {
            os_name: std::env::consts::OS.to_string(),
            os_version: Self::get_os_version(),
            shell_name,
            shell_version: "unknown".to_string(),
            pwd: pwd.clone(),
            username,
            git_context,
            detected_languages,
            recent_commands: vec![],
            project_type,
            directory_type,
        })
    }

    fn get_os_version() -> String {
        #[cfg(target_os = "linux")]
        {
            // Try to read /etc/os-release
            if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
                for line in content.lines() {
                    if line.starts_with("PRETTY_NAME=") {
                        return line
                            .trim_start_matches("PRETTY_NAME=")
                            .trim_matches('"')
                            .to_string();
                    }
                }
            }
            "Linux".to_string()
        }

        #[cfg(target_os = "macos")]
        {
            "macOS".to_string()
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            "Unknown".to_string()
        }
    }

    fn detect_git_context(path: &PathBuf) -> Option<GitContext> {
        // Try to open git repository
        if let Ok(repo) = git2::Repository::discover(path) {
            let head = repo.head().ok()?;
            let branch = head.shorthand()?.to_string();

            let repo_name = repo.workdir()?.file_name()?.to_string_lossy().to_string();

            // Check for uncommitted changes
            let has_uncommitted_changes = repo
                .statuses(None)
                .map(|statuses| !statuses.is_empty())
                .unwrap_or(false);

            // Get remote URL
            let remote_url = repo
                .find_remote("origin")
                .ok()
                .and_then(|remote| remote.url().map(|s| s.to_string()));

            // Get ahead/behind counts
            let ahead_behind = Self::get_ahead_behind(&repo, &branch);

            // Get total commit count
            let total_commits = Self::get_commit_count(&repo);

            // Get last commit message
            let last_commit_message = head
                .peel_to_commit()
                .ok()
                .and_then(|commit| commit.message().map(|s| s.to_string()));

            Some(GitContext {
                repo_name,
                current_branch: branch,
                has_uncommitted_changes,
                remote_url,
                ahead_behind,
                total_commits,
                last_commit_message,
            })
        } else {
            None
        }
    }

    fn get_ahead_behind(repo: &git2::Repository, branch: &str) -> Option<(usize, usize)> {
        let local_oid = repo.refname_to_id(&format!("refs/heads/{}", branch)).ok()?;

        // Try to get upstream branch
        let local_branch = repo.find_branch(branch, git2::BranchType::Local).ok()?;
        let upstream = local_branch.upstream().ok()?;
        let upstream_oid = upstream.get().target()?;

        repo.graph_ahead_behind(local_oid, upstream_oid).ok()
    }

    fn get_commit_count(repo: &git2::Repository) -> Option<usize> {
        let mut revwalk = repo.revwalk().ok()?;
        revwalk.push_head().ok()?;
        Some(revwalk.count())
    }

    /// Detect project type based on files in directory
    fn detect_project_type(path: &PathBuf) -> Option<ProjectType> {
        // Check for project files
        if path.join("Cargo.toml").exists() {
            return Some(ProjectType::Rust);
        }
        if path.join("package.json").exists() {
            return Some(ProjectType::Node);
        }
        if path.join("go.mod").exists() {
            return Some(ProjectType::Go);
        }
        if path.join("setup.py").exists()
            || path.join("requirements.txt").exists()
            || path.join("pyproject.toml").exists() {
            return Some(ProjectType::Python);
        }
        if path.join("pom.xml").exists() || path.join("build.gradle").exists() {
            return Some(ProjectType::Java);
        }
        if path.join("Gemfile").exists() {
            return Some(ProjectType::Ruby);
        }
        if path.join("composer.json").exists() {
            return Some(ProjectType::Php);
        }
        if path.join("Dockerfile").exists() {
            return Some(ProjectType::Docker);
        }

        // Check for .csproj or .sln files
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "csproj" || ext == "sln" {
                        return Some(ProjectType::CSharp);
                    }
                    if ext == "tf" {
                        return Some(ProjectType::Terraform);
                    }
                }
            }
        }

        None
    }

    /// Detect directory type
    fn detect_directory_type(path: &PathBuf, _username: &str) -> DirectoryType {
        let path_str = path.to_string_lossy();

        // Check for specific directory types
        if path == &PathBuf::from("/") {
            return DirectoryType::Root;
        }

        if path_str.starts_with("/tmp") || path_str.starts_with("/var/tmp") {
            return DirectoryType::Temp;
        }

        if path_str.starts_with("/usr") || path_str.starts_with("/etc") || path_str.starts_with("/var") {
            return DirectoryType::System;
        }

        // Check if it's home directory
        if let Ok(home) = std::env::var("HOME") {
            if path == &PathBuf::from(&home) {
                return DirectoryType::Home;
            }

            // Check for common folders
            if path_str.contains("Downloads") {
                return DirectoryType::Downloads;
            }
            if path_str.contains("Documents") {
                return DirectoryType::Documents;
            }
        }

        // Check if it's a project directory
        if Self::detect_project_type(path).is_some() {
            return DirectoryType::Project;
        }

        DirectoryType::Other
    }

    /// Detect programming languages in directory
    fn detect_languages(path: &PathBuf) -> Vec<String> {
        let mut languages = Vec::new();

        if let Ok(entries) = std::fs::read_dir(path) {
            let mut extensions = HashSet::new();

            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    extensions.insert(ext.to_string_lossy().to_string());
                }
            }

            // Map extensions to languages
            if extensions.contains("rs") {
                languages.push("Rust".to_string());
            }
            if extensions.contains("js") || extensions.contains("ts") || extensions.contains("jsx") || extensions.contains("tsx") {
                languages.push("JavaScript/TypeScript".to_string());
            }
            if extensions.contains("py") {
                languages.push("Python".to_string());
            }
            if extensions.contains("go") {
                languages.push("Go".to_string());
            }
            if extensions.contains("java") {
                languages.push("Java".to_string());
            }
            if extensions.contains("rb") {
                languages.push("Ruby".to_string());
            }
            if extensions.contains("php") {
                languages.push("PHP".to_string());
            }
            if extensions.contains("cs") {
                languages.push("C#".to_string());
            }
            if extensions.contains("c") || extensions.contains("h") {
                languages.push("C".to_string());
            }
            if extensions.contains("cpp") || extensions.contains("hpp") || extensions.contains("cc") {
                languages.push("C++".to_string());
            }
            if extensions.contains("sh") || extensions.contains("bash") {
                languages.push("Shell".to_string());
            }
            if extensions.contains("yaml") || extensions.contains("yml") {
                languages.push("YAML".to_string());
            }
            if extensions.contains("json") {
                languages.push("JSON".to_string());
            }
            if extensions.contains("toml") {
                languages.push("TOML".to_string());
            }
        }

        languages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn create_test_config() -> Arc<Config> {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        let data_dir = temp_dir.path().join("data");
        std::fs::create_dir_all(&data_dir).unwrap();

        std::fs::write(
            &config_path,
            format!(
                r#"
license:
  key: "test"
daemon:
  socket_path: "{}/orbit.sock"
provider_mode: manual
default_provider: "test"
providers:
  test:
    api_key: "test"
learning:
  enabled: true
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

        std::env::set_var("ORBIT_CONFIG", config_path.to_str().unwrap());
        std::env::set_var("ORBIT_DEV_MODE", "1");
        std::mem::forget(temp_dir);

        Arc::new(Config::load().await.unwrap())
    }

    #[tokio::test]
    async fn test_context_engine_initialization() {
        let config = create_test_config().await;
        let engine = ContextEngine::new(config).await;
        assert!(engine.is_ok(), "Failed to initialize context engine");
    }

    #[tokio::test]
    async fn test_get_basic_context() {
        let config = create_test_config().await;
        let engine = ContextEngine::new(config).await.unwrap();

        let context = engine.get_context().await;
        assert!(context.is_ok(), "Failed to get context");

        let context = context.unwrap();

        // Check OS name
        assert!(!context.os_name.is_empty(), "OS name should not be empty");
        assert_eq!(context.os_name, std::env::consts::OS);

        // Check username
        assert!(!context.username.is_empty(), "Username should not be empty");

        // Check pwd
        assert!(
            context.pwd.exists() || context.pwd == PathBuf::from("/"),
            "PWD should be valid path"
        );
    }

    #[tokio::test]
    async fn test_context_shell_detection() {
        let config = create_test_config().await;
        let engine = ContextEngine::new(config).await.unwrap();

        let context = engine.get_context().await.unwrap();

        // Shell name should be set (from SHELL env var or default to bash)
        assert!(
            !context.shell_name.is_empty(),
            "Shell name should not be empty"
        );
    }

    #[test]
    fn test_get_os_version() {
        let version = ContextEngine::get_os_version();
        assert!(!version.is_empty(), "OS version should not be empty");

        #[cfg(target_os = "linux")]
        {
            // On Linux, should contain "Linux" or distro name
            assert!(
                version.contains("Linux") || version.len() > 3,
                "Linux OS version should be meaningful: {}",
                version
            );
        }

        #[cfg(target_os = "macos")]
        {
            assert_eq!(version, "macOS", "macOS version should be 'macOS'");
        }
    }

    #[test]
    fn test_detect_git_context_no_repo() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let git_context = ContextEngine::detect_git_context(&path);
        assert!(
            git_context.is_none(),
            "Should detect no git repo in temp dir"
        );
    }

    #[test]
    fn test_detect_git_context_with_repo() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize a git repo
        let repo = git2::Repository::init(repo_path).unwrap();

        // Create initial commit
        let sig = git2::Signature::now("Test User", "test@example.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();

        let git_context = ContextEngine::detect_git_context(&repo_path.to_path_buf());
        assert!(git_context.is_some(), "Should detect git repo");

        let git_context = git_context.unwrap();
        assert_eq!(
            git_context.current_branch, "master",
            "Should be on master branch"
        );
        assert!(
            !git_context.has_uncommitted_changes,
            "Should have no uncommitted changes"
        );
    }

    #[test]
    fn test_detect_uncommitted_changes() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize repo
        let repo = git2::Repository::init(repo_path).unwrap();
        let sig = git2::Signature::now("Test", "test@example.com").unwrap();

        // Initial commit
        let tree_id = {
            let mut index = repo.index().unwrap();
            index.write_tree().unwrap()
        };
        let tree = repo.find_tree(tree_id).unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial", &tree, &[])
            .unwrap();

        // Create an uncommitted file
        std::fs::write(repo_path.join("test.txt"), "test content").unwrap();

        let git_context = ContextEngine::detect_git_context(&repo_path.to_path_buf());
        assert!(git_context.is_some(), "Should detect git repo");

        let git_context = git_context.unwrap();
        assert!(
            git_context.has_uncommitted_changes,
            "Should detect uncommitted changes"
        );
    }

    #[tokio::test]
    async fn test_context_serialization() {
        let context = Context {
            os_name: "Linux".to_string(),
            os_version: "Ubuntu 22.04".to_string(),
            shell_name: "bash".to_string(),
            shell_version: "5.1.0".to_string(),
            pwd: PathBuf::from("/home/test"),
            username: "testuser".to_string(),
            git_context: Some(GitContext {
                repo_name: "test-repo".to_string(),
                current_branch: "main".to_string(),
                has_uncommitted_changes: false,
                remote_url: Some("https://github.com/test/repo.git".to_string()),
                ahead_behind: Some((1, 0)),
                total_commits: Some(42),
                last_commit_message: Some("Test commit".to_string()),
            }),
            detected_languages: vec!["Rust".to_string(), "Python".to_string()],
            recent_commands: vec!["ls".to_string(), "cd".to_string()],
            project_type: Some(ProjectType::Rust),
            directory_type: DirectoryType::Project,
        };

        // Test serialization
        let json = serde_json::to_string(&context);
        assert!(json.is_ok(), "Should serialize to JSON");

        // Test deserialization
        let json_str = json.unwrap();
        let deserialized: Result<Context, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok(), "Should deserialize from JSON");

        let deserialized = deserialized.unwrap();
        assert_eq!(deserialized.username, "testuser");
        assert_eq!(deserialized.os_name, "Linux");
        assert!(deserialized.git_context.is_some());
        assert_eq!(deserialized.project_type, Some(ProjectType::Rust));
        assert_eq!(deserialized.directory_type, DirectoryType::Project);
    }

    #[test]
    fn test_detect_project_type_rust() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create Cargo.toml
        std::fs::write(path.join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();

        let project_type = ContextEngine::detect_project_type(&path);
        assert_eq!(project_type, Some(ProjectType::Rust));
    }

    #[test]
    fn test_detect_project_type_node() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create package.json
        std::fs::write(path.join("package.json"), "{}").unwrap();

        let project_type = ContextEngine::detect_project_type(&path);
        assert_eq!(project_type, Some(ProjectType::Node));
    }

    #[test]
    fn test_detect_project_type_python() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create requirements.txt
        std::fs::write(path.join("requirements.txt"), "django==4.0").unwrap();

        let project_type = ContextEngine::detect_project_type(&path);
        assert_eq!(project_type, Some(ProjectType::Python));
    }

    #[test]
    fn test_detect_project_type_go() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create go.mod
        std::fs::write(path.join("go.mod"), "module test").unwrap();

        let project_type = ContextEngine::detect_project_type(&path);
        assert_eq!(project_type, Some(ProjectType::Go));
    }

    #[test]
    fn test_detect_directory_type_root() {
        let path = PathBuf::from("/");
        let dir_type = ContextEngine::detect_directory_type(&path, "test");
        assert_eq!(dir_type, DirectoryType::Root);
    }

    #[test]
    fn test_detect_directory_type_temp() {
        let path = PathBuf::from("/tmp/test");
        let dir_type = ContextEngine::detect_directory_type(&path, "test");
        assert_eq!(dir_type, DirectoryType::Temp);
    }

    #[test]
    fn test_detect_directory_type_system() {
        let path = PathBuf::from("/etc/config");
        let dir_type = ContextEngine::detect_directory_type(&path, "test");
        assert_eq!(dir_type, DirectoryType::System);
    }

    #[test]
    fn test_detect_languages() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        // Create various language files
        std::fs::write(path.join("main.rs"), "fn main() {}").unwrap();
        std::fs::write(path.join("index.js"), "console.log('hello')").unwrap();
        std::fs::write(path.join("app.py"), "print('hello')").unwrap();

        let languages = ContextEngine::detect_languages(&path);
        assert!(languages.contains(&"Rust".to_string()));
        assert!(languages.contains(&"JavaScript/TypeScript".to_string()));
        assert!(languages.contains(&"Python".to_string()));
    }

    #[test]
    fn test_detect_languages_empty_dir() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_path_buf();

        let languages = ContextEngine::detect_languages(&path);
        assert!(languages.is_empty());
    }
}
