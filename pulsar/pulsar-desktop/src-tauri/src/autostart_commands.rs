// Tauri commands for managing daemon auto-start

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result type for commands
type CommandResult<T> = Result<T, String>;

/// Auto-start manager state
pub struct AutoStartState {
    daemon_path: Arc<RwLock<Option<PathBuf>>>,
}

impl AutoStartState {
    pub fn new() -> Self {
        Self {
            daemon_path: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set_daemon_path(&self, path: PathBuf) {
        let mut daemon_path = self.daemon_path.write().await;
        *daemon_path = Some(path);
    }

    pub async fn get_daemon_path(&self) -> Option<PathBuf> {
        self.daemon_path.read().await.clone()
    }
}

/// Daemon status for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatusDto {
    pub installed: bool,
    pub enabled: bool,
    pub running: bool,
}

/// Set the daemon path for auto-start management
#[tauri::command]
pub async fn autostart_set_daemon_path(
    state: State<'_, AutoStartState>,
    path: String,
) -> CommandResult<()> {
    let path_buf = PathBuf::from(path);
    state.set_daemon_path(path_buf).await;
    Ok(())
}

/// Check if auto-start is installed
#[tauri::command]
pub async fn autostart_is_installed(
    state: State<'_, AutoStartState>,
) -> CommandResult<bool> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.is_installed().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Ok(false)
}

/// Check if auto-start is enabled
#[tauri::command]
pub async fn autostart_is_enabled(
    state: State<'_, AutoStartState>,
) -> CommandResult<bool> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.is_enabled().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Ok(false)
}

/// Install auto-start
#[tauri::command]
pub async fn autostart_install(
    state: State<'_, AutoStartState>,
) -> CommandResult<()> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.install().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Auto-start not supported on this platform".to_string())
}

/// Uninstall auto-start
#[tauri::command]
pub async fn autostart_uninstall(
    state: State<'_, AutoStartState>,
) -> CommandResult<()> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.uninstall().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Auto-start not supported on this platform".to_string())
}

/// Enable auto-start
#[tauri::command]
pub async fn autostart_enable(
    state: State<'_, AutoStartState>,
) -> CommandResult<()> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.enable().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Auto-start not supported on this platform".to_string())
}

/// Disable auto-start
#[tauri::command]
pub async fn autostart_disable(
    state: State<'_, AutoStartState>,
) -> CommandResult<()> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.disable().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Auto-start not supported on this platform".to_string())
}

/// Start the daemon service
#[tauri::command]
pub async fn autostart_start(
    state: State<'_, AutoStartState>,
) -> CommandResult<()> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.start().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Auto-start not supported on this platform".to_string())
}

/// Stop the daemon service
#[tauri::command]
pub async fn autostart_stop(
    state: State<'_, AutoStartState>,
) -> CommandResult<()> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        manager.stop().map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Err("Auto-start not supported on this platform".to_string())
}

/// Get daemon status
#[tauri::command]
pub async fn autostart_get_status(
    state: State<'_, AutoStartState>,
) -> CommandResult<DaemonStatusDto> {
    let daemon_path = state.get_daemon_path().await
        .ok_or_else(|| "Daemon path not set".to_string())?;

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    {
        use orbitd::autostart::AutoStartManager;
        let manager = AutoStartManager::new(daemon_path);
        let status = manager.status().map_err(|e| e.to_string())?;
        Ok(DaemonStatusDto {
            installed: status.installed,
            enabled: status.enabled,
            running: status.running,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    Ok(DaemonStatusDto {
        installed: false,
        enabled: false,
        running: false,
    })
}
