use anyhow::Result;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone)]
pub struct GitRepo {
    pub path: PathBuf,
    pub name: String,
    pub current_branch: String,
    pub has_uncommitted_changes: bool,
    pub ahead_commits: usize,
    pub behind_commits: usize,
    pub branch_age_days: i64,
}

impl GitRepo {
    pub fn discover(path: &PathBuf) -> Result<Option<Self>> {
        // Try to open git repository
        let repo = match git2::Repository::discover(path) {
            Ok(r) => r,
            Err(_) => return Ok(None),
        };

        let head = match repo.head() {
            Ok(h) => h,
            Err(e) => {
                debug!("Failed to get HEAD: {}", e);
                return Ok(None);
            }
        };

        let current_branch = head.shorthand().unwrap_or("unknown").to_string();

        let name = repo
            .workdir()
            .and_then(|w| w.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Check for uncommitted changes
        let has_uncommitted_changes = repo
            .statuses(None)
            .map(|statuses| !statuses.is_empty())
            .unwrap_or(false);

        // Calculate branch age
        let branch_age_days = Self::get_branch_age(&repo, &head).unwrap_or(0);

        // Get ahead/behind counts
        let (ahead_commits, behind_commits) =
            Self::get_ahead_behind(&repo, &head).unwrap_or((0, 0));

        Ok(Some(GitRepo {
            path: repo.workdir().unwrap_or(path).to_path_buf(),
            name,
            current_branch,
            has_uncommitted_changes,
            ahead_commits,
            behind_commits,
            branch_age_days,
        }))
    }

    fn get_branch_age(_repo: &git2::Repository, head: &git2::Reference) -> Result<i64> {
        let commit = head.peel_to_commit()?;
        let commit_time = commit.time();
        let commit_timestamp = commit_time.seconds();

        let now = chrono::Utc::now().timestamp();
        let age_seconds = now - commit_timestamp;
        let age_days = age_seconds / 86400; // seconds in a day

        Ok(age_days)
    }

    fn get_ahead_behind(repo: &git2::Repository, head: &git2::Reference) -> Result<(usize, usize)> {
        let local_oid = head.target().ok_or_else(|| anyhow::anyhow!("No target"))?;

        // Get upstream branch
        let branch_name = head
            .shorthand()
            .ok_or_else(|| anyhow::anyhow!("No shorthand"))?;
        let branch = repo.find_branch(branch_name, git2::BranchType::Local)?;

        let upstream = match branch.upstream() {
            Ok(u) => u,
            Err(_) => {
                // No upstream configured
                return Ok((0, 0));
            }
        };

        let upstream_oid = upstream
            .get()
            .target()
            .ok_or_else(|| anyhow::anyhow!("No upstream target"))?;

        // Calculate ahead/behind
        let (ahead, behind) = repo.graph_ahead_behind(local_oid, upstream_oid)?;

        Ok((ahead, behind))
    }

    pub fn needs_pull(&self) -> bool {
        self.behind_commits > 0
    }

    pub fn needs_push(&self) -> bool {
        self.ahead_commits > 0
    }

    pub fn is_stale(&self) -> bool {
        self.branch_age_days > 7
    }
}

#[derive(Debug, Clone)]
pub enum GitSuggestion {
    UncommittedChanges {
        repo_name: String,
        #[allow(dead_code)]
        path: PathBuf,
    },
    BehindRemote {
        repo_name: String,
        branch: String,
        behind_commits: usize,
        path: PathBuf,
    },
    AheadOfRemote {
        repo_name: String,
        branch: String,
        ahead_commits: usize,
        path: PathBuf,
    },
    StaleBranch {
        repo_name: String,
        branch: String,
        age_days: i64,
        #[allow(dead_code)]
        path: PathBuf,
    },
}

impl GitSuggestion {
    pub fn message(&self) -> String {
        match self {
            GitSuggestion::UncommittedChanges { repo_name, .. } => {
                format!("📝 {} has uncommitted changes", repo_name)
            }
            GitSuggestion::BehindRemote {
                repo_name,
                branch,
                behind_commits,
                ..
            } => {
                format!(
                    "⬇️  {} ({}) is {} commit{} behind origin",
                    repo_name,
                    branch,
                    behind_commits,
                    if *behind_commits == 1 { "" } else { "s" }
                )
            }
            GitSuggestion::AheadOfRemote {
                repo_name,
                branch,
                ahead_commits,
                ..
            } => {
                format!(
                    "⬆️  {} ({}) is {} commit{} ahead of origin",
                    repo_name,
                    branch,
                    ahead_commits,
                    if *ahead_commits == 1 { "" } else { "s" }
                )
            }
            GitSuggestion::StaleBranch {
                repo_name,
                branch,
                age_days,
                ..
            } => {
                format!(
                    "🕐 {} branch '{}' is {} days old",
                    repo_name, branch, age_days
                )
            }
        }
    }

