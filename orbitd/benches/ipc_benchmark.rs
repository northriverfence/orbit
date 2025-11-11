// IPC Performance Benchmarks
//
// Benchmarks for Named Pipes (Windows) and Unix Domain Sockets (Unix)
// using the criterion benchmarking framework.
//
// Run with: cargo bench --bench ipc_benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(windows)]
use tokio::net::windows::named_pipe::{ClientOptions, ServerOptions};

#[cfg(unix)]
use tokio::net::UnixStream;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkRequest {
    #[serde(rename = "type")]
    request_type: String,
    payload: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkResponse {
    status: String,
    data: Option<String>,
}

/// Serialize a message with 4-byte length prefix
fn serialize_message<T: Serialize>(msg: &T) -> Vec<u8> {
    let json = serde_json::to_vec(msg).expect("Failed to serialize");
    let len = json.len() as u32;
    let mut buffer = len.to_le_bytes().to_vec();
    buffer.extend_from_slice(&json);
    buffer
}

/// Benchmark: JSON serialization overhead
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    group.bench_function("small_request", |b| {
        let request = BenchmarkRequest {
            request_type: "status".to_string(),
            payload: None,
        };
        b.iter(|| {
            let _ = black_box(serialize_message(&request));
        });
    });

    group.bench_function("large_request", |b| {
        let payload = "x".repeat(10240); // 10KB
        let request = BenchmarkRequest {
            request_type: "query".to_string(),
            payload: Some(payload),
        };
        b.iter(|| {
            let _ = black_box(serialize_message(&request));
        });
    });

    group.finish();
}

/// Benchmark: Named Pipe connection establishment (Windows)
#[cfg(windows)]
fn bench_named_pipe_connection(c: &mut Criterion) {
    use std::sync::Arc;
    use tokio::sync::Barrier;

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("named_pipe");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("connection_establishment", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let pipe_name = format!(r"\\.\pipe\orbit-bench-{}", uuid::Uuid::new_v4());

                // Server task
                let server_name = pipe_name.clone();
                let server_task = tokio::spawn(async move {
                    let server = ServerOptions::new()
                        .first_pipe_instance(true)
                        .create(&server_name)
                        .expect("Failed to create server");

                    server.connect().await.expect("Failed to connect");
                });

                // Client task
                tokio::time::sleep(Duration::from_millis(10)).await;

                let client = ClientOptions::new()
                    .open(&pipe_name)
                    .expect("Failed to open client");

                server_task.await.expect("Server task failed");
                black_box(client);
            });
        });
    });

    group.finish();
}

/// Benchmark: Unix socket connection establishment (Unix)
#[cfg(unix)]
fn bench_unix_socket_connection(c: &mut Criterion) {
    use std::path::PathBuf;
    use tokio::net::UnixListener;

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("unix_socket");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("connection_establishment", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let socket_path = PathBuf::from(format!("/tmp/orbit-bench-{}.sock", uuid::Uuid::new_v4()));

                // Clean up if exists
                let _ = std::fs::remove_file(&socket_path);

                // Server task
                let server_path = socket_path.clone();
                let server_task = tokio::spawn(async move {
                    let listener = UnixListener::bind(&server_path).expect("Failed to bind");
                    let (stream, _) = listener.accept().await.expect("Failed to accept");
                    black_box(stream);
                });

                // Client task
                tokio::time::sleep(Duration::from_millis(10)).await;

                let client = UnixStream::connect(&socket_path).await.expect("Failed to connect");

                server_task.await.expect("Server task failed");

                // Cleanup
                let _ = std::fs::remove_file(&socket_path);
                black_box(client);
            });
        });
    });

    group.finish();
}

