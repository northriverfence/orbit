use serde::{Deserialize, Serialize};
use std::fmt;

/// Current protocol version
/// Format: MAJOR.MINOR.PATCH
/// - MAJOR: Breaking changes (incompatible)
/// - MINOR: New features (backward compatible)
/// - PATCH: Bug fixes (fully compatible)
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Protocol version structure for semantic versioning
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ProtocolVersion {
    /// Parse version string (e.g., "1.2.3")
    pub fn parse(version: &str) -> Result<Self, String> {
        let parts: Vec<&str> = version.split('.').collect();

        if parts.len() != 3 {
            return Err(format!("Invalid version format: {}", version));
        }

        let major = parts[0].parse()
            .map_err(|_| format!("Invalid major version: {}", parts[0]))?;
        let minor = parts[1].parse()
            .map_err(|_| format!("Invalid minor version: {}", parts[1]))?;
        let patch = parts[2].parse()
            .map_err(|_| format!("Invalid patch version: {}", parts[2]))?;

        Ok(Self { major, minor, patch })
    }

    /// Check if two versions are compatible (same major version)
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }

    /// Check if this version supports a feature added in required version
    pub fn supports_feature(&self, required: &Self) -> bool {
        self.major > required.major ||
        (self.major == required.major && self.minor >= required.minor)
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Versioned request wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionedRequest {
    pub version: String,
    #[serde(flatten)]
    pub request: Request,
}

/// Versioned response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct VersionedResponse {
    pub version: String,
    #[serde(flatten)]
    pub response: Response,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Request {
    Command {
        input: String,
        cwd: String,
        shell: String,
    },
    Feedback {
        input: String,
        executed: String,
        result: FeedbackResult,
    },
    Status,
    Shutdown,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Passthrough,
    Replaced {
        command: String,
    },
    Error {
        message: String,
    },
    Status {
        uptime_secs: u64,
        commands_processed: u64,
    },
    Ok,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeedbackResult {
    Success,
    Failed,
    Rejected,
    Edited { new_command: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = ProtocolVersion::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_parse_invalid() {
        assert!(ProtocolVersion::parse("1.2").is_err());
        assert!(ProtocolVersion::parse("1.2.3.4").is_err());
        assert!(ProtocolVersion::parse("a.b.c").is_err());
    }

    #[test]
    fn test_version_compatibility() {
        let v1_0 = ProtocolVersion::parse("1.0.0").unwrap();
        let v1_1 = ProtocolVersion::parse("1.1.0").unwrap();
        let v1_2 = ProtocolVersion::parse("1.2.0").unwrap();
        let v2_0 = ProtocolVersion::parse("2.0.0").unwrap();

        // Same major version = compatible
        assert!(v1_0.is_compatible(&v1_1));
        assert!(v1_1.is_compatible(&v1_0));
        assert!(v1_0.is_compatible(&v1_2));

        // Different major version = incompatible
        assert!(!v1_0.is_compatible(&v2_0));
        assert!(!v2_0.is_compatible(&v1_0));
    }

    #[test]
    fn test_feature_support() {
        let v1_0 = ProtocolVersion::parse("1.0.0").unwrap();
        let v1_1 = ProtocolVersion::parse("1.1.0").unwrap();
        let v1_2 = ProtocolVersion::parse("1.2.0").unwrap();
        let v2_0 = ProtocolVersion::parse("2.0.0").unwrap();

        // Higher version supports features from lower version
        assert!(v1_1.supports_feature(&v1_0));
        assert!(v1_2.supports_feature(&v1_0));
        assert!(v1_2.supports_feature(&v1_1));
        assert!(v2_0.supports_feature(&v1_0));

        // Lower version doesn't support features from higher version
        assert!(!v1_0.supports_feature(&v1_1));
        assert!(!v1_1.supports_feature(&v1_2));
    }

    #[test]
    fn test_version_display() {
        let v = ProtocolVersion::parse("1.2.3").unwrap();
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn test_versioned_request_serialize() {
        let request = Request::Status;
        let versioned = VersionedRequest {
            version: PROTOCOL_VERSION.to_string(),
            request,
        };

        let json = serde_json::to_string(&versioned).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("Status"));
    }

    #[test]
    fn test_versioned_response_serialize() {
        let response = Response::Ok;
        let versioned = VersionedResponse {
            version: PROTOCOL_VERSION.to_string(),
            response,
        };

        let json = serde_json::to_string(&versioned).unwrap();
        assert!(json.contains("\"version\":\"1.0.0\""));
        assert!(json.contains("Ok"));
    }
}
