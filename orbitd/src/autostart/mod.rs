// Auto-start management for orbitd daemon
//
// This module provides cross-platform daemon auto-start functionality:
// - Linux: systemd user service
// - macOS: launchd plist
// - Windows: Windows Service

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

#[cfg(target_os = "linux")]
mod systemd;
#[cfg(target_os = "linux")]
pub use systemd::*;

#[cfg(target_os = "macos")]
mod launchd;
#[cfg(target_os = "macos")]
pub use launchd::*;

#[cfg(target_os = "windows")]
mod windows_service;
#[cfg(target_os = "windows")]
pub use windows_service::*;

/// Auto-start manager for daemon
pub struct AutoStartManager {
    daemon_path: PathBuf,
}

impl AutoStartManager {
    /// Create a new auto-start manager
    ///
    /// # Arguments
    /// * `daemon_path` - Path to orbitd executable
    pub fn new<P: AsRef<Path>>(daemon_path: P) -> Self {
        Self {
            daemon_path: daemon_path.as_ref().to_path_buf(),
        }
    }

    /// Check if auto-start is installed
    pub fn is_installed(&self) -> Result<bool> {
        #[cfg(target_os = "linux")]
        return systemd_is_installed();

        #[cfg(target_os = "macos")]
        return launchd_is_installed();

        #[cfg(target_os = "windows")]
        return windows_service_is_installed();

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        Ok(false)
    }

    /// Check if auto-start is enabled
    pub fn is_enabled(&self) -> Result<bool> {
        #[cfg(target_os = "linux")]
        return systemd_is_enabled();

        #[cfg(target_os = "macos")]
        return launchd_is_enabled();

        #[cfg(target_os = "windows")]
        return windows_service_is_enabled();

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        Ok(false)
    }

    /// Install auto-start
    pub fn install(&self) -> Result<()> {
        if self.is_installed()? {
            tracing::info!("Auto-start already installed");
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            systemd_install(&self.daemon_path)?;
            tracing::info!("Installed systemd user service");
        }

        #[cfg(target_os = "macos")]
        {
            launchd_install(&self.daemon_path)?;
            tracing::info!("Installed launchd plist");
        }

        #[cfg(target_os = "windows")]
        {
            windows_service_install(&self.daemon_path)?;
            tracing::info!("Installed Windows service");
        }

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            anyhow::bail!("Auto-start not supported on this platform");
        }

        Ok(())
    }

    /// Uninstall auto-start
    pub fn uninstall(&self) -> Result<()> {
        if !self.is_installed()? {
            tracing::info!("Auto-start not installed");
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            systemd_uninstall()?;
            tracing::info!("Uninstalled systemd user service");
        }

        #[cfg(target_os = "macos")]
        {
            launchd_uninstall()?;
            tracing::info!("Uninstalled launchd plist");
        }

        #[cfg(target_os = "windows")]
        {
            windows_service_uninstall()?;
            tracing::info!("Uninstalled Windows service");
        }

        Ok(())
    }

    /// Enable auto-start
    pub fn enable(&self) -> Result<()> {
        if !self.is_installed()? {
            self.install()?;
        }

        #[cfg(target_os = "linux")]
        {
            systemd_enable()?;
            tracing::info!("Enabled systemd user service");
        }

        #[cfg(target_os = "macos")]
        {
            launchd_enable()?;
            tracing::info!("Enabled launchd service");
        }

        #[cfg(target_os = "windows")]
        {
            windows_service_enable()?;
            tracing::info!("Enabled Windows service");
        }

        Ok(())
    }

    /// Disable auto-start
    pub fn disable(&self) -> Result<()> {
        if !self.is_enabled()? {
            tracing::info!("Auto-start not enabled");
            return Ok(());
        }

        #[cfg(target_os = "linux")]
        {
            systemd_disable()?;
            tracing::info!("Disabled systemd user service");
        }

        #[cfg(target_os = "macos")]
        {
            launchd_disable()?;
            tracing::info!("Disabled launchd service");
        }

        #[cfg(target_os = "windows")]
        {
            windows_service_disable()?;
            tracing::info!("Disabled Windows service");
        }

        Ok(())
    }

    /// Start the daemon service
    pub fn start(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            systemd_start()?;
            tracing::info!("Started systemd user service");
        }

        #[cfg(target_os = "macos")]
        {
            launchd_start()?;
            tracing::info!("Started launchd service");
        }

        #[cfg(target_os = "windows")]
        {
            windows_service_start()?;
            tracing::info!("Started Windows service");
        }

        Ok(())
    }

    /// Stop the daemon service
    pub fn stop(&self) -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            systemd_stop()?;
            tracing::info!("Stopped systemd user service");
        }

        #[cfg(target_os = "macos")]
        {
            launchd_stop()?;
            tracing::info!("Stopped launchd service");
        }

        #[cfg(target_os = "windows")]
        {
            windows_service_stop()?;
            tracing::info!("Stopped Windows service");
        }

        Ok(())
    }

    /// Get daemon status
    pub fn status(&self) -> Result<DaemonStatus> {
        #[cfg(target_os = "linux")]
        return systemd_status();

        #[cfg(target_os = "macos")]
        return launchd_status();

        #[cfg(target_os = "windows")]
        return windows_service_status();

        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        Ok(DaemonStatus {
            installed: false,
            enabled: false,
            running: false,
        })
    }
}

/// Daemon status
#[derive(Debug, Clone)]
pub struct DaemonStatus {
    pub installed: bool,
    pub enabled: bool,
    pub running: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_autostart_manager_creation() {
        let manager = AutoStartManager::new("/usr/local/bin/orbitd");
        assert_eq!(manager.daemon_path, PathBuf::from("/usr/local/bin/orbitd"));
    }

    #[test]
    fn test_status_check() {
        let manager = AutoStartManager::new("/usr/local/bin/orbitd");
        // This should not panic
        let _ = manager.status();
    }
}