/// Benchmark: Message roundtrip latency
#[cfg(windows)]
fn bench_message_roundtrip(c: &mut Criterion) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_roundtrip");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("small_message", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let pipe_name = format!(r"\\.\pipe\orbit-bench-{}", uuid::Uuid::new_v4());

                let server_name = pipe_name.clone();
                let server_task = tokio::spawn(async move {
                    let mut server = ServerOptions::new()
                        .first_pipe_instance(true)
                        .create(&server_name)
                        .expect("Failed to create server");

                    server.connect().await.expect("Failed to connect");

                    // Echo server
                    let mut length_buf = [0u8; 4];
                    server.read_exact(&mut length_buf).await.expect("Failed to read length");
                    let length = u32::from_le_bytes(length_buf) as usize;

                    let mut data = vec![0u8; length];
                    server.read_exact(&mut data).await.expect("Failed to read data");

                    // Echo back
                    server.write_all(&length_buf).await.expect("Failed to write length");
                    server.write_all(&data).await.expect("Failed to write data");
                    server.flush().await.expect("Failed to flush");
                });

                tokio::time::sleep(Duration::from_millis(10)).await;

                let mut client = ClientOptions::new()
                    .open(&pipe_name)
                    .expect("Failed to open client");

                let request = BenchmarkRequest {
                    request_type: "status".to_string(),
                    payload: None,
                };
                let message = serialize_message(&request);

                // Send
                client.write_all(&message).await.expect("Failed to write");
                client.flush().await.expect("Failed to flush");

                // Receive
                let mut length_buf = [0u8; 4];
                client.read_exact(&mut length_buf).await.expect("Failed to read length");
                let length = u32::from_le_bytes(length_buf) as usize;

                let mut response = vec![0u8; length];
                client.read_exact(&mut response).await.expect("Failed to read response");

                server_task.await.expect("Server task failed");
                black_box(response);
            });
        });
    });

    group.bench_function("large_message", |b| {
        b.iter(|| {
            runtime.block_on(async {
                let pipe_name = format!(r"\\.\pipe\orbit-bench-{}", uuid::Uuid::new_v4());

                let server_name = pipe_name.clone();
                let server_task = tokio::spawn(async move {
                    let mut server = ServerOptions::new()
                        .first_pipe_instance(true)
                        .create(&server_name)
                        .expect("Failed to create server");

                    server.connect().await.expect("Failed to connect");

                    let mut length_buf = [0u8; 4];
                    server.read_exact(&mut length_buf).await.expect("Failed to read length");
                    let length = u32::from_le_bytes(length_buf) as usize;

                    let mut data = vec![0u8; length];
                    server.read_exact(&mut data).await.expect("Failed to read data");

                    server.write_all(&length_buf).await.expect("Failed to write length");
                    server.write_all(&data).await.expect("Failed to write data");
                    server.flush().await.expect("Failed to flush");
                });

                tokio::time::sleep(Duration::from_millis(10)).await;

                let mut client = ClientOptions::new()
                    .open(&pipe_name)
                    .expect("Failed to open client");

                let payload = "x".repeat(10240); // 10KB
                let request = BenchmarkRequest {
                    request_type: "query".to_string(),
                    payload: Some(payload),
                };
                let message = serialize_message(&request);

                client.write_all(&message).await.expect("Failed to write");
                client.flush().await.expect("Failed to flush");

                let mut length_buf = [0u8; 4];
                client.read_exact(&mut length_buf).await.expect("Failed to read length");
                let length = u32::from_le_bytes(length_buf) as usize;

                let mut response = vec![0u8; length];
                client.read_exact(&mut response).await.expect("Failed to read response");

                server_task.await.expect("Server task failed");
                black_box(response);
            });
        });
    });

    group.finish();
}

/// Benchmark: Throughput (messages per second)
#[cfg(windows)]
fn bench_throughput(c: &mut Criterion) {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("throughput");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for message_count in [100, 1000, 10000].iter() {
        group.throughput(Throughput::Elements(*message_count as u64));

        group.bench_with_input(
            format!("{}_messages", message_count),
            message_count,
            |b, &count| {
                b.iter(|| {
                    runtime.block_on(async {
                        let pipe_name = format!(r"\\.\pipe\orbit-bench-{}", uuid::Uuid::new_v4());

                        let server_name = pipe_name.clone();
                        let server_task = tokio::spawn(async move {
                            let mut server = ServerOptions::new()
                                .first_pipe_instance(true)
                                .create(&server_name)
                                .expect("Failed to create server");

                            server.connect().await.expect("Failed to connect");

                            for _ in 0..count {
                                let mut length_buf = [0u8; 4];
                                server.read_exact(&mut length_buf).await.expect("Failed to read");
                                let length = u32::from_le_bytes(length_buf) as usize;

                                let mut data = vec![0u8; length];
                                server.read_exact(&mut data).await.expect("Failed to read data");

                                server.write_all(&length_buf).await.expect("Failed to write");
                                server.write_all(&data).await.expect("Failed to write data");
                            }

                            server.flush().await.expect("Failed to flush");
                        });

                        tokio::time::sleep(Duration::from_millis(10)).await;

                        let mut client = ClientOptions::new()
                            .open(&pipe_name)
                            .expect("Failed to open client");

                        let request = BenchmarkRequest {
                            request_type: "status".to_string(),
                            payload: None,
                        };
                        let message = serialize_message(&request);

                        for _ in 0..count {
                            client.write_all(&message).await.expect("Failed to write");
                            client.flush().await.expect("Failed to flush");

                            let mut length_buf = [0u8; 4];
                            client.read_exact(&mut length_buf).await.expect("Failed to read");
                            let length = u32::from_le_bytes(length_buf) as usize;

                            let mut response = vec![0u8; length];
                            client.read_exact(&mut response).await.expect("Failed to read response");
                        }

                        server_task.await.expect("Server task failed");
                    });
                });
            },
        );
    }

    group.finish();
}

// Criterion benchmark groups
criterion_group!(
    benches,
    bench_serialization,
    #[cfg(windows)]
    bench_named_pipe_connection,
    #[cfg(windows)]
    bench_message_roundtrip,
    #[cfg(windows)]
    bench_throughput,
    #[cfg(unix)]
    bench_unix_socket_connection,
);

criterion_main!(benches);
