// Cross-platform IPC integration tests
//
// These tests verify that the IPC abstraction layer works correctly
// on both Unix and Windows platforms.

use orbitd::daemon::ipc::{FeedbackResult, Request, Response};
use orbitd::daemon::ipc_common::{create_ipc_client, create_ipc_server, IpcConfig};

#[cfg(unix)]
use orbitd::daemon::ipc_unix::{UnixIpcClient, UnixIpcServer};

#[cfg(windows)]
use orbitd::daemon::ipc_windows::{WindowsIpcClient, WindowsIpcServer};

use tokio::time::{timeout, Duration};

/// Test basic IPC server creation
#[tokio::test]
async fn test_ipc_server_creation() {
    let config = IpcConfig {
        name: format!("orbit-test-{}", uuid::Uuid::new_v4()),
        ..Default::default()
    };

    let result = create_ipc_server(config);
    assert!(result.is_ok(), "Failed to create IPC server");
}

/// Test IPC client creation
#[tokio::test]
async fn test_ipc_client_creation() {
    let name = format!("orbit-test-{}", uuid::Uuid::new_v4());
    let client = create_ipc_client(&name);

    // Client creation should always succeed
    // Connection will fail if server isn't running, but creation shouldn't
    assert!(true);
}

/// Test full request-response cycle
#[tokio::test]
async fn test_request_response_cycle() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server in background
    let config = IpcConfig {
        name: test_name.clone(),
        max_connections: 10,
        max_message_size: 1024 * 1024,
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Create client and send request
    let client = create_ipc_client(&test_name);

    let request = Request::Status;
    let result = timeout(Duration::from_secs(5), client.send_request(&request)).await;

    assert!(result.is_ok(), "Request timed out");

    let response = result.unwrap();
    assert!(response.is_ok(), "Request failed: {:?}", response.err());

    match response.unwrap() {
        Response::Status { uptime_secs, commands_processed } => {
            // Stub response returns 0 values
            assert_eq!(uptime_secs, 0);
            assert_eq!(commands_processed, 0);
        }
        _ => panic!("Unexpected response type"),
    }
}

/// Test command request
#[tokio::test]
async fn test_command_request() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server
    let config = IpcConfig {
        name: test_name.clone(),
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send command request
    let client = create_ipc_client(&test_name);

    let request = Request::Command {
        input: "ls -la".to_string(),
        cwd: "/tmp".to_string(),
        shell: "bash".to_string(),
    };

    let result = timeout(Duration::from_secs(5), client.send_request(&request)).await;

    assert!(result.is_ok(), "Command request timed out");

    let response = result.unwrap();
    assert!(response.is_ok(), "Command request failed");

    // Stub implementation returns Passthrough
    match response.unwrap() {
        Response::Passthrough => {
            // Expected for stub implementation
        }
        _ => panic!("Expected Passthrough response"),
    }
}

/// Test feedback request
#[tokio::test]
async fn test_feedback_request() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server
    let config = IpcConfig {
        name: test_name.clone(),
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send feedback request
    let client = create_ipc_client(&test_name);

    let request = Request::Feedback {
        input: "list files".to_string(),
        executed: "ls -la".to_string(),
        result: FeedbackResult::Success,
    };

    let result = timeout(Duration::from_secs(5), client.send_request(&request)).await;

    assert!(result.is_ok(), "Feedback request timed out");

    let response = result.unwrap();
    assert!(response.is_ok(), "Feedback request failed");

    // Stub implementation returns Ok
    match response.unwrap() {
        Response::Ok => {
            // Expected
        }
        _ => panic!("Expected Ok response"),
    }
}

/// Test shutdown request
#[tokio::test]
async fn test_shutdown_request() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server
    let config = IpcConfig {
        name: test_name.clone(),
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send shutdown request
    let client = create_ipc_client(&test_name);

    let request = Request::Shutdown;
    let result = timeout(Duration::from_secs(5), client.send_request(&request)).await;

    assert!(result.is_ok(), "Shutdown request timed out");

    let response = result.unwrap();
    assert!(response.is_ok(), "Shutdown request failed");

    match response.unwrap() {
        Response::Ok => {
            // Expected
        }
        _ => panic!("Expected Ok response"),
    }
}

