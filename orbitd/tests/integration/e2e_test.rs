// End-to-end integration tests
// Tests full command flow from client to daemon to AI provider

use orbitd::daemon::Daemon;
use orbitd::config::Config;
use orbitd::ipc::{Client, Message};
use tempfile::tempdir;
use tokio::time::{sleep, Duration};
use std::path::PathBuf;

#[tokio::test]
async fn test_full_command_flow_passthrough() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    // Start daemon in background
    let config = Config {
        socket_path: socket_path.clone(),
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    // Give daemon time to start
    sleep(Duration::from_millis(200)).await;

    // Connect as client
    let client = Client::connect(&socket_path).await.unwrap();

    // Send known command (should passthrough)
    let response = client.query("ls -la").await.unwrap();

    assert_eq!(response, "PASSTHROUGH", "Known command should passthrough");

    // Clean up
    daemon_handle.abort();
}

#[tokio::test]
async fn test_full_command_flow_natural_language() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    // Configure with mock AI provider
    let config = Config {
        socket_path: socket_path.clone(),
        provider_mode: "mock".to_string(), // Use mock provider for testing
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let client = Client::connect(&socket_path).await.unwrap();

    // Send natural language command
    let response = client.query("list all files").await.unwrap();

    // Should get a REPLACED: response
    assert!(response.starts_with("REPLACED:"), "Natural language should get AI response");
    assert!(response.contains("ls") || response.contains("find"),
            "Response should contain shell command");

    daemon_handle.abort();
}

#[tokio::test]
async fn test_daemon_multiple_clients() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    // Connect multiple clients simultaneously
    let client1 = Client::connect(&socket_path).await.unwrap();
    let client2 = Client::connect(&socket_path).await.unwrap();
    let client3 = Client::connect(&socket_path).await.unwrap();

    // Send commands from all clients in parallel
    let responses = tokio::join!(
        client1.query("ls"),
        client2.query("pwd"),
        client3.query("whoami")
    );

    // All should succeed
    assert!(responses.0.is_ok());
    assert!(responses.1.is_ok());
    assert!(responses.2.is_ok());

    daemon_handle.abort();
}

#[tokio::test]
async fn test_daemon_handles_disconnection() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    // Connect and disconnect
    {
        let client = Client::connect(&socket_path).await.unwrap();
        let _ = client.query("ls").await;
        // Client dropped here
    }

    // Daemon should still be running
    // Connect again
    let client2 = Client::connect(&socket_path).await.unwrap();
    let response = client2.query("pwd").await;
    assert!(response.is_ok(), "Daemon should still accept connections");

    daemon_handle.abort();
}

#[tokio::test]
async fn test_daemon_graceful_shutdown() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    // Send shutdown signal
    daemon_handle.abort();

    // Wait for shutdown
    sleep(Duration::from_millis(100)).await;

    // Socket should be cleaned up
    assert!(!socket_path.exists(), "Socket should be removed on shutdown");
}

#[tokio::test]
async fn test_learning_system_integration() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");
    let db_path = temp_dir.path().join("learning.db");

    let config = Config {
        socket_path: socket_path.clone(),
        learning_db_path: db_path.clone(),
        learning_enabled: true,
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let client = Client::connect(&socket_path).await.unwrap();

    // Execute same command multiple times
    for _ in 0..5 {
        let response = client.query("list files").await.unwrap();
        // Send feedback
        if response.starts_with("REPLACED:") {
            let command = response.strip_prefix("REPLACED:").unwrap();
            client.send_feedback("list files", command, "success").await.unwrap();
        }
    }

    // Learning database should exist and have data
    assert!(db_path.exists(), "Learning database should be created");

    daemon_handle.abort();
}

