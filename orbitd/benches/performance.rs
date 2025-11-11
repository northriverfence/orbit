// Performance benchmarks for Orbit daemon
// Measures latency of core operations

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use orbitd::classifier::Classifier;
use orbitd::context::Context;
use std::path::PathBuf;
use tempfile::tempdir;

fn bench_classifier_known_commands(c: &mut Criterion) {
    let classifier = Classifier::new();

    let commands = vec![
        "ls",
        "ls -la",
        "git status",
        "cargo build",
        "docker ps",
        "npm install",
        "echo hello",
        "cat file.txt",
        "grep pattern file",
        "find . -name '*.rs'",
    ];

    c.bench_function("classify_known_simple", |b| {
        b.iter(|| {
            classifier.classify(black_box("ls"))
        });
    });

    c.bench_function("classify_known_with_args", |b| {
        b.iter(|| {
            classifier.classify(black_box("ls -la /tmp"))
        });
    });

    let mut group = c.benchmark_group("classify_known_variety");
    for cmd in &commands {
        group.bench_with_input(BenchmarkId::from_parameter(cmd), cmd, |b, cmd| {
            b.iter(|| classifier.classify(black_box(cmd)));
        });
    }
    group.finish();
}

fn bench_classifier_natural_language(c: &mut Criterion) {
    let classifier = Classifier::new();

    let queries = vec![
        "list all files",
        "show me the current directory",
        "find large files in this folder",
        "what processes are running?",
        "how much disk space is free?",
    ];

    c.bench_function("classify_natural_simple", |b| {
        b.iter(|| {
            classifier.classify(black_box("list all files"))
        });
    });

    let mut group = c.benchmark_group("classify_natural_variety");
    for query in &queries {
        group.bench_with_input(BenchmarkId::from_parameter(query), query, |b, query| {
            b.iter(|| classifier.classify(black_box(query)));
        });
    }
    group.finish();
}

fn bench_classifier_batch(c: &mut Criterion) {
    let classifier = Classifier::new();

    let commands = vec![
        "ls", "ls -la", "pwd", "whoami", "date",
        "git status", "cargo build", "docker ps",
        "npm install", "echo test",
    ];

    c.bench_function("classify_batch_10", |b| {
        b.iter(|| {
            for cmd in &commands {
                classifier.classify(black_box(cmd));
            }
        });
    });

    c.bench_function("classify_batch_100", |b| {
        b.iter(|| {
            for _ in 0..10 {
                for cmd in &commands {
                    classifier.classify(black_box(cmd));
                }
            }
        });
    });
}

fn bench_context_detection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    c.bench_function("context_detect_empty", |b| {
        b.iter(|| {
            rt.block_on(async {
                Context::detect(black_box(temp_dir.path())).await
            })
        });
    });

    // Create a Git repo for benchmarking
    let git_dir = tempdir().unwrap();
    std::process::Command::new("git")
        .args(&["init"])
        .current_dir(git_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.name", "Test"])
        .current_dir(git_dir.path())
        .output()
        .unwrap();

    std::process::Command::new("git")
        .args(&["config", "user.email", "test@test.com"])
        .current_dir(git_dir.path())
        .output()
        .unwrap();

    c.bench_function("context_detect_git_repo", |b| {
        b.iter(|| {
            rt.block_on(async {
                Context::detect(black_box(git_dir.path())).await
            })
        });
    });

    // Create a project with markers
    let project_dir = tempdir().unwrap();
    std::fs::write(project_dir.path().join("Cargo.toml"), "[package]\nname = \"test\"\n").unwrap();
    std::fs::write(project_dir.path().join("main.rs"), "fn main() {}").unwrap();
    std::fs::write(project_dir.path().join("lib.rs"), "pub fn test() {}").unwrap();

    c.bench_function("context_detect_rust_project", |b| {
        b.iter(|| {
            rt.block_on(async {
                Context::detect(black_box(project_dir.path())).await
            })
        });
    });
}

fn bench_context_hash(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let context = rt.block_on(async {
        Context::detect(temp_dir.path()).await.unwrap()
    });

    c.bench_function("context_hash_generation", |b| {
        b.iter(|| {
            context.generate_hash()
        });
    });
}

fn bench_learning_operations(c: &mut Criterion) {
    use orbitd::learning::{LearningSystem, ExecutionResult};

    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().join("bench.db");

    let mut learning = rt.block_on(async {
        LearningSystem::new(&db_path).await.unwrap()
    });

    // Pre-populate with some data
    rt.block_on(async {
        for i in 0..100 {
            learning.record_execution(
                &format!("test command {}", i),
                Some(&format!("cmd {}", i)),
                &format!("cmd {}", i),
                ExecutionResult::Success,
                100,
            ).await.unwrap();
        }
    });

    c.bench_function("learning_record_execution", |b| {
        b.iter(|| {
            rt.block_on(async {
                learning.record_execution(
                    black_box("test"),
                    Some(black_box("echo test")),
                    black_box("echo test"),
                    ExecutionResult::Success,
                    100,
                ).await
            })
        });
    });

    c.bench_function("learning_detect_patterns", |b| {
        b.iter(|| {
            rt.block_on(async {
                learning.detect_patterns().await
            })
        });
    });

    c.bench_function("learning_rank_suggestions", |b| {
        let suggestions = vec![
            "ls".to_string(),
            "ls -la".to_string(),
            "find".to_string(),
        ];

        b.iter(|| {
            rt.block_on(async {
                learning.rank_suggestions(
                    black_box("list files"),
                    black_box(suggestions.clone())
                ).await
            })
        });
    });
}

