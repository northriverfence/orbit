// Prometheus metrics for Orbit daemon
// Tracks commands, latency, cache hits, errors, and resource usage

use prometheus::{
    Counter, Histogram, IntGauge, Registry, Encoder, TextEncoder,
    Opts, HistogramOpts,
};
use std::sync::Once;
use std::sync::Mutex;

static INIT: Once = Once::new();
static mut METRICS: Option<Metrics> = None;

/// Global metrics instance
pub struct Metrics {
    pub registry: Registry,

    // Command metrics
    pub commands_total: Counter,
    pub commands_duration: Histogram,
    pub commands_by_type: prometheus::CounterVec,

    // AI provider metrics
    pub ai_queries_total: Counter,
    pub ai_query_duration: Histogram,
    pub ai_queries_by_provider: prometheus::CounterVec,
    pub ai_tokens_used: prometheus::HistogramVec,

    // Learning system metrics
    pub learning_queries_total: Counter,
    pub learning_query_duration: Histogram,
    pub learning_patterns_detected: IntGauge,

    // Cache metrics
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub cache_size: IntGauge,

    // Error metrics
    pub errors_total: prometheus::CounterVec,

    // Connection metrics
    pub active_connections: IntGauge,
    pub connection_errors: Counter,

    // System metrics
    pub memory_usage_bytes: IntGauge,
    pub cpu_usage_percent: prometheus::Gauge,
}

impl Metrics {
    /// Initialize the global metrics instance
    pub fn init() -> &'static Metrics {
        unsafe {
            INIT.call_once(|| {
                let registry = Registry::new();

                // Command metrics
                let commands_total = Counter::with_opts(
                    Opts::new("orbit_commands_total", "Total number of commands processed")
                ).unwrap();
                registry.register(Box::new(commands_total.clone())).unwrap();

                let commands_duration = Histogram::with_opts(
                    HistogramOpts::new("orbit_commands_duration_seconds", "Command processing duration")
                        .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0])
                ).unwrap();
                registry.register(Box::new(commands_duration.clone())).unwrap();

                let commands_by_type = prometheus::CounterVec::new(
                    Opts::new("orbit_commands_by_type", "Commands grouped by type"),
                    &["type"]
                ).unwrap();
                registry.register(Box::new(commands_by_type.clone())).unwrap();

                // AI provider metrics
                let ai_queries_total = Counter::with_opts(
                    Opts::new("orbit_ai_queries_total", "Total AI provider queries")
                ).unwrap();
                registry.register(Box::new(ai_queries_total.clone())).unwrap();

                let ai_query_duration = Histogram::with_opts(
                    HistogramOpts::new("orbit_ai_query_duration_seconds", "AI provider query duration")
                        .buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0])
                ).unwrap();
                registry.register(Box::new(ai_query_duration.clone())).unwrap();

                let ai_queries_by_provider = prometheus::CounterVec::new(
                    Opts::new("orbit_ai_queries_by_provider", "AI queries grouped by provider"),
                    &["provider"]
                ).unwrap();
                registry.register(Box::new(ai_queries_by_provider.clone())).unwrap();

                let ai_tokens_used = prometheus::HistogramVec::new(
                    HistogramOpts::new("orbit_ai_tokens_used", "AI tokens used per request")
                        .buckets(vec![100.0, 500.0, 1000.0, 2000.0, 5000.0, 10000.0]),
                    &["provider", "type"]
                ).unwrap();
                registry.register(Box::new(ai_tokens_used.clone())).unwrap();

                // Learning system metrics
                let learning_queries_total = Counter::with_opts(
                    Opts::new("orbit_learning_queries_total", "Total learning system queries")
                ).unwrap();
                registry.register(Box::new(learning_queries_total.clone())).unwrap();

                let learning_query_duration = Histogram::with_opts(
                    HistogramOpts::new("orbit_learning_query_duration_seconds", "Learning query duration")
                        .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1])
                ).unwrap();
                registry.register(Box::new(learning_query_duration.clone())).unwrap();

                let learning_patterns_detected = IntGauge::with_opts(
                    Opts::new("orbit_learning_patterns_detected", "Number of detected patterns")
                ).unwrap();
                registry.register(Box::new(learning_patterns_detected.clone())).unwrap();

                // Cache metrics
                let cache_hits = Counter::with_opts(
                    Opts::new("orbit_cache_hits_total", "Total cache hits")
                ).unwrap();
                registry.register(Box::new(cache_hits.clone())).unwrap();

                let cache_misses = Counter::with_opts(
                    Opts::new("orbit_cache_misses_total", "Total cache misses")
                ).unwrap();
                registry.register(Box::new(cache_misses.clone())).unwrap();

                let cache_size = IntGauge::with_opts(
                    Opts::new("orbit_cache_size", "Current cache size")
                ).unwrap();
                registry.register(Box::new(cache_size.clone())).unwrap();

                // Error metrics
                let errors_total = prometheus::CounterVec::new(
                    Opts::new("orbit_errors_total", "Total errors by type"),
                    &["type"]
                ).unwrap();
                registry.register(Box::new(errors_total.clone())).unwrap();

                // Connection metrics
                let active_connections = IntGauge::with_opts(
                    Opts::new("orbit_active_connections", "Number of active IPC connections")
                ).unwrap();
                registry.register(Box::new(active_connections.clone())).unwrap();

                let connection_errors = Counter::with_opts(
                    Opts::new("orbit_connection_errors_total", "Total connection errors")
                ).unwrap();
                registry.register(Box::new(connection_errors.clone())).unwrap();

                // System metrics
                let memory_usage_bytes = IntGauge::with_opts(
                    Opts::new("orbit_memory_usage_bytes", "Memory usage in bytes")
                ).unwrap();
                registry.register(Box::new(memory_usage_bytes.clone())).unwrap();

                let cpu_usage_percent = prometheus::Gauge::with_opts(
                    Opts::new("orbit_cpu_usage_percent", "CPU usage percentage")
                ).unwrap();
                registry.register(Box::new(cpu_usage_percent.clone())).unwrap();

                METRICS = Some(Metrics {
                    registry,
                    commands_total,
                    commands_duration,
                    commands_by_type,
                    ai_queries_total,
                    ai_query_duration,
                    ai_queries_by_provider,
                    ai_tokens_used,
                    learning_queries_total,
                    learning_query_duration,
                    learning_patterns_detected,
                    cache_hits,
                    cache_misses,
                    cache_size,
                    errors_total,
                    active_connections,
                    connection_errors,
                    memory_usage_bytes,
                    cpu_usage_percent,
                });
            });

            METRICS.as_ref().unwrap()
        }
    }

    /// Export metrics in Prometheus text format
    pub fn export(&self) -> String {
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();

        let mut buffer = Vec::new();
        encoder.encode(&metric_families, &mut buffer).unwrap();

        String::from_utf8(buffer).unwrap()
    }

    /// Record command execution
    pub fn record_command(&self, command_type: &str, duration_secs: f64) {
        self.commands_total.inc();
        self.commands_duration.observe(duration_secs);
        self.commands_by_type.with_label_values(&[command_type]).inc();
    }

    /// Record AI query
    pub fn record_ai_query(&self, provider: &str, duration_secs: f64, input_tokens: u32, output_tokens: u32) {
        self.ai_queries_total.inc();
        self.ai_query_duration.observe(duration_secs);
        self.ai_queries_by_provider.with_label_values(&[provider]).inc();
        self.ai_tokens_used.with_label_values(&[provider, "input"]).observe(input_tokens as f64);
        self.ai_tokens_used.with_label_values(&[provider, "output"]).observe(output_tokens as f64);
    }

    /// Record learning query
    pub fn record_learning_query(&self, duration_secs: f64) {
        self.learning_queries_total.inc();
        self.learning_query_duration.observe(duration_secs);
    }

    /// Record cache hit
    pub fn record_cache_hit(&self) {
        self.cache_hits.inc();
    }

    /// Record cache miss
    pub fn record_cache_miss(&self) {
        self.cache_misses.inc();
    }

    /// Update cache size
    pub fn update_cache_size(&self, size: usize) {
        self.cache_size.set(size as i64);
    }

    /// Record error
    pub fn record_error(&self, error_type: &str) {
        self.errors_total.with_label_values(&[error_type]).inc();
    }

    /// Update active connections
    pub fn update_active_connections(&self, count: usize) {
        self.active_connections.set(count as i64);
    }

    /// Record connection error
    pub fn record_connection_error(&self) {
        self.connection_errors.inc();
    }

    /// Update memory usage
    pub fn update_memory_usage(&self, bytes: usize) {
        self.memory_usage_bytes.set(bytes as i64);
    }

    /// Update CPU usage
    pub fn update_cpu_usage(&self, percent: f64) {
        self.cpu_usage_percent.set(percent);
    }

    /// Calculate cache hit rate
    pub fn cache_hit_rate(&self) -> f64 {
        let hits = self.cache_hits.get();
        let misses = self.cache_misses.get();
        let total = hits + misses;

        if total == 0.0 {
            0.0
        } else {
            hits / total
        }
    }
}

