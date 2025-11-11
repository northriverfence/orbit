// Command sandboxing and validation for Orbit AI Terminal
//
// This module provides security controls for command execution including:
// - Dangerous pattern detection
// - Destructive command identification
// - User confirmation requirements
// - Configurable allow/block lists

use regex::Regex;
use std::collections::HashSet;
use std::sync::OnceLock;

/// Command validator for detecting dangerous operations
pub struct CommandValidator {
    dangerous_patterns: Vec<Regex>,
    destructive_commands: HashSet<String>,
    requires_sudo: HashSet<String>,
    network_commands: HashSet<String>,
}

impl CommandValidator {
    /// Create a new command validator with default rules
    pub fn new() -> Self {
        let dangerous_patterns = vec![
            // Dangerous rm patterns
            Regex::new(r"rm\s+-rf\s+/\s*$").unwrap(),
            Regex::new(r"rm\s+-rf\s+/\w+").unwrap(),
            Regex::new(r"rm\s+.*\s+-rf\s+/").unwrap(),

            // Fork bomb
            Regex::new(r":()\s*\{\s*:\|:&\s*\};\s*:").unwrap(),

            // Disk formatting
            Regex::new(r"mkfs\.").unwrap(),
            Regex::new(r"dd\s+if=.*\s+of=/dev/sd").unwrap(),
            Regex::new(r"dd\s+if=.*\s+of=/dev/nvme").unwrap(),

            // System modification
            Regex::new(r">\s*/dev/(sd|nvme)").unwrap(),
            Regex::new(r"chmod\s+-R\s+777\s+/").unwrap(),
            Regex::new(r"chown\s+-R\s+.*\s+/").unwrap(),

            // Dangerous shell redirects
            Regex::new(r">\s*/etc/(passwd|shadow|sudoers)").unwrap(),

            // Command injection patterns
            Regex::new(r";\s*rm\s+-rf").unwrap(),
            Regex::new(r"\|\s*rm\s+-rf").unwrap(),
            Regex::new(r"&&\s*rm\s+-rf").unwrap(),

            // Kernel manipulation
            Regex::new(r"/proc/sys/kernel").unwrap(),

            // Critical system files
            Regex::new(r"rm\s+.*/(vmlinuz|initrd|grub)").unwrap(),
        ];

        let destructive_commands = HashSet::from([
            "rm".to_string(),
            "dd".to_string(),
            "mkfs".to_string(),
            "fdisk".to_string(),
            "parted".to_string(),
            "wipefs".to_string(),
            "shred".to_string(),
            "cryptsetup".to_string(),
        ]);

        let requires_sudo = HashSet::from([
            "sudo".to_string(),
            "su".to_string(),
            "doas".to_string(),
            "systemctl".to_string(),
            "service".to_string(),
            "reboot".to_string(),
            "shutdown".to_string(),
            "poweroff".to_string(),
            "halt".to_string(),
        ]);

        let network_commands = HashSet::from([
            "curl".to_string(),
            "wget".to_string(),
            "nc".to_string(),
            "netcat".to_string(),
            "nmap".to_string(),
            "telnet".to_string(),
            "ssh".to_string(),
            "scp".to_string(),
            "rsync".to_string(),
            "ftp".to_string(),
        ]);

        Self {
            dangerous_patterns,
            destructive_commands,
            requires_sudo,
            network_commands,
        }
    }

