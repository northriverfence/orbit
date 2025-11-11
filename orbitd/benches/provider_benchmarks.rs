// Provider performance benchmarks
// Measures AI provider response times and throughput

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use orbitd::providers::{ProviderManager, ProviderConfig};
use orbitd::context::Context;
use std::path::PathBuf;
use tempfile::tempdir;

fn bench_provider_selection(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let config = create_test_config();
    let manager = rt.block_on(async {
        ProviderManager::new(config).await.unwrap()
    });

    c.bench_function("provider_select_cheapest", |b| {
        b.iter(|| {
            rt.block_on(async {
                manager.select_provider(black_box(SelectionCriteria::Cheapest)).await
            })
        });
    });

    c.bench_function("provider_select_fastest", |b| {
        b.iter(|| {
            rt.block_on(async {
                manager.select_provider(black_box(SelectionCriteria::Fastest)).await
            })
        });
    });

    c.bench_function("provider_select_user_preferred", |b| {
        b.iter(|| {
            rt.block_on(async {
                manager.select_provider(black_box(SelectionCriteria::UserPreferred)).await
            })
        });
    });
}

fn bench_provider_routing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let config = create_test_config();
    let manager = rt.block_on(async {
        ProviderManager::new(config).await.unwrap()
    });

    let context = rt.block_on(async {
        Context::detect(temp_dir.path()).await.unwrap()
    });

    let request = Request {
        prompt: "test query".to_string(),
        context: context.clone(),
    };

    c.bench_function("provider_routing_simple", |b| {
        b.iter(|| {
            rt.block_on(async {
                // In real benchmark, this would query actual provider
                // For now, just measure routing overhead
                manager.select_provider(SelectionCriteria::UserPreferred).await
            })
        });
    });
}

fn bench_cost_tracking(c: &mut Criterion) {
    use orbitd::providers::CostTracker;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut tracker = CostTracker::new();

    c.bench_function("cost_record_usage", |b| {
        b.iter(|| {
            rt.block_on(async {
                tracker.record_usage(
                    black_box("claude"),
                    black_box(1000),
                    black_box(500),
                    black_box(0.003),
                    black_box(0.015)
                ).await
            })
        });
    });

    // Pre-populate with data
    rt.block_on(async {
        for i in 0..100 {
            tracker.record_usage("claude", 1000, 500, 0.003, 0.015).await.unwrap();
            tracker.record_usage("openai", 800, 400, 0.03, 0.06).await.unwrap();
        }
    });

    c.bench_function("cost_get_total", |b| {
        b.iter(|| {
            tracker.get_total_cost()
        });
    });

    c.bench_function("cost_by_provider", |b| {
        b.iter(|| {
            rt.block_on(async {
                tracker.get_cost_by_provider().await
            })
        });
    });
}

fn bench_provider_health_checks(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let config = create_test_config();
    let manager = rt.block_on(async {
        ProviderManager::new(config).await.unwrap()
    });

    c.bench_function("health_check_single", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Mock health check
                // In real benchmark, would ping actual provider
                std::future::ready(Ok(true))
            })
        });
    });

    c.bench_function("health_check_all", |b| {
        b.iter(|| {
            rt.block_on(async {
                manager.health_check_all().await
            })
        });
    });
}

fn bench_provider_caching(c: &mut Criterion) {
    use orbitd::cache::ResponseCache;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let context = rt.block_on(async {
        Context::detect(temp_dir.path()).await.unwrap()
    });

    let mut cache = ResponseCache::new(1000, 300);

    // Populate cache with responses
    for i in 0..500 {
        cache.put(
            &format!("query {}", i),
            &context,
            format!("response {}", i)
        );
    }

    c.bench_function("cache_hit_provider_response", |b| {
        b.iter(|| {
            cache.get(black_box("query 250"), black_box(&context))
        });
    });

    c.bench_function("cache_miss_provider_response", |b| {
        b.iter(|| {
            cache.get(black_box("nonexistent query"), black_box(&context))
        });
    });

    c.bench_function("cache_put_provider_response", |b| {
        let mut i = 500;
        b.iter(|| {
            cache.put(
                black_box(&format!("query {}", i)),
                black_box(&context),
                black_box(format!("response {}", i))
            );
            i += 1;
        });
    });
}