/// Get the global metrics instance
pub fn get_metrics() -> &'static Metrics {
    Metrics::init()
}

/// Helper macros for easy metric recording

#[macro_export]
macro_rules! record_command {
    ($cmd_type:expr, $duration:expr) => {
        {
            use $crate::observability::metrics::get_metrics;
            let metrics = get_metrics();
            metrics.record_command($cmd_type, $duration);
        }
    };
}

#[macro_export]
macro_rules! record_ai_query {
    ($provider:expr, $duration:expr, $input_tokens:expr, $output_tokens:expr) => {
        {
            use $crate::observability::metrics::get_metrics;
            let metrics = get_metrics();
            metrics.record_ai_query($provider, $duration, $input_tokens, $output_tokens);
        }
    };
}

#[macro_export]
macro_rules! record_error {
    ($error_type:expr) => {
        {
            use $crate::observability::metrics::get_metrics;
            let metrics = get_metrics();
            metrics.record_error($error_type);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_initialization() {
        let metrics = get_metrics();
        assert_eq!(metrics.commands_total.get(), 0.0);
    }

    #[test]
    fn test_record_command() {
        let metrics = get_metrics();
        let before = metrics.commands_total.get();

        metrics.record_command("known", 0.001);

        assert!(metrics.commands_total.get() > before);
    }

    #[test]
    fn test_cache_hit_rate() {
        let metrics = get_metrics();

        metrics.record_cache_hit();
        metrics.record_cache_hit();
        metrics.record_cache_miss();

        let rate = metrics.cache_hit_rate();
        assert!((rate - 0.666).abs() < 0.01);
    }

    #[test]
    fn test_metrics_export() {
        let metrics = get_metrics();
        metrics.record_command("test", 0.5);

        let exported = metrics.export();
        assert!(exported.contains("orbit_commands_total"));
        assert!(exported.contains("orbit_commands_duration_seconds"));
    }
}
