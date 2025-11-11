// Observability module
// Provides metrics, health checks, and monitoring capabilities

pub mod metrics;
pub mod health;
pub mod logging;
pub mod server;

pub use metrics::{get_metrics, Metrics};
pub use health::{HealthChecker, HealthStatus, Status};
pub use logging::init_logging;
pub use server::start_server;