    /// Validate a command and return the validation result
    ///
    /// # Arguments
    /// * `command` - The command to validate
    ///
    /// # Returns
    /// ValidationResult indicating safety level and required action
    pub fn validate(&self, command: &str) -> ValidationResult {
        let trimmed = command.trim();

        // Check for empty command
        if trimmed.is_empty() {
            return ValidationResult::Invalid {
                reason: "Empty command".to_string(),
            };
        }

        // Check for dangerous patterns first (highest priority)
        for (idx, pattern) in self.dangerous_patterns.iter().enumerate() {
            if pattern.is_match(trimmed) {
                let reason = match idx {
                    0..=2 => "Recursive delete of root or system directory detected",
                    3 => "Fork bomb detected",
                    4..=6 => "Disk formatting or raw disk write detected",
                    7..=9 => "Direct system file modification detected",
                    10 => "Critical system file modification detected",
                    11..=13 => "Command injection with rm detected",
                    14 => "Kernel parameter modification detected",
                    15 => "Critical boot file deletion detected",
                    _ => "Dangerous pattern detected",
                };

                return ValidationResult::Dangerous {
                    reason: reason.to_string(),
                    severity: Severity::Critical,
                    pattern: pattern.as_str().to_string(),
                };
            }
        }

        let first_word = trimmed.split_whitespace().next().unwrap_or("");

        // Check for destructive commands
        if self.destructive_commands.contains(first_word) {
            let severity = if trimmed.contains("-rf") || trimmed.contains("-r") {
                Severity::High
            } else {
                Severity::Medium
            };

            return ValidationResult::RequiresConfirmation {
                reason: format!("'{}' is a destructive command", first_word),
                severity,
                command_type: CommandType::Destructive,
            };
        }

        // Check for sudo commands
        if self.requires_sudo.contains(first_word) {
            return ValidationResult::RequiresConfirmation {
                reason: format!("'{}' requires elevated privileges", first_word),
                severity: Severity::High,
                command_type: CommandType::Privileged,
            };
        }

        // Check for network commands
        if self.network_commands.contains(first_word) {
            return ValidationResult::RequiresConfirmation {
                reason: format!("'{}' will make network requests", first_word),
                severity: Severity::Low,
                command_type: CommandType::Network,
            };
        }

        // Additional checks for shell metacharacters
        if self.has_dangerous_shell_features(trimmed) {
            return ValidationResult::RequiresConfirmation {
                reason: "Command contains shell metacharacters or pipes".to_string(),
                severity: Severity::Medium,
                command_type: CommandType::Complex,
            };
        }

        ValidationResult::Safe
    }

    /// Check for potentially dangerous shell features
    fn has_dangerous_shell_features(&self, command: &str) -> bool {
        // Check for dangerous shell features but allow common patterns
        let has_multiple_commands = command.matches(';').count() > 0;
        let has_pipes = command.matches('|').count() > 0;
        let has_redirects = command.matches('>').filter(|&c| c != ">>").count() > 0;
        let has_backticks = command.contains('`');
        let has_subshell = command.contains("$(");

        // Only flag if multiple dangerous features are present
        let danger_count = [
            has_multiple_commands,
            has_pipes,
            has_redirects,
            has_backticks,
            has_subshell,
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        danger_count >= 2
    }

    /// Check if a command is on the blocklist
    pub fn is_blocked(&self, command: &str) -> bool {
        matches!(self.validate(command), ValidationResult::Dangerous { .. })
    }

    /// Get statistics about validation rules
    pub fn stats(&self) -> ValidatorStats {
        ValidatorStats {
            dangerous_patterns_count: self.dangerous_patterns.len(),
            destructive_commands_count: self.destructive_commands.len(),
            privileged_commands_count: self.requires_sudo.len(),
            network_commands_count: self.network_commands.len(),
        }
    }
}

impl Default for CommandValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of command validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationResult {
    /// Command is safe to execute
    Safe,

    /// Command requires user confirmation
    RequiresConfirmation {
        reason: String,
        severity: Severity,
        command_type: CommandType,
    },

    /// Command is dangerous and should be blocked
    Dangerous {
        reason: String,
        severity: Severity,
        pattern: String,
    },

    /// Command is invalid
    Invalid {
        reason: String,
    },
}

impl ValidationResult {
    /// Check if the command is safe to execute without confirmation
    pub fn is_safe(&self) -> bool {
        matches!(self, ValidationResult::Safe)
    }

    /// Check if the command requires confirmation
    pub fn requires_confirmation(&self) -> bool {
        matches!(self, ValidationResult::RequiresConfirmation { .. })
    }

    /// Check if the command is dangerous
    pub fn is_dangerous(&self) -> bool {
        matches!(self, ValidationResult::Dangerous { .. })
    }

    /// Get a user-friendly message
    pub fn message(&self) -> String {
        match self {
            ValidationResult::Safe => "Command is safe to execute".to_string(),
            ValidationResult::RequiresConfirmation { reason, severity, .. } => {
                format!("[{}] {}", severity, reason)
            }
            ValidationResult::Dangerous { reason, severity, .. } => {
                format!("[{}] {} - Command blocked for safety", severity, reason)
            }
            ValidationResult::Invalid { reason } => format!("Invalid: {}", reason),
        }
    }
}

/// Severity level for validation results
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Type of command requiring confirmation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandType {
    Destructive,
    Privileged,
    Network,
    Complex,
}

impl std::fmt::Display for CommandType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandType::Destructive => write!(f, "Destructive"),
            CommandType::Privileged => write!(f, "Privileged"),
            CommandType::Network => write!(f, "Network"),
            CommandType::Complex => write!(f, "Complex"),
        }
    }
}

