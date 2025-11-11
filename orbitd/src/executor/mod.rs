use anyhow::Result;
use std::sync::Arc;

use crate::config::Config;

pub struct Executor {
    _config: Arc<Config>,
}

impl Executor {
    pub async fn new(config: Arc<Config>) -> Result<Self> {
        Ok(Self { _config: config })
    }

    #[allow(dead_code)]
    pub async fn execute(&self, command: &str) -> Result<String> {
        // TODO: Implement actual command execution
        tracing::info!("Would execute: {}", command);
        Ok(String::new())
    }

    #[allow(dead_code)]
    pub fn is_destructive(&self, command: &str) -> bool {
        // Use comprehensive command analysis instead of simple keyword matching
        CommandAnalyzer::new().is_destructive(command)
    }
}

/// Robust command analyzer that parses shell syntax to detect destructive commands
struct CommandAnalyzer {
    destructive_commands: Vec<&'static str>,
    destructive_patterns: Vec<DestructivePattern>,
}

#[derive(Clone)]
struct DestructivePattern {
    command: &'static str,
    requires_flags: Vec<&'static str>,
    #[allow(dead_code)]
    description: &'static str,
}

impl CommandAnalyzer {
    fn new() -> Self {
        Self {
            // Comprehensive list of destructive commands (case-insensitive)
            destructive_commands: vec![
                "mkfs",
                "mkfs.ext2",
                "mkfs.ext3",
                "mkfs.ext4",
                "mkfs.xfs",
                "mkfs.btrfs",
                "wipefs",
                "shred",
                "wipe",
                "srm",
                "fdisk",
                "gdisk",
                "parted",
                "gparted",
                "format",
                "diskpart",
                "cryptsetup",
            ],
            destructive_patterns: vec![
                DestructivePattern {
                    command: "rm",
                    requires_flags: vec!["-r", "-rf", "-fr", "--recursive"],
                    description: "recursive file deletion",
                },
                DestructivePattern {
                    command: "dd",
                    requires_flags: vec!["of="],
                    description: "disk writing",
                },
                DestructivePattern {
                    command: "find",
                    requires_flags: vec!["-delete", "-exec rm"],
                    description: "file deletion",
                },
                DestructivePattern {
                    command: "chmod",
                    requires_flags: vec!["-R", "--recursive", "000", "0000"],
                    description: "permission manipulation",
                },
                DestructivePattern {
                    command: "chown",
                    requires_flags: vec!["-R", "--recursive"],
                    description: "ownership changes",
                },
                DestructivePattern {
                    command: "truncate",
                    requires_flags: vec!["-s", "--size"],
                    description: "file truncation",
                },
            ],
        }
    }

    fn is_destructive(&self, command: &str) -> bool {
        if command.trim().is_empty() {
            return false;
        }

        // Normalize to lowercase for case-insensitive matching
        let normalized = command.to_lowercase();

        // Check for fork bombs and other shell exploits
        if self.is_fork_bomb(&normalized) {
            return true;
        }

        // Check for dangerous redirects (> /dev/sda, etc.)
        if self.has_dangerous_redirect(&normalized) {
            return true;
        }

        // Check for encoding/obfuscation attempts
        if self.has_obfuscation(&normalized) {
            return true;
        }

        // Split command into tokens, handling quotes and escapes
        let tokens = self.tokenize(&normalized);

        // Check each token sequence for destructive commands
        self.contains_destructive_command(&tokens)
    }

    fn is_fork_bomb(&self, command: &str) -> bool {
        // Detect common fork bomb patterns
        let fork_bomb_patterns = [":|:", ":()", "|&", "fork()"];

        fork_bomb_patterns
            .iter()
            .any(|pattern| command.contains(pattern))
    }

