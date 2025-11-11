// Library exports for testing and CLI tool
pub mod autostart;
pub mod classifier;
pub mod config;
pub mod context;
pub mod credentials;
pub mod daemon;
pub mod embeddings;
pub mod executor;
pub mod learning;
pub mod license;
pub mod monitor;
pub mod prompts;
pub mod providers;
pub mod service;
pub mod session;

// Re-export commonly used types for CLI
pub use daemon::ipc::{
    FeedbackResult, ProtocolVersion, Request, Response,
    VersionedRequest, VersionedResponse, PROTOCOL_VERSION,
};