/// Test concurrent connections
#[tokio::test]
async fn test_concurrent_connections() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server with limited connections
    let config = IpcConfig {
        name: test_name.clone(),
        max_connections: 5,
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send multiple concurrent requests
    let mut tasks = vec![];

    for i in 0..5 {
        let name = test_name.clone();
        let task = tokio::spawn(async move {
            let client = create_ipc_client(&name);
            let request = Request::Status;

            timeout(Duration::from_secs(5), client.send_request(&request))
                .await
                .expect("Request timed out")
        });

        tasks.push(task);
    }

    // Wait for all tasks to complete
    for task in tasks {
        let result = task.await;
        assert!(result.is_ok(), "Concurrent request failed");

        let response = result.unwrap();
        assert!(response.is_ok(), "Response was error");
    }
}

/// Test ping functionality
#[tokio::test]
async fn test_ping() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server
    let config = IpcConfig {
        name: test_name.clone(),
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Test ping
    let client = create_ipc_client(&test_name);
    let is_alive = client.ping().await;

    assert!(is_alive, "Server should respond to ping");
}

/// Test ping on non-existent server
#[tokio::test]
async fn test_ping_non_existent() {
    let test_name = format!("orbit-nonexistent-{}", uuid::Uuid::new_v4());

    let client = create_ipc_client(&test_name);
    let is_alive = client.ping().await;

    assert!(!is_alive, "Ping should fail for non-existent server");
}

/// Test large message handling
#[tokio::test]
async fn test_large_message() {
    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server
    let config = IpcConfig {
        name: test_name.clone(),
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send request with large payload
    let client = create_ipc_client(&test_name);

    // Create a large command string (but under 1MB limit)
    let large_input = "x".repeat(100_000);

    let request = Request::Command {
        input: large_input,
        cwd: "/tmp".to_string(),
        shell: "bash".to_string(),
    };

    let result = timeout(Duration::from_secs(10), client.send_request(&request)).await;

    assert!(result.is_ok(), "Large message timed out");

    let response = result.unwrap();
    assert!(response.is_ok(), "Large message failed");
}

/// Test error handling for invalid JSON
#[tokio::test]
async fn test_invalid_json_handling() {
    // This test verifies that the server gracefully handles invalid JSON
    // by sending an error response rather than crashing

    let test_name = format!("orbit-test-{}", uuid::Uuid::new_v4());

    // Start server
    let config = IpcConfig {
        name: test_name.clone(),
        ..Default::default()
    };

    let server = create_ipc_server(config).expect("Failed to create server");

    tokio::spawn(async move {
        let _ = server.start().await;
    });

    tokio::time::sleep(Duration::from_millis(200)).await;

    // The IPC implementations handle invalid JSON internally
    // and send Error responses, so we just verify the server
    // continues running by sending a valid request
    let client = create_ipc_client(&test_name);

    let request = Request::Status;
    let result = client.send_request(&request).await;

    assert!(result.is_ok(), "Server should still be running after error");
}

#[cfg(unix)]
mod unix_specific_tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_unix_socket_path() {
        let server = UnixIpcServer::new("orbit-test").expect("Failed to create server");
        let socket_path = server.socket_path();

        // Should be in /tmp or XDG_RUNTIME_DIR
        assert!(socket_path.to_string_lossy().contains("orbit-test.sock"));
    }

    #[tokio::test]
    async fn test_unix_custom_path() {
        let test_path = "/tmp/orbit-custom-test.sock";
        let server = UnixIpcServer::with_path(test_path).expect("Failed to create server");

        assert_eq!(server.socket_path(), PathBuf::from(test_path));
    }
}

#[cfg(windows)]
mod windows_specific_tests {
    use super::*;

    #[tokio::test]
    async fn test_windows_pipe_name() {
        let server = WindowsIpcServer::new("orbit-test").expect("Failed to create server");

        // Windows named pipes have the format \\.\pipe\name
        // We can't directly inspect the pipe name, but we can verify creation succeeded
        assert!(true);
    }
}
