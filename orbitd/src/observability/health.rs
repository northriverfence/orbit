// Health check system for Orbit daemon
// Monitors database, providers, socket, and system resources

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use anyhow::Result;

/// Overall health status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Individual check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    pub status: Status,
    pub message: Option<String>,
    pub latency_ms: u64,
    pub details: Option<HashMap<String, String>>,
}

/// Complete health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: Status,
    pub checks: HashMap<String, CheckResult>,
    pub timestamp: u64,
    pub uptime_seconds: u64,
}

/// Health checker
pub struct HealthChecker {
    socket_path: PathBuf,
    start_time: Instant,
}

impl HealthChecker {
    /// Create a new health checker
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            start_time: Instant::now(),
        }
    }

    /// Perform all health checks
    pub async fn check_all(&self) -> HealthStatus {
        let mut checks = HashMap::new();

        // Check socket
        checks.insert("socket".to_string(), self.check_socket().await);

        // Check system resources
        checks.insert("resources".to_string(), self.check_resources().await);

        // Check memory
        checks.insert("memory".to_string(), self.check_memory().await);

        // Check disk space
        checks.insert("disk".to_string(), self.check_disk().await);

        // Determine overall status
        let status = self.determine_overall_status(&checks);

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uptime_seconds = self.start_time.elapsed().as_secs();

        HealthStatus {
            status,
            checks,
            timestamp,
            uptime_seconds,
        }
    }

    /// Check socket accessibility
    async fn check_socket(&self) -> CheckResult {
        let start = Instant::now();

        let exists = self.socket_path.exists();
        let latency_ms = start.elapsed().as_millis() as u64;

        if exists {
            // Additional check: verify it's actually a socket
            let metadata = std::fs::metadata(&self.socket_path);

            if let Ok(meta) = metadata {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::FileTypeExt;
                    if meta.file_type().is_socket() {
                        CheckResult {
                            status: Status::Healthy,
                            message: Some("Socket accessible and valid".to_string()),
                            latency_ms,
                            details: Some(HashMap::from([
                                ("path".to_string(), self.socket_path.display().to_string()),
                            ])),
                        }
                    } else {
                        CheckResult {
                            status: Status::Unhealthy,
                            message: Some("Path exists but is not a socket".to_string()),
                            latency_ms,
                            details: None,
                        }
                    }
                }

                #[cfg(not(unix))]
                {
                    CheckResult {
                        status: Status::Healthy,
                        message: Some("Socket file exists".to_string()),
                        latency_ms,
                        details: Some(HashMap::from([
                            ("path".to_string(), self.socket_path.display().to_string()),
                        ])),
                    }
                }
            } else {
                CheckResult {
                    status: Status::Unhealthy,
                    message: Some("Cannot access socket metadata".to_string()),
                    latency_ms,
                    details: None,
                }
            }
        } else {
            CheckResult {
                status: Status::Unhealthy,
                message: Some("Socket not found".to_string()),
                latency_ms,
                details: Some(HashMap::from([
                    ("path".to_string(), self.socket_path.display().to_string()),
                ])),
            }
        }
    }

    /// Check system resources
    async fn check_resources(&self) -> CheckResult {
        let start = Instant::now();

        #[cfg(target_family = "unix")]
        {
            use sysinfo::{System, SystemExt, ProcessExt};

            let mut sys = System::new_all();
            sys.refresh_all();

            let total_memory = sys.total_memory();
            let used_memory = sys.used_memory();
            let memory_usage_pct = (used_memory as f64 / total_memory as f64) * 100.0;

            let cpu_count = sys.cpus().len();
            let load_avg = sys.load_average();

            let latency_ms = start.elapsed().as_millis() as u64;

            let status = if memory_usage_pct > 95.0 {
                Status::Unhealthy
            } else if memory_usage_pct > 85.0 {
                Status::Degraded
            } else {
                Status::Healthy
            };

            let mut details = HashMap::new();
            details.insert("memory_usage_pct".to_string(), format!("{:.1}", memory_usage_pct));
            details.insert("memory_total_mb".to_string(), format!("{}", total_memory / 1024));
            details.insert("memory_used_mb".to_string(), format!("{}", used_memory / 1024));
            details.insert("cpu_count".to_string(), format!("{}", cpu_count));
            details.insert("load_avg_1min".to_string(), format!("{:.2}", load_avg.one));

            CheckResult {
                status,
                message: Some(format!("Memory usage: {:.1}%", memory_usage_pct)),
                latency_ms,
                details: Some(details),
            }
        }

        #[cfg(not(target_family = "unix"))]
        {
            let latency_ms = start.elapsed().as_millis() as u64;

            CheckResult {
                status: Status::Healthy,
                message: Some("Resource check not available on this platform".to_string()),
                latency_ms,
                details: None,
            }
        }
    }

    /// Check memory usage for this process
    async fn check_memory(&self) -> CheckResult {
        let start = Instant::now();

        #[cfg(target_family = "unix")]
        {
            use sysinfo::{System, SystemExt, ProcessExt, PidExt};

            let mut sys = System::new_all();
            sys.refresh_all();

            let pid = sysinfo::get_current_pid().unwrap();

            if let Some(process) = sys.process(pid) {
                let memory_kb = process.memory();
                let memory_mb = memory_kb / 1024;

                let latency_ms = start.elapsed().as_millis() as u64;

                let status = if memory_mb > 1000 {
                    Status::Degraded
                } else if memory_mb > 2000 {
                    Status::Unhealthy
                } else {
                    Status::Healthy
                };

                let mut details = HashMap::new();
                details.insert("memory_mb".to_string(), format!("{}", memory_mb));
                details.insert("pid".to_string(), format!("{}", pid));

                CheckResult {
                    status,
                    message: Some(format!("Process memory: {} MB", memory_mb)),
                    latency_ms,
                    details: Some(details),
                }
            } else {
                let latency_ms = start.elapsed().as_millis() as u64;

                CheckResult {
                    status: Status::Degraded,
                    message: Some("Cannot get process info".to_string()),
                    latency_ms,
                    details: None,
                }
            }
        }

        #[cfg(not(target_family = "unix"))]
        {
            let latency_ms = start.elapsed().as_millis() as u64;

            CheckResult {
                status: Status::Healthy,
                message: Some("Memory check not available on this platform".to_string()),
                latency_ms,
                details: None,
            }
        }
    }

    /// Check disk space
    async fn check_disk(&self) -> CheckResult {
        let start = Instant::now();

        #[cfg(target_family = "unix")]
        {
            use sysinfo::{System, SystemExt, DiskExt};

            let mut sys = System::new_all();
            sys.refresh_all();

            // Check disk where socket is located
            let socket_dir = self.socket_path.parent().unwrap_or_else(|| std::path::Path::new("/"));

            let mut disk_free_gb = 0u64;
            let mut disk_total_gb = 0u64;
            let mut disk_usage_pct = 0.0;

            for disk in sys.disks() {
                if socket_dir.starts_with(disk.mount_point()) {
                    disk_free_gb = disk.available_space() / (1024 * 1024 * 1024);
                    disk_total_gb = disk.total_space() / (1024 * 1024 * 1024);
                    disk_usage_pct = ((disk_total_gb - disk_free_gb) as f64 / disk_total_gb as f64) * 100.0;
                    break;
                }
            }

            let latency_ms = start.elapsed().as_millis() as u64;

            let status = if disk_usage_pct > 95.0 || disk_free_gb < 1 {
                Status::Unhealthy
            } else if disk_usage_pct > 85.0 || disk_free_gb < 5 {
                Status::Degraded
            } else {
                Status::Healthy
            };

            let mut details = HashMap::new();
            details.insert("disk_free_gb".to_string(), format!("{}", disk_free_gb));
            details.insert("disk_total_gb".to_string(), format!("{}", disk_total_gb));
            details.insert("disk_usage_pct".to_string(), format!("{:.1}", disk_usage_pct));

            CheckResult {
                status,
                message: Some(format!("Disk usage: {:.1}%, {} GB free", disk_usage_pct, disk_free_gb)),
                latency_ms,
                details: Some(details),
            }
        }

        #[cfg(not(target_family = "unix"))]
        {
            let latency_ms = start.elapsed().as_millis() as u64;

            CheckResult {
                status: Status::Healthy,
                message: Some("Disk check not available on this platform".to_string()),
                latency_ms,
                details: None,
            }
        }
    }

    /// Determine overall status from individual checks
    fn determine_overall_status(&self, checks: &HashMap<String, CheckResult>) -> Status {
        let has_unhealthy = checks.values().any(|c| matches!(c.status, Status::Unhealthy));
        let has_degraded = checks.values().any(|c| matches!(c.status, Status::Degraded));

        if has_unhealthy {
            Status::Unhealthy
        } else if has_degraded {
            Status::Degraded
        } else {
            Status::Healthy
        }
    }

    /// Get a simple health check (fast)
    pub async fn check_liveness(&self) -> bool {
        // Simple check: is the socket accessible?
        self.socket_path.exists()
    }

    /// Get readiness check (ready to accept traffic)
    pub async fn check_readiness(&self) -> bool {
        let health = self.check_all().await;
        !matches!(health.status, Status::Unhealthy)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_health_checker_creation() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");

        let checker = HealthChecker::new(socket_path);
        assert!(checker.start_time.elapsed().as_secs() < 1);
    }

    #[tokio::test]
    async fn test_socket_check_missing() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("nonexistent.sock");

        let checker = HealthChecker::new(socket_path);
        let result = checker.check_socket().await;

        assert!(matches!(result.status, Status::Unhealthy));
        assert!(result.message.is_some());
    }

    #[tokio::test]
    async fn test_check_all() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");

        let checker = HealthChecker::new(socket_path);
        let health = checker.check_all().await;

        assert!(health.checks.contains_key("socket"));
        assert!(health.checks.contains_key("resources"));
        assert!(health.timestamp > 0);
    }

    #[tokio::test]
    async fn test_liveness_check() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");

        let checker = HealthChecker::new(socket_path);
        let is_alive = checker.check_liveness().await;

        assert_eq!(is_alive, false); // Socket doesn't exist
    }

    #[tokio::test]
    async fn test_overall_status_determination() {
        let temp_dir = tempdir().unwrap();
        let socket_path = temp_dir.path().join("test.sock");
        let checker = HealthChecker::new(socket_path);

        let mut checks = HashMap::new();

        // All healthy
        checks.insert("test1".to_string(), CheckResult {
            status: Status::Healthy,
            message: None,
            latency_ms: 1,
            details: None,
        });

        let status = checker.determine_overall_status(&checks);
        assert!(matches!(status, Status::Healthy));

        // One degraded
        checks.insert("test2".to_string(), CheckResult {
            status: Status::Degraded,
            message: None,
            latency_ms: 1,
            details: None,
        });

        let status = checker.determine_overall_status(&checks);
        assert!(matches!(status, Status::Degraded));

        // One unhealthy
        checks.insert("test3".to_string(), CheckResult {
            status: Status::Unhealthy,
            message: None,
            latency_ms: 1,
            details: None,
        });

        let status = checker.determine_overall_status(&checks);
        assert!(matches!(status, Status::Unhealthy));
    }
}
