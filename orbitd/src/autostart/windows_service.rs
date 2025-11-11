// Windows Service implementation

use super::DaemonStatus;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

const SERVICE_NAME: &str = "OrbitDaemon";
const DISPLAY_NAME: &str = "Orbit AI Terminal Daemon";

/// Check if Windows service is installed
pub fn windows_service_is_installed() -> Result<bool> {
    let output = Command::new("sc")
        .args(["query", SERVICE_NAME])
        .output()
        .context("Failed to query Windows service")?;

    Ok(output.status.success())
}

/// Check if Windows service is enabled (auto-start)
pub fn windows_service_is_enabled() -> Result<bool> {
    if !windows_service_is_installed()? {
        return Ok(false);
    }

    let output = Command::new("sc")
        .args(["qc", SERVICE_NAME])
        .output()
        .context("Failed to query Windows service config")?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Check if START_TYPE is AUTO_START
        Ok(stdout.contains("AUTO_START"))
    } else {
        Ok(false)
    }
}

/// Install Windows service
pub fn windows_service_install(daemon_path: &Path) -> Result<()> {
    let daemon_path_str = daemon_path
        .to_str()
        .context("Invalid daemon path")?;

    let output = Command::new("sc")
        .args([
            "create",
            SERVICE_NAME,
            "binPath=",
            &format!("\"{}\"", daemon_path_str),
            "DisplayName=",
            &format!("\"{}\"", DISPLAY_NAME),
            "start=",
            "demand", // Don't auto-start on install
        ])
        .output()
        .context("Failed to create Windows service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to create Windows service: {}", stderr);
    }

    // Set description
    let _ = Command::new("sc")
        .args([
            "description",
            SERVICE_NAME,
            "Orbit AI Terminal Daemon - provides terminal intelligence and session management",
        ])
        .output();

    tracing::info!("Installed Windows service: {}", SERVICE_NAME);
    Ok(())
}

/// Uninstall Windows service
pub fn windows_service_uninstall() -> Result<()> {
    // Stop first
    let _ = windows_service_stop();

    let output = Command::new("sc")
        .args(["delete", SERVICE_NAME])
        .output()
        .context("Failed to delete Windows service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if service doesn't exist
        if !stderr.contains("does not exist") {
            anyhow::bail!("Failed to delete Windows service: {}", stderr);
        }
    }

    Ok(())
}

/// Enable Windows service (set to auto-start)
pub fn windows_service_enable() -> Result<()> {
    let output = Command::new("sc")
        .args(["config", SERVICE_NAME, "start=", "auto"])
        .output()
        .context("Failed to configure Windows service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to enable Windows service: {}", stderr);
    }

    Ok(())
}

/// Disable Windows service (set to manual start)
pub fn windows_service_disable() -> Result<()> {
    let output = Command::new("sc")
        .args(["config", SERVICE_NAME, "start=", "demand"])
        .output()
        .context("Failed to configure Windows service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to disable Windows service: {}", stderr);
    }

    Ok(())
}

/// Start Windows service
pub fn windows_service_start() -> Result<()> {
    let output = Command::new("sc")
        .args(["start", SERVICE_NAME])
        .output()
        .context("Failed to start Windows service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if already started
        if !stderr.contains("already been started") {
            anyhow::bail!("Failed to start Windows service: {}", stderr);
        }
    }

    Ok(())
}

/// Stop Windows service
pub fn windows_service_stop() -> Result<()> {
    let output = Command::new("sc")
        .args(["stop", SERVICE_NAME])
        .output()
        .context("Failed to stop Windows service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if not running
        if !stderr.contains("not started") {
            anyhow::bail!("Failed to stop Windows service: {}", stderr);
        }
    }

    Ok(())
}

/// Get Windows service status
pub fn windows_service_status() -> Result<DaemonStatus> {
    let installed = windows_service_is_installed()?;
    let enabled = if installed {
        windows_service_is_enabled().unwrap_or(false)
    } else {
        false
    };

    let running = if installed {
        let output = Command::new("sc")
            .args(["query", SERVICE_NAME])
            .output()
            .context("Failed to query Windows service")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("RUNNING")
        } else {
            false
        }
    } else {
        false
    };

    Ok(DaemonStatus {
        installed,
        enabled,
        running,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_constants() {
        assert_eq!(SERVICE_NAME, "OrbitDaemon");
        assert_eq!(DISPLAY_NAME, "Orbit AI Terminal Daemon");
    }
}