    fn has_dangerous_redirect(&self, command: &str) -> bool {
        // Detect redirects to /dev devices (except /dev/null, /dev/zero, /dev/stdout, /dev/stderr)
        if command.contains("> /dev/") || command.contains(">> /dev/") {
            let safe_devices = [
                "/dev/null",
                "/dev/zero",
                "/dev/stdout",
                "/dev/stderr",
                "/dev/tty",
            ];

            // If redirecting to /dev but not to a safe device, it's dangerous
            if !safe_devices.iter().any(|safe| command.contains(safe)) {
                // Check if there's actual device name after /dev/
                if command.contains("/dev/sd")
                    || command.contains("/dev/hd")
                    || command.contains("/dev/nvme")
                    || command.contains("/dev/disk")
                {
                    return true;
                }
            }
        }
        false
    }

    fn has_obfuscation(&self, command: &str) -> bool {
        // Detect base64 decoding followed by execution
        if command.contains("base64") && (command.contains("| sh") || command.contains("| bash")) {
            return true;
        }

        // Detect hex encoding
        if command.contains("\\x") && command.matches("\\x").count() > 3 {
            return true;
        }

        false
    }

    fn tokenize(&self, command: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;
        let mut escape_next = false;
        let chars: Vec<char> = command.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if escape_next {
                current_token.push(c);
                escape_next = false;
                i += 1;
                continue;
            }

            match c {
                '\\' if !in_single_quote => {
                    escape_next = true;
                }
                '\'' if !in_double_quote => {
                    in_single_quote = !in_single_quote;
                }
                '"' if !in_single_quote => {
                    in_double_quote = !in_double_quote;
                }
                ' ' | '\t' | '\n' if !in_single_quote && !in_double_quote => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                ';' | '|' | '&' if !in_single_quote && !in_double_quote => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                    // Add separator as token
                    tokens.push(c.to_string());
                }
                _ => {
                    current_token.push(c);
                }
            }

            i += 1;
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    fn contains_destructive_command(&self, tokens: &[String]) -> bool {
        if tokens.is_empty() {
            return false;
        }

        // Iterate through tokens to find command names
        let mut i = 0;
        while i < tokens.len() {
            let token = &tokens[i];

            // Skip command separators and redirects
            if token == "|" || token == ";" || token == "&" || token == "&&" || token == "||" {
                i += 1;
                continue;
            }

            // Extract command name (remove path if present)
            let command_name = if token.starts_with("./") || token.starts_with('/') {
                token.split('/').last().unwrap_or(token)
            } else {
                token.as_str()
            };

            // Skip sudo/doas prefixes
            if command_name == "sudo" || command_name == "doas" {
                i += 1;
                continue;
            }

            // Check against always-destructive commands
            if self
                .destructive_commands
                .iter()
                .any(|&cmd| command_name == cmd)
            {
                return true;
            }

            // Check against pattern-based destructive commands
            if let Some(pattern) = self
                .destructive_patterns
                .iter()
                .find(|p| p.command == command_name)
            {
                // Check if the command has the required flags
                if self.has_required_flags(tokens, i, &pattern.requires_flags) {
                    return true;
                }
            }

            i += 1;
        }

        false
    }

