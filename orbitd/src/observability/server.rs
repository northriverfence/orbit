// HTTP server for metrics and health endpoints
// Serves Prometheus metrics and health check endpoints

use axum::{
    routing::get,
    Router,
    response::{IntoResponse, Response},
    http::StatusCode,
    Json,
};
use std::net::SocketAddr;
use std::path::PathBuf;
use anyhow::Result;
use serde_json::json;

use super::{get_metrics, HealthChecker, HealthStatus};

/// Start the observability HTTP server
pub async fn start_server(port: u16, socket_path: PathBuf) -> Result<()> {
    let health_checker = HealthChecker::new(socket_path);

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/health", get(|| async { health_handler(health_checker.clone()).await }))
        .route("/health/live", get(|| async { liveness_handler(health_checker.clone()).await }))
        .route("/health/ready", get(|| async { readiness_handler(health_checker.clone()).await }));

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("Starting observability server on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Metrics endpoint handler
async fn metrics_handler() -> Response {
    let metrics = get_metrics();
    let body = metrics.export();

    (
        StatusCode::OK,
        [("Content-Type", "text/plain; version=0.0.4")],
        body,
    )
        .into_response()
}

/// Health endpoint handler (full health check)
async fn health_handler(checker: HealthChecker) -> Response {
    let health = checker.check_all().await;

    let status_code = match health.status {
        super::Status::Healthy => StatusCode::OK,
        super::Status::Degraded => StatusCode::OK, // Still accepting traffic
        super::Status::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health)).into_response()
}

/// Liveness probe handler (Kubernetes-style)
async fn liveness_handler(checker: HealthChecker) -> Response {
    let is_alive = checker.check_liveness().await;

    if is_alive {
        (StatusCode::OK, Json(json!({"status": "alive"}))).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"status": "dead"}))).into_response()
    }
}

/// Readiness probe handler (Kubernetes-style)
async fn readiness_handler(checker: HealthChecker) -> Response {
    let is_ready = checker.check_readiness().await;

    if is_ready {
        (StatusCode::OK, Json(json!({"status": "ready"}))).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, Json(json!({"status": "not_ready"}))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_metrics_handler() {
        let response = metrics_handler().await;
        // Response should contain Prometheus metrics
        // In a real test, we'd extract the body and verify format
    }

    #[tokio::test]
    async fn test_liveness_handler() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");
        let checker = HealthChecker::new(socket_path);

        let response = liveness_handler(checker).await;
        // Should return 503 since socket doesn't exist
    }
}
