//! IPC protocol messages for daemon-client communication
//!
//! Implements JSON-RPC 2.0 style protocol over Unix sockets

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::session_manager::{SessionInfo, SessionType};

/// Request message from client to daemon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// Unique request ID (for matching responses)
    pub id: String,
    /// Method to invoke
    pub method: String,
    /// Method parameters (method-specific)
    pub params: serde_json::Value,
}

/// Response message from daemon to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// Request ID (matches request)
    pub id: String,
    /// Response result or error
    #[serde(flatten)]
    pub result: ResponseResult,
}

/// Response result (success or error)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseResult {
    Success { result: serde_json::Value },
    Error { error: ErrorInfo },
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    pub code: i32,
    pub message: String,
}

// ===== Method-specific parameter types =====

/// Parameters for create_session method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionParams {
    pub name: String,
    #[serde(rename = "type")]
    pub session_type: SessionType,
    pub cols: Option<u16>,
    pub rows: Option<u16>,
}

/// Parameters for attach_session method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachSessionParams {
    pub session_id: Uuid,
    pub client_id: Uuid,
}

/// Parameters for detach_session method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetachSessionParams {
    pub session_id: Uuid,
    pub client_id: Uuid,
}

/// Parameters for send_input method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendInputParams {
    pub session_id: Uuid,
    pub data: String,  // Base64-encoded binary data
}

/// Parameters for receive_output method (streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReceiveOutputParams {
    pub session_id: Uuid,
    pub timeout_ms: Option<u64>,  // Optional timeout for blocking read
}

/// Parameters for resize_terminal method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeTerminalParams {
    pub session_id: Uuid,
    pub cols: u16,
    pub rows: u16,
}

/// Parameters for terminate_session method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminateSessionParams {
    pub session_id: Uuid,
}

// ===== Response types =====

/// Response for create_session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionResult {
    pub session_id: Uuid,
}

/// Response for list_sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSessionsResult {
    pub sessions: Vec<SessionInfo>,
}

/// Response for get_status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResult {
    pub version: String,
    pub uptime_seconds: u64,
    pub num_sessions: usize,
    pub num_clients: usize,
}

// ===== Error codes =====

pub mod error_codes {
    pub const INVALID_REQUEST: i32 = -32600;
    pub const METHOD_NOT_FOUND: i32 = -32601;
    pub const INVALID_PARAMS: i32 = -32602;
    pub const INTERNAL_ERROR: i32 = -32603;
    pub const SESSION_NOT_FOUND: i32 = 1001;
    pub const SESSION_EXISTS: i32 = 1002;
}

// ===== Helper functions =====

impl Response {
    /// Create a success response
    pub fn success(id: String, result: impl Serialize) -> Self {
        Self {
            id,
            result: ResponseResult::Success {
                result: serde_json::to_value(result).unwrap_or(serde_json::Value::Null),
            },
        }
    }

    /// Create an error response
    pub fn error(id: String, code: i32, message: String) -> Self {
        Self {
            id,
            result: ResponseResult::Error {
                error: ErrorInfo { code, message },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let req = Request {
            id: "1".to_string(),
            method: "create_session".to_string(),
            params: serde_json::json!({
                "name": "test",
                "type": "Local"
            }),
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: Request = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.id, "1");
        assert_eq!(parsed.method, "create_session");
    }

    #[test]
    fn test_response_success_serialization() {
        let resp = Response::success("1".to_string(), CreateSessionResult {
            session_id: Uuid::new_v4(),
        });

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("result"));
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_response_error_serialization() {
        let resp = Response::error(
            "1".to_string(),
            error_codes::SESSION_NOT_FOUND,
            "Session not found".to_string(),
        );

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains(&error_codes::SESSION_NOT_FOUND.to_string()));
    }
}