    pub fn command(&self) -> Option<String> {
        match self {
            GitSuggestion::UncommittedChanges { path, .. } => {
                Some(format!("cd {} && git status", path.display()))
            }
            GitSuggestion::BehindRemote { path, .. } => {
                Some(format!("cd {} && git pull", path.display()))
            }
            GitSuggestion::AheadOfRemote { path, .. } => {
                Some(format!("cd {} && git push", path.display()))
            }
            GitSuggestion::StaleBranch { .. } => None,
        }
    }

    #[allow(dead_code)]
    pub fn priority(&self) -> Priority {
        match self {
            GitSuggestion::UncommittedChanges { .. } => Priority::Low,
            GitSuggestion::BehindRemote { .. } => Priority::Medium,
            GitSuggestion::AheadOfRemote { .. } => Priority::Low,
            GitSuggestion::StaleBranch { .. } => Priority::Low,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low,
    Medium,
    High,
}

pub async fn find_git_repos(search_paths: Vec<PathBuf>) -> Vec<GitRepo> {
    let mut repos = Vec::new();

    for path in search_paths {
        if let Ok(Some(repo)) = GitRepo::discover(&path) {
            repos.push(repo);
        }

        // Also search subdirectories
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        if let Ok(Some(repo)) = GitRepo::discover(&entry_path) {
                            repos.push(repo);
                        }
                    }
                }
            }
        }
    }

    repos
}