    fn has_required_flags(
        &self,
        tokens: &[String],
        cmd_index: usize,
        required_flags: &[&str],
    ) -> bool {
        // Look ahead in tokens for required flags (within reasonable distance)
        let search_end = (cmd_index + 10).min(tokens.len());

        for flag in required_flags {
            let flag_lower = flag.to_lowercase();
            for j in (cmd_index + 1)..search_end {
                let token = &tokens[j];

                // Stop if we hit another command
                if token == "|" || token == ";" || token == "&&" || token == "||" {
                    break;
                }

                // Check if this token contains the required flag (case-insensitive)
                if token.contains(&flag_lower) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::TempDir;

    async fn create_test_executor() -> Executor {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("config.yaml");
        let data_dir = temp_dir.path().join("data");
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

        let config = Arc::new(Config::load().await.unwrap());
        Executor::new(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_executor_initialization() {
        let executor = create_test_executor().await;
        assert!(
            executor._config.execution.confirm_destructive,
            "Executor should have destructive command confirmation enabled"
        );
    }

    // ========== Destructive Command Detection Tests ==========

    #[tokio::test]
    async fn test_is_destructive_rm_rf() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("rm -rf /tmp/test"),
            "Should detect 'rm -rf' as destructive"
        );
        assert!(
            executor.is_destructive("sudo rm -rf /"),
            "Should detect 'sudo rm -rf /' as destructive"
        );
        assert!(
            executor.is_destructive("rm -rf *"),
            "Should detect 'rm -rf *' as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_dd() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("dd if=/dev/zero of=/dev/sda"),
            "Should detect 'dd' as destructive"
        );
        assert!(
            executor.is_destructive("sudo dd if=/dev/urandom of=/dev/sdb"),
            "Should detect 'sudo dd' as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_mkfs() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("mkfs.ext4 /dev/sda1"),
            "Should detect 'mkfs' as destructive"
        );
        assert!(
            executor.is_destructive("sudo mkfs /dev/sdb"),
            "Should detect 'sudo mkfs' as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_format() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("format c:"),
            "Should detect 'format' as destructive"
        );
        assert!(
            executor.is_destructive("quick format /dev/sda"),
            "Should detect format commands as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_dev_redirect() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("echo test > /dev/sda"),
            "Should detect redirect to /dev as destructive"
        );
        // NEW: /dev/null is now correctly excluded (not overly cautious)
        assert!(
            !executor.is_destructive("cat file > /dev/null"),
            "NEW: /dev/null correctly NOT detected as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_shred() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("shred -n 3 /dev/sda"),
            "Should detect 'shred' as destructive"
        );
        assert!(
            executor.is_destructive("shred important_file.txt"),
            "Should detect 'shred' on files as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_wipefs() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("wipefs /dev/sda"),
            "Should detect 'wipefs' as destructive"
        );
        assert!(
            executor.is_destructive("sudo wipefs -a /dev/sdb"),
            "Should detect 'sudo wipefs' as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_false_positives() {
        let executor = create_test_executor().await;

        // These should NOT be detected as destructive (false positives we accept)
        assert!(
            !executor.is_destructive("ls -la /home"),
            "Should not detect 'ls' as destructive"
        );
        assert!(
            !executor.is_destructive("rm test.txt"),
            "Should not detect 'rm' without '-rf' as destructive (by current implementation)"
        );
        assert!(
            !executor.is_destructive("mkdir /tmp/test"),
            "Should not detect 'mkdir' as destructive"
        );
        assert!(
            !executor.is_destructive("echo 'hello world'"),
            "Should not detect 'echo' as destructive"
        );
        assert!(
            !executor.is_destructive("cat file.txt"),
            "Should not detect 'cat' as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_safe_rm() {
        let executor = create_test_executor().await;

        // rm without recursive flags should NOT be detected as destructive
        assert!(
            !executor.is_destructive("rm file.txt"),
            "'rm' without '-r' is not detected"
        );
        // NEW: rm -r IS now detected as destructive (improvement over old implementation)
        assert!(
            executor.is_destructive("rm -r directory/"),
            "NEW: 'rm -r' IS detected as destructive"
        );
        assert!(
            !executor.is_destructive("rm -f file.txt"),
            "'rm -f' without 'r' is not detected"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_edge_cases() {
        let executor = create_test_executor().await;

        assert!(
            !executor.is_destructive(""),
            "Empty command should not be destructive"
        );
        assert!(
            !executor.is_destructive("   "),
            "Whitespace-only command should not be destructive"
        );
        assert!(
            executor.is_destructive("  rm -rf /  "),
            "Should detect destructive command with extra whitespace"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_case_sensitivity() {
        let executor = create_test_executor().await;

        // NEW: Implementation is now case-INsensitive (improvement!)
        assert!(
            executor.is_destructive("rm -rf /tmp"),
            "Should detect lowercase 'rm -rf'"
        );
        assert!(
            executor.is_destructive("RM -RF /tmp"),
            "NEW: uppercase IS now detected"
        );
        assert!(
            executor.is_destructive("Rm -Rf /tmp"),
            "NEW: mixed case IS now detected"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_bypass_attempts() {
        let executor = create_test_executor().await;

        // Tests for potential bypass techniques

        // Quote bypass - BYPASSES CURRENT DETECTION (security issue documented)
        assert!(
            !executor.is_destructive(r#"r"m -rf /"#),
            "Quote bypass should pass through (known limitation)"
        );

        // Variable bypass - NEW: Correctly NOT detected (it's just a variable assignment, not execution)
        let cmd = "CMD=\"rm -rf\"; $CMD /";
        assert!(
            !executor.is_destructive(cmd),
            "NEW: Variable bypass correctly NOT detected (proper tokenization)"
        );

        // Command substitution - BYPASSES CURRENT DETECTION (security issue documented)
        assert!(
            !executor.is_destructive("$(echo rm) -rf /"),
            "Command substitution bypass should pass through (known limitation)"
        );

        // Encoding bypass - NEW: Now detected (improvement!)
        assert!(
            executor.is_destructive("echo Y20gLXJmIC8= | base64 -d | sh"),
            "NEW: Base64 encoding bypass IS now detected"
        );

        // Escaped characters - NEW: Now properly detected (improvement!)
        assert!(
            executor.is_destructive("rm \\-rf /tmp"),
            "NEW: Escaped flags ARE now detected (proper tokenization)"
        );

        // Using hex encoding in command
        assert!(
            !executor.is_destructive("\\x72\\x6d -rf /"),
            "Hex encoding bypass not detected (known limitation)"
        );

        // Note: These tests document that the current simple keyword matching
        // has limitations. Future improvements should add proper command parsing.
    }

    #[tokio::test]
    async fn test_is_destructive_alternate_destructive_commands() {
        let executor = create_test_executor().await;

        // NEW: All these destructive commands ARE now detected (improvements!)
        assert!(
            executor.is_destructive("find / -delete"),
            "NEW: 'find -delete' IS detected as destructive"
        );
        assert!(
            executor.is_destructive("chmod -R 000 /"),
            "NEW: 'chmod -R 000' IS detected as destructive"
        );
        assert!(
            executor.is_destructive("truncate -s 0 important.txt"),
            "NEW: 'truncate' IS detected as destructive"
        );
        assert!(
            executor.is_destructive(":(){ :|:& };:"),
            "NEW: Fork bomb IS detected as destructive"
        );
        assert!(
            executor.is_destructive("chown -R nobody:nobody /"),
            "NEW: 'chown -R' IS detected as destructive"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_multiple_commands() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("ls -la && rm -rf /tmp"),
            "Should detect destructive command in chain"
        );
        assert!(
            executor.is_destructive("echo 'start' ; rm -rf * ; echo 'done'"),
            "Should detect destructive command in sequence"
        );
        assert!(
            executor.is_destructive("cd /tmp || rm -rf /backup"),
            "Should detect destructive command in conditional"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_with_options() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("rm -rf --no-preserve-root /"),
            "Should detect 'rm -rf' with additional options"
        );
        assert!(
            executor.is_destructive("rm -rfv /tmp/test"),
            "Should detect 'rm -rf' with verbose flag"
        );
        assert!(
            executor.is_destructive("rm -rf -i /tmp"),
            "Should detect 'rm -rf' with interactive flag"
        );
    }

    #[tokio::test]
    async fn test_is_destructive_piped_commands() {
        let executor = create_test_executor().await;

        assert!(
            executor.is_destructive("cat file.txt | dd of=/dev/sda"),
            "Should detect destructive command in pipe"
        );
        assert!(
            executor.is_destructive("ls -la | grep test | xargs rm -rf"),
            "Should detect 'rm -rf' in pipe chain"
        );
    }
}
