// Security module for Orbit AI Terminal
//
// This module provides comprehensive security features including:
// - Command sandboxing and validation
// - Audit logging for compliance
// - Data encryption at rest
// - Rate limiting for abuse prevention

pub mod sandbox;
pub mod audit;
pub mod encryption;

pub use sandbox::{CommandValidator, ValidationResult, Severity};
pub use audit::{AuditLogger, AuditEvent, EventType};
pub use encryption::Encryptor;