pub fn analyze_repo(repo: &GitRepo) -> Vec<GitSuggestion> {
    let mut suggestions = Vec::new();

    // Check for uncommitted changes
    if repo.has_uncommitted_changes {
        suggestions.push(GitSuggestion::UncommittedChanges {
            repo_name: repo.name.clone(),
            path: repo.path.clone(),
        });
    }

    // Check if behind remote
    if repo.needs_pull() {
        suggestions.push(GitSuggestion::BehindRemote {
            repo_name: repo.name.clone(),
            branch: repo.current_branch.clone(),
            behind_commits: repo.behind_commits,
            path: repo.path.clone(),
        });
    }

    // Check if ahead of remote
    if repo.needs_push() {
        suggestions.push(GitSuggestion::AheadOfRemote {
            repo_name: repo.name.clone(),
            branch: repo.current_branch.clone(),
            ahead_commits: repo.ahead_commits,
            path: repo.path.clone(),
        });
    }

    // Check if branch is stale
    if repo.is_stale() {
        suggestions.push(GitSuggestion::StaleBranch {
            repo_name: repo.name.clone(),
            branch: repo.current_branch.clone(),
            age_days: repo.branch_age_days,
            path: repo.path.clone(),
        });
    }

    suggestions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // ========== GitSuggestion Message Tests ==========

    #[test]
    fn test_git_suggestion_uncommitted_changes_message() {
        let suggestion = GitSuggestion::UncommittedChanges {
            repo_name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
        };

        let message = suggestion.message();
        assert!(
            message.contains("test-repo"),
            "Message should contain repo name"
        );
        assert!(
            message.contains("uncommitted changes"),
            "Message should mention uncommitted changes"
        );
    }

    #[test]
    fn test_git_suggestion_behind_remote_message() {
        let suggestion = GitSuggestion::BehindRemote {
            repo_name: "my-project".to_string(),
            branch: "main".to_string(),
            behind_commits: 3,
            path: PathBuf::from("/tmp/my-project"),
        };

        let message = suggestion.message();
        assert!(message.contains("my-project"));
        assert!(message.contains("main"));
        assert!(message.contains("3"));
        assert!(message.contains("commits"));
        assert!(message.contains("behind"));
    }

    #[test]
    fn test_git_suggestion_behind_remote_singular() {
        let suggestion = GitSuggestion::BehindRemote {
            repo_name: "my-project".to_string(),
            branch: "main".to_string(),
            behind_commits: 1,
            path: PathBuf::from("/tmp/my-project"),
        };

        let message = suggestion.message();
        assert!(
            message.contains("1 commit behind"),
            "Should use singular 'commit' for 1"
        );
    }

    #[test]
    fn test_git_suggestion_ahead_remote_message() {
        let suggestion = GitSuggestion::AheadOfRemote {
            repo_name: "my-project".to_string(),
            branch: "feature".to_string(),
            ahead_commits: 5,
            path: PathBuf::from("/tmp/my-project"),
        };

        let message = suggestion.message();
        assert!(message.contains("my-project"));
        assert!(message.contains("feature"));
        assert!(message.contains("5"));
        assert!(message.contains("commits"));
        assert!(message.contains("ahead"));
    }

    #[test]
    fn test_git_suggestion_stale_branch_message() {
        let suggestion = GitSuggestion::StaleBranch {
            repo_name: "old-project".to_string(),
            branch: "develop".to_string(),
            age_days: 14,
            path: PathBuf::from("/tmp/old-project"),
        };

        let message = suggestion.message();
        assert!(message.contains("old-project"));
        assert!(message.contains("develop"));
        assert!(message.contains("14"));
        assert!(message.contains("days old"));
    }

    // ========== GitSuggestion Command Tests ==========

    #[test]
    fn test_git_suggestion_uncommitted_changes_command() {
        let suggestion = GitSuggestion::UncommittedChanges {
            repo_name: "test-repo".to_string(),
            path: PathBuf::from("/tmp/test-repo"),
        };

        let command = suggestion.command();
        assert!(command.is_some());
        let cmd = command.unwrap();
        assert!(cmd.contains("cd /tmp/test-repo"));
        assert!(cmd.contains("git status"));
    }

    #[test]
    fn test_git_suggestion_behind_remote_command() {
        let suggestion = GitSuggestion::BehindRemote {
            repo_name: "my-project".to_string(),
            branch: "main".to_string(),
            behind_commits: 3,
            path: PathBuf::from("/tmp/my-project"),
        };

        let command = suggestion.command();
        assert!(command.is_some());
        let cmd = command.unwrap();
        assert!(cmd.contains("cd /tmp/my-project"));
        assert!(cmd.contains("git pull"));
    }

    #[test]
    fn test_git_suggestion_ahead_remote_command() {
        let suggestion = GitSuggestion::AheadOfRemote {
            repo_name: "my-project".to_string(),
            branch: "feature".to_string(),
            ahead_commits: 5,
            path: PathBuf::from("/tmp/my-project"),
        };

        let command = suggestion.command();
        assert!(command.is_some());
        let cmd = command.unwrap();
        assert!(cmd.contains("cd /tmp/my-project"));
        assert!(cmd.contains("git push"));
    }

    #[test]
    fn test_git_suggestion_stale_branch_no_command() {
        let suggestion = GitSuggestion::StaleBranch {
            repo_name: "old-project".to_string(),
            branch: "develop".to_string(),
            age_days: 14,
            path: PathBuf::from("/tmp/old-project"),
        };

        let command = suggestion.command();
        assert!(
            command.is_none(),
            "Stale branch suggestion should not have a command"
        );
    }

    // ========== GitSuggestion Priority Tests ==========

    #[test]
    fn test_git_suggestion_priority() {
        let uncommitted = GitSuggestion::UncommittedChanges {
            repo_name: "test".to_string(),
            path: PathBuf::from("/tmp/test"),
        };
        assert_eq!(uncommitted.priority(), Priority::Low);

        let behind = GitSuggestion::BehindRemote {
            repo_name: "test".to_string(),
            branch: "main".to_string(),
            behind_commits: 5,
            path: PathBuf::from("/tmp/test"),
        };
        assert_eq!(behind.priority(), Priority::Medium);

        let ahead = GitSuggestion::AheadOfRemote {
            repo_name: "test".to_string(),
            branch: "main".to_string(),
            ahead_commits: 3,
            path: PathBuf::from("/tmp/test"),
        };
        assert_eq!(ahead.priority(), Priority::Low);

        let stale = GitSuggestion::StaleBranch {
            repo_name: "test".to_string(),
            branch: "main".to_string(),
            age_days: 10,
            path: PathBuf::from("/tmp/test"),
        };
        assert_eq!(stale.priority(), Priority::Low);
    }

    // ========== GitRepo Helper Methods Tests ==========

    #[test]
    fn test_gitrepo_needs_pull() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 0,
            behind_commits: 3,
            branch_age_days: 0,
        };

        assert!(repo.needs_pull(), "Repo with behind_commits > 0 needs pull");

        let repo_no_pull = GitRepo {
            behind_commits: 0,
            ..repo
        };
        assert!(
            !repo_no_pull.needs_pull(),
            "Repo with behind_commits = 0 doesn't need pull"
        );
    }

    #[test]
    fn test_gitrepo_needs_push() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 5,
            behind_commits: 0,
            branch_age_days: 0,
        };

        assert!(repo.needs_push(), "Repo with ahead_commits > 0 needs push");

        let repo_no_push = GitRepo {
            ahead_commits: 0,
            ..repo
        };
        assert!(
            !repo_no_push.needs_push(),
            "Repo with ahead_commits = 0 doesn't need push"
        );
    }

    #[test]
    fn test_gitrepo_is_stale() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 0,
            behind_commits: 0,
            branch_age_days: 10,
        };

        assert!(repo.is_stale(), "Repo with branch_age_days > 7 is stale");

        let repo_not_stale = GitRepo {
            branch_age_days: 5,
            ..repo
        };
        assert!(
            !repo_not_stale.is_stale(),
            "Repo with branch_age_days <= 7 is not stale"
        );
    }

    // ========== analyze_repo Tests ==========

    #[test]
    fn test_analyze_repo_no_issues() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 0,
            behind_commits: 0,
            branch_age_days: 3,
        };

        let suggestions = analyze_repo(&repo);
        assert!(
            suggestions.is_empty(),
            "Repo with no issues should have no suggestions"
        );
    }

    #[test]
    fn test_analyze_repo_uncommitted_changes() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: true,
            ahead_commits: 0,
            behind_commits: 0,
            branch_age_days: 3,
        };

        let suggestions = analyze_repo(&repo);
        assert_eq!(suggestions.len(), 1);
        assert!(matches!(
            suggestions[0],
            GitSuggestion::UncommittedChanges { .. }
        ));
    }

    #[test]
    fn test_analyze_repo_behind_remote() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 0,
            behind_commits: 5,
            branch_age_days: 3,
        };

        let suggestions = analyze_repo(&repo);
        assert_eq!(suggestions.len(), 1);
        assert!(matches!(suggestions[0], GitSuggestion::BehindRemote { .. }));
    }

    #[test]
    fn test_analyze_repo_ahead_of_remote() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 3,
            behind_commits: 0,
            branch_age_days: 3,
        };

        let suggestions = analyze_repo(&repo);
        assert_eq!(suggestions.len(), 1);
        assert!(matches!(
            suggestions[0],
            GitSuggestion::AheadOfRemote { .. }
        ));
    }

    #[test]
    fn test_analyze_repo_stale_branch() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: false,
            ahead_commits: 0,
            behind_commits: 0,
            branch_age_days: 14,
        };

        let suggestions = analyze_repo(&repo);
        assert_eq!(suggestions.len(), 1);
        assert!(matches!(suggestions[0], GitSuggestion::StaleBranch { .. }));
    }

    #[test]
    fn test_analyze_repo_multiple_issues() {
        let repo = GitRepo {
            path: PathBuf::from("/tmp/test"),
            name: "test".to_string(),
            current_branch: "main".to_string(),
            has_uncommitted_changes: true,
            ahead_commits: 2,
            behind_commits: 3,
            branch_age_days: 10,
        };

        let suggestions = analyze_repo(&repo);
        assert_eq!(
            suggestions.len(),
            4,
            "Repo with multiple issues should have multiple suggestions"
        );

        // Check that all expected suggestion types are present
        let has_uncommitted = suggestions
            .iter()
            .any(|s| matches!(s, GitSuggestion::UncommittedChanges { .. }));
        let has_behind = suggestions
            .iter()
            .any(|s| matches!(s, GitSuggestion::BehindRemote { .. }));
        let has_ahead = suggestions
            .iter()
            .any(|s| matches!(s, GitSuggestion::AheadOfRemote { .. }));
        let has_stale = suggestions
            .iter()
            .any(|s| matches!(s, GitSuggestion::StaleBranch { .. }));

        assert!(
            has_uncommitted,
            "Should have uncommitted changes suggestion"
        );
        assert!(has_behind, "Should have behind remote suggestion");
        assert!(has_ahead, "Should have ahead of remote suggestion");
        assert!(has_stale, "Should have stale branch suggestion");
    }

    // ========== find_git_repos Tests ==========

    #[tokio::test]
    async fn test_find_git_repos_empty_paths() {
        let repos = find_git_repos(vec![]).await;
        assert!(
            repos.is_empty(),
            "Empty search paths should return no repos"
        );
    }

    #[tokio::test]
    async fn test_find_git_repos_nonexistent_paths() {
        let repos = find_git_repos(vec![
            PathBuf::from("/nonexistent/path/xyz"),
            PathBuf::from("/another/fake/path/abc"),
        ])
        .await;

        assert!(repos.is_empty(), "Nonexistent paths should return no repos");
    }

    #[tokio::test]
    async fn test_find_git_repos_non_git_directory() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let non_git_path = temp_dir.path().to_path_buf();

        // Create a non-git directory
        fs::create_dir_all(&non_git_path).unwrap();

        let repos = find_git_repos(vec![non_git_path]).await;

        assert!(repos.is_empty(), "Non-git directory should return no repos");
    }
}