fn bench_parallel_provider_queries(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    let config = create_test_config();
    let manager = rt.block_on(async {
        ProviderManager::new(config).await.unwrap()
    });

    let context = rt.block_on(async {
        Context::detect(temp_dir.path()).await.unwrap()
    });

    c.bench_function("parallel_query_2_providers", |b| {
        b.iter(|| {
            rt.block_on(async {
                // In real benchmark, would query 2 providers in parallel
                // For now, just measure overhead
                tokio::join!(
                    std::future::ready(Ok("response1")),
                    std::future::ready(Ok("response2"))
                )
            })
        });
    });

    c.bench_function("parallel_query_3_providers", |b| {
        b.iter(|| {
            rt.block_on(async {
                tokio::join!(
                    std::future::ready(Ok("response1")),
                    std::future::ready(Ok("response2")),
                    std::future::ready(Ok("response3"))
                )
            })
        });
    });
}

fn bench_rate_limiting(c: &mut Criterion) {
    use orbitd::security::RateLimitManager;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let mut limiter = RateLimitManager::new();

    c.bench_function("rate_limit_check", |b| {
        b.iter(|| {
            rt.block_on(async {
                limiter.check_limit(black_box("test_user")).await
            })
        });
    });
}

fn bench_prompt_building(c: &mut Criterion) {
    use orbitd::context::ContextAwarePromptBuilder;

    let rt = tokio::runtime::Runtime::new().unwrap();
    let temp_dir = tempdir().unwrap();

    // Create rich context
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

    std::fs::write(git_dir.path().join("Cargo.toml"), "[package]\n").unwrap();

    let context = rt.block_on(async {
        Context::detect(git_dir.path()).await.unwrap()
    });

    let builder = ContextAwarePromptBuilder::new(context);

    c.bench_function("build_context_aware_prompt", |b| {
        b.iter(|| {
            builder.build_prompt(black_box("show me files"))
        });
    });

    c.bench_function("build_simple_prompt", |b| {
        let simple_context = rt.block_on(async {
            Context::detect(temp_dir.path()).await.unwrap()
        });
        let simple_builder = ContextAwarePromptBuilder::new(simple_context);

        b.iter(|| {
            simple_builder.build_prompt(black_box("list files"))
        });
    });
}

// Helper functions

fn create_test_config() -> ProviderManagerConfig {
    ProviderManagerConfig {
        providers: vec![
            ProviderConfig {
                name: "claude".to_string(),
                enabled: true,
                provider_type: "mock".to_string(),
                api_key: "test_key".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                max_tokens: 1000,
                temperature: 0.7,
                priority: 1,
            },
            ProviderConfig {
                name: "openai".to_string(),
                enabled: true,
                provider_type: "mock".to_string(),
                api_key: "test_key".to_string(),
                model: "gpt-4".to_string(),
                max_tokens: 1000,
                temperature: 0.7,
                priority: 2,
            },
            ProviderConfig {
                name: "local".to_string(),
                enabled: true,
                provider_type: "mock".to_string(),
                api_key: "".to_string(),
                model: "llama2".to_string(),
                max_tokens: 1000,
                temperature: 0.7,
                priority: 3,
            },
        ],
        default_provider: "claude".to_string(),
        fallback_chain: vec!["claude".to_string(), "openai".to_string()],
        timeout_seconds: 30,
        max_retries: 3,
    }
}

use orbitd::providers::SelectionCriteria;

struct Request {
    prompt: String,
    context: Context,
}

struct ProviderManagerConfig {
    providers: Vec<ProviderConfig>,
    default_provider: String,
    fallback_chain: Vec<String>,
    timeout_seconds: u64,
    max_retries: u32,
}

criterion_group!(
    benches,
    bench_provider_selection,
    bench_provider_routing,
    bench_cost_tracking,
    bench_provider_health_checks,
    bench_provider_caching,
    bench_parallel_provider_queries,
    bench_rate_limiting,
    bench_prompt_building
);

criterion_main!(benches);