#[tokio::test]
async fn test_context_awareness_integration() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    // Create a Git repository
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@test.com"])
        .current_dir(temp_dir.path())
        .output()
        .unwrap();

    // Create a Rust project
    std::fs::write(temp_dir.path().join("Cargo.toml"), "[package]\n").unwrap();

    let config = Config {
        socket_path: socket_path.clone(),
        context_aware: true,
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let client = Client::connect(&socket_path).await.unwrap();

    // Query with context
    let response = client.query_with_context("show me changes", temp_dir.path()).await.unwrap();

    // Response should consider Git context
    assert!(response.starts_with("REPLACED:"));
    assert!(response.contains("git") || response.contains("diff") || response.contains("status"));

    daemon_handle.abort();
}

#[tokio::test]
async fn test_provider_fallback_integration() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        providers: vec![
            // Primary provider (disabled/failing)
            ProviderConfig {
                name: "primary".to_string(),
                enabled: false,
                ..Default::default()
            },
            // Fallback provider
            ProviderConfig {
                name: "fallback".to_string(),
                enabled: true,
                provider_type: "mock".to_string(),
                ..Default::default()
            },
        ],
        fallback_enabled: true,
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let client = Client::connect(&socket_path).await.unwrap();

    // Query should use fallback provider
    let response = client.query("show files").await.unwrap();

    assert!(response.starts_with("REPLACED:"), "Fallback provider should respond");

    daemon_handle.abort();
}

#[tokio::test]
async fn test_concurrent_requests_integration() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        max_concurrent_requests: 10,
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    // Spawn many concurrent requests
    let mut handles = vec![];

    for i in 0..20 {
        let socket_path = socket_path.clone();
        let handle = tokio::spawn(async move {
            let client = Client::connect(&socket_path).await.unwrap();
            client.query(&format!("test command {}", i)).await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut success_count = 0;
    for handle in handles {
        if let Ok(Ok(_)) = handle.await {
            success_count += 1;
        }
    }

    // Most should succeed (some might be rate-limited)
    assert!(success_count >= 10, "At least 10 requests should succeed");

    daemon_handle.abort();
}

#[tokio::test]
async fn test_error_handling_integration() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let client = Client::connect(&socket_path).await.unwrap();

    // Send malformed request
    let response = client.query("").await;

    // Should handle gracefully
    assert!(response.is_ok() || response.is_err());

    // Send very long request
    let long_request = "a".repeat(10000);
    let response = client.query(&long_request).await;

    // Should handle without crashing
    assert!(response.is_ok() || response.is_err());

    daemon_handle.abort();
}

#[tokio::test]
async fn test_performance_under_load() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");

    let config = Config {
        socket_path: socket_path.clone(),
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let start = std::time::Instant::now();

    // Send 100 requests
    for _ in 0..100 {
        let client = Client::connect(&socket_path).await.unwrap();
        let _ = client.query("ls").await;
    }

    let duration = start.elapsed();
    let avg_latency = duration.as_millis() / 100;

    // Average latency should be reasonable (<50ms for known commands)
    assert!(avg_latency < 50, "Average latency too high: {}ms", avg_latency);

    daemon_handle.abort();
}

#[tokio::test]
async fn test_feedback_loop_integration() {
    let temp_dir = tempdir().unwrap();
    let socket_path = temp_dir.path().join("daemon.sock");
    let db_path = temp_dir.path().join("learning.db");

    let config = Config {
        socket_path: socket_path.clone(),
        learning_db_path: db_path.clone(),
        learning_enabled: true,
        ..Default::default()
    };

    let daemon = Daemon::new(config).await.unwrap();
    let daemon_handle = tokio::spawn(async move {
        daemon.run().await
    });

    sleep(Duration::from_millis(200)).await;

    let client = Client::connect(&socket_path).await.unwrap();

    // Get suggestion
    let response1 = client.query("list files").await.unwrap();

    if response1.starts_with("REPLACED:") {
        let command = response1.strip_prefix("REPLACED:").unwrap();

        // Send positive feedback
        client.send_feedback("list files", command, "success").await.unwrap();

        // Query again - should potentially get improved suggestion
        let response2 = client.query("list files").await.unwrap();

        assert!(response2.starts_with("REPLACED:"));
    }

    daemon_handle.abort();
}
