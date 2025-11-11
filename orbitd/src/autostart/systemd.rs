// systemd user service implementation for Linux

use super::DaemonStatus;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SERVICE_NAME: &str = "orbitd.service";

/// Get systemd user service directory
fn get_service_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?;
    Ok(config_dir.join("systemd").join("user"))
}

/// Get service file path
fn get_service_path() -> Result<PathBuf> {
    Ok(get_service_dir()?.join(SERVICE_NAME))
}

/// Generate systemd service file content
fn generate_service_file(daemon_path: &Path) -> String {
    let home = dirs::home_dir()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "~".to_string());

    format!(
        r#"[Unit]
Description=Orbit AI Terminal Daemon
Documentation=https://github.com/singulio/orbit
After=network.target

[Service]
Type=simple
ExecStart={}
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths={}/.config/orbit {}/.local/share/orbit

# Resource limits
LimitNOFILE=65536
MemoryMax=512M

# Environment
Environment="RUST_LOG=info"
Environment="ORBIT_CONFIG_DIR={}/.config/orbit"
Environment="ORBIT_DATA_DIR={}/.local/share/orbit"

[Install]
WantedBy=default.target
"#,
        daemon_path.display(),
        home,
        home,
        home,
        home
    )
}

/// Check if systemd service is installed
pub fn systemd_is_installed() -> Result<bool> {
    let service_path = get_service_path()?;
    Ok(service_path.exists())
}

/// Check if systemd service is enabled
pub fn systemd_is_enabled() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["--user", "is-enabled", SERVICE_NAME])
        .output()
        .context("Failed to check systemd service status")?;

    Ok(output.status.success())
}

/// Install systemd service
pub fn systemd_install(daemon_path: &Path) -> Result<()> {
    // Create service directory if it doesn't exist
    let service_dir = get_service_dir()?;
    fs::create_dir_all(&service_dir)
        .context("Failed to create systemd user directory")?;

    // Write service file
    let service_path = get_service_path()?;
    let service_content = generate_service_file(daemon_path);
    fs::write(&service_path, service_content)
        .context("Failed to write systemd service file")?;

    // Reload systemd
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .context("Failed to reload systemd daemon")?;

    tracing::info!("Installed systemd service at {}", service_path.display());
    Ok(())
}

/// Uninstall systemd service
pub fn systemd_uninstall() -> Result<()> {
    // Stop and disable first
    let _ = systemd_stop();
    let _ = systemd_disable();

    // Remove service file
    let service_path = get_service_path()?;
    if service_path.exists() {
        fs::remove_file(&service_path)
            .context("Failed to remove systemd service file")?;
    }

    // Reload systemd
    Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output()
        .context("Failed to reload systemd daemon")?;

    Ok(())
}

/// Enable systemd service
pub fn systemd_enable() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "enable", SERVICE_NAME])
        .output()
        .context("Failed to enable systemd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to enable systemd service: {}", stderr);
    }

    Ok(())
}

/// Disable systemd service
pub fn systemd_disable() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "disable", SERVICE_NAME])
        .output()
        .context("Failed to disable systemd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to disable systemd service: {}", stderr);
    }

    Ok(())
}

/// Start systemd service
pub fn systemd_start() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "start", SERVICE_NAME])
        .output()
        .context("Failed to start systemd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to start systemd service: {}", stderr);
    }

    Ok(())
}

/// Stop systemd service
pub fn systemd_stop() -> Result<()> {
    let output = Command::new("systemctl")
        .args(["--user", "stop", SERVICE_NAME])
        .output()
        .context("Failed to stop systemd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to stop systemd service: {}", stderr);
    }

    Ok(())
}

/// Get systemd service status
pub fn systemd_status() -> Result<DaemonStatus> {
    let installed = systemd_is_installed()?;
    let enabled = if installed {
        systemd_is_enabled().unwrap_or(false)
    } else {
        false
    };

    let running = if installed {
        let output = Command::new("systemctl")
            .args(["--user", "is-active", SERVICE_NAME])
            .output()
            .context("Failed to check systemd service status")?;

        output.status.success()
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
    fn test_service_file_generation() {
        let daemon_path = Path::new("/usr/local/bin/orbitd");
        let content = generate_service_file(daemon_path);

        assert!(content.contains("ExecStart=/usr/local/bin/orbitd"));
        assert!(content.contains("Restart=on-failure"));
        assert!(content.contains("WantedBy=default.target"));
    }

    #[test]
    fn test_service_dir() {
        let dir = get_service_dir();
        assert!(dir.is_ok());
    }
}