/// Statistics about the validator
#[derive(Debug, Clone)]
pub struct ValidatorStats {
    pub dangerous_patterns_count: usize,
    pub destructive_commands_count: usize,
    pub privileged_commands_count: usize,
    pub network_commands_count: usize,
}

/// Get the global command validator
pub fn get_validator() -> &'static CommandValidator {
    static VALIDATOR: OnceLock<CommandValidator> = OnceLock::new();
    VALIDATOR.get_or_init(CommandValidator::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_commands() {
        let validator = CommandValidator::new();

        assert!(validator.validate("ls -la").is_safe());
        assert!(validator.validate("echo hello").is_safe());
        assert!(validator.validate("cat file.txt").is_safe());
        assert!(validator.validate("grep pattern file").is_safe());
        assert!(validator.validate("pwd").is_safe());
    }

    #[test]
    fn test_dangerous_commands() {
        let validator = CommandValidator::new();

        assert!(validator.validate("rm -rf /").is_dangerous());
        assert!(validator.validate("rm -rf /usr").is_dangerous());
        assert!(validator.validate(":(){ :|:& };:").is_dangerous());
        assert!(validator.validate("mkfs.ext4 /dev/sda1").is_dangerous());
        assert!(validator.validate("dd if=/dev/zero of=/dev/sda").is_dangerous());
        assert!(validator.validate("> /etc/passwd").is_dangerous());
    }

    #[test]
    fn test_requires_confirmation() {
        let validator = CommandValidator::new();

        let result = validator.validate("rm file.txt");
        assert!(result.requires_confirmation());

        let result = validator.validate("sudo apt install package");
        assert!(result.requires_confirmation());

        let result = validator.validate("curl https://example.com");
        assert!(result.requires_confirmation());
    }

    #[test]
    fn test_destructive_with_rf() {
        let validator = CommandValidator::new();

        let result = validator.validate("rm -rf /tmp/test");
        assert!(result.requires_confirmation());

        if let ValidationResult::RequiresConfirmation { severity, .. } = result {
            assert_eq!(severity, Severity::High);
        } else {
            panic!("Expected RequiresConfirmation");
        }
    }

    #[test]
    fn test_empty_command() {
        let validator = CommandValidator::new();

        let result = validator.validate("");
        assert!(matches!(result, ValidationResult::Invalid { .. }));

        let result = validator.validate("   ");
        assert!(matches!(result, ValidationResult::Invalid { .. }));
    }

    #[test]
    fn test_command_injection() {
        let validator = CommandValidator::new();

        assert!(validator.validate("ls; rm -rf /").is_dangerous());
        assert!(validator.validate("cat file | rm -rf /").is_dangerous());
        assert!(validator.validate("echo test && rm -rf /tmp").is_dangerous());
    }

    #[test]
    fn test_validation_result_message() {
        let result = ValidationResult::Safe;
        assert_eq!(result.message(), "Command is safe to execute");

        let result = ValidationResult::Dangerous {
            reason: "Test danger".to_string(),
            severity: Severity::Critical,
            pattern: "test".to_string(),
        };
        assert!(result.message().contains("CRITICAL"));
        assert!(result.message().contains("Test danger"));
    }

    #[test]
    fn test_validator_stats() {
        let validator = CommandValidator::new();
        let stats = validator.stats();

        assert!(stats.dangerous_patterns_count > 0);
        assert!(stats.destructive_commands_count > 0);
        assert!(stats.privileged_commands_count > 0);
        assert!(stats.network_commands_count > 0);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn test_shell_metacharacters() {
        let validator = CommandValidator::new();

        // Single pipe should be safe (common use)
        assert!(validator.validate("ls | grep test").is_safe());

        // Multiple dangerous features should require confirmation
        let result = validator.validate("ls | grep test > file; cat file");
        assert!(result.requires_confirmation());
    }
}