fn bench_ipc_overhead(c: &mut Criterion) {
    use orbitd::ipc::Message;

    c.bench_function("message_serialization", |b| {
        let msg = Message::Query {
            command: "ls -la".to_string(),
            context_hash: Some("abc123".to_string()),
        };

        b.iter(|| {
            serde_json::to_string(black_box(&msg))
        });
    });

    c.bench_function("message_deserialization", |b| {
        let json = r#"{"Query":{"command":"ls -la","context_hash":"abc123"}}"#;

        b.iter(|| {
            serde_json::from_str::<Message>(black_box(json))
        });
    });
}

fn bench_caching(c: &mut Criterion) {
    use orbitd::cache::ResponseCache;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let context = rt.block_on(async {
        Context::detect(temp_dir.path()).await.unwrap()
    });

    let cache = ResponseCache::new(100, 300); // 100 capacity, 5 min TTL

    c.bench_function("cache_put", |b| {
        let mut i = 0;
        b.iter(|| {
            cache.put(
                black_box(&format!("test command {}", i)),
                black_box(&context),
                black_box("ls -la".to_string())
            );
            i += 1;
        });
    });

    // Populate cache
    for i in 0..50 {
        cache.put(
            &format!("command {}", i),
            &context,
            format!("result {}", i)
        );
    }

    c.bench_function("cache_hit", |b| {
        b.iter(|| {
            cache.get(black_box("command 25"), black_box(&context))
        });
    });

    c.bench_function("cache_miss", |b| {
        b.iter(|| {
            cache.get(black_box("nonexistent command"), black_box(&context))
        });
    });

    // Benchmark different cache sizes
    let mut group = c.benchmark_group("cache_size_scaling");
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            let cache = ResponseCache::new(*size, 300);
            b.iter(|| {
                cache.put("test", &context, "response".to_string());
                cache.get("test", &context);
            });
        });
    }
    group.finish();
}

fn bench_resource_limiter(c: &mut Criterion) {
    use orbitd::optimization::ResourceLimiter;

    let rt = tokio::runtime::Runtime::new().unwrap();

    let limiter = ResourceLimiter::new(1000, 100); // 1000MB, 100 concurrent

    c.bench_function("resource_check_memory", |b| {
        b.iter(|| {
            limiter.check_memory()
        });
    });

    c.bench_function("resource_try_acquire", |b| {
        b.iter(|| {
            limiter.try_acquire_permit()
        });
    });

    c.bench_function("resource_stats", |b| {
        b.iter(|| {
            limiter.stats()
        });
    });

    c.bench_function("resource_acquire_release", |b| {
        b.iter(|| {
            rt.block_on(async {
                let _permit = limiter.acquire_permit().await;
                // Permit automatically released on drop
            })
        });
    });
}

fn bench_rate_limiter(c: &mut Criterion) {
    use orbitd::optimization::resources::RateLimiter;

    // Benchmark with different token rates
    let mut group = c.benchmark_group("rate_limiter_scaling");
    for rate in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(rate), rate, |b, rate| {
            let limiter = RateLimiter::new(*rate, *rate);
            b.iter(|| {
                limiter.try_acquire()
            });
        });
    }
    group.finish();

    let limiter = RateLimiter::new(100, 50);

    c.bench_function("rate_limiter_try_acquire", |b| {
        b.iter(|| {
            limiter.try_acquire()
        });
    });

    c.bench_function("rate_limiter_available_tokens", |b| {
        b.iter(|| {
            limiter.available_tokens()
        });
    });
}

fn bench_batch_operations(c: &mut Criterion) {
    use orbitd::optimization::async_ops::BatchExecutor;

    let rt = tokio::runtime::Runtime::new().unwrap();

    // Benchmark batch executor with different batch sizes
    let mut group = c.benchmark_group("batch_executor_scaling");
    for size in [10, 50, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, size| {
            b.iter(|| {
                let mut executor = BatchExecutor::new(*size);
                for i in 0..*size {
                    executor.add(i);
                }
                rt.block_on(async {
                    executor.execute(|x| async move { Ok::<i32, anyhow::Error>(x * 2) }).await
                });
            });
        });
    }
    group.finish();
}

fn bench_end_to_end_latency(c: &mut Criterion) {
    // This would benchmark full daemon round-trip in a real integration test
    // For now, we'll benchmark the components
    let classifier = Classifier::new();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    c.bench_function("e2e_known_command", |b| {
        b.iter(|| {
            let _classification = classifier.classify(black_box("ls -la"));
            // In real scenario, would query daemon and get response
        });
    });

    c.bench_function("e2e_with_context", |b| {
        b.iter(|| {
            let _classification = classifier.classify(black_box("show files"));
            let _context = rt.block_on(async {
                Context::detect(black_box(temp_dir.path())).await
            });
            // In real scenario, would send to AI provider
        });
    });
}

criterion_group!(
    benches,
    bench_classifier_known_commands,
    bench_classifier_natural_language,
    bench_classifier_batch,
    bench_context_detection,
    bench_context_hash,
    bench_learning_operations,
    bench_ipc_overhead,
    bench_caching,
    bench_resource_limiter,
    bench_rate_limiter,
    bench_batch_operations,
    bench_end_to_end_latency
);

criterion_main!(benches);
