//! Application settings management
//!
//! This module provides a comprehensive settings system with:
//! - Type-safe settings structures
//! - Persistent storage (TOML format)
//! - Default values and validation
//! - Atomic writes for safety
//! - Hot-reload support

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

mod storage;
pub use storage::SettingsStorage;

/// Complete application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub appearance: AppearanceSettings,
    pub connection: ConnectionSettings,
    pub security: SecuritySettings,
    pub shortcuts: KeyboardShortcuts,
    pub general: GeneralSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            appearance: AppearanceSettings::default(),
            connection: ConnectionSettings::default(),
            security: SecuritySettings::default(),
            shortcuts: KeyboardShortcuts::default(),
            general: GeneralSettings::default(),
        }
    }
}

/// Appearance and UI settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppearanceSettings {
    /// Theme mode: "light", "dark", or "system"
    pub theme: String,

    /// Terminal font family
    pub font_family: String,

    /// Font size in pixels (12-24)
    pub font_size: u8,

    /// Line height multiplier (1.0-2.0)
    pub line_height: f32,

    /// Cursor style: "block", "beam", or "underline"
    pub cursor_style: String,

    /// Enable cursor blinking
    pub cursor_blink: bool,

    /// Number of scrollback lines (1000-50000)
    pub scrollback_lines: u32,

    /// Terminal color scheme name
    pub color_scheme: String,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            font_family: "Menlo".to_string(),
            font_size: 14,
            line_height: 1.2,
            cursor_style: "block".to_string(),
            cursor_blink: true,
            scrollback_lines: 10000,
            color_scheme: "default".to_string(),
        }
    }
}

impl AppearanceSettings {
    pub fn validate(&self) -> Result<()> {
        // Validate font size
        if !(12..=24).contains(&self.font_size) {
            anyhow::bail!("Font size must be between 12 and 24");
        }

        // Validate line height
        if !(1.0..=2.0).contains(&self.line_height) {
            anyhow::bail!("Line height must be between 1.0 and 2.0");
        }

        // Validate scrollback lines
        if !(1000..=50000).contains(&self.scrollback_lines) {
            anyhow::bail!("Scrollback lines must be between 1000 and 50000");
        }

        // Validate theme
        match self.theme.as_str() {
            "light" | "dark" | "system" => Ok(()),
            _ => anyhow::bail!("Theme must be 'light', 'dark', or 'system'"),
        }
    }
}

/// SSH connection settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ConnectionSettings {
    /// Default SSH port
    pub default_port: u16,

    /// Default SSH username
    pub default_username: String,

    /// Connection timeout in seconds
    pub connect_timeout: u64,

    /// Keepalive interval in seconds (0 = disabled)
    pub keepalive_interval: u64,

    /// Automatically reconnect on connection loss
    pub auto_reconnect: bool,

    /// Maximum reconnect attempts
    pub max_reconnect_attempts: u32,
}

impl Default for ConnectionSettings {
    fn default() -> Self {
        Self {
            default_port: 22,
            default_username: std::env::var("USER").unwrap_or_else(|_| "user".to_string()),
            connect_timeout: 30,
            keepalive_interval: 60,
            auto_reconnect: true,
            max_reconnect_attempts: 3,
        }
    }
}

impl ConnectionSettings {
    pub fn validate(&self) -> Result<()> {
        if self.default_port == 0 {
            anyhow::bail!("Port must be greater than 0");
        }

        if self.connect_timeout < 5 || self.connect_timeout > 300 {
            anyhow::bail!("Connection timeout must be between 5 and 300 seconds");
        }

        if self.keepalive_interval > 600 {
            anyhow::bail!("Keepalive interval must be <= 600 seconds");
        }

        if self.max_reconnect_attempts == 0 || self.max_reconnect_attempts > 10 {
            anyhow::bail!("Max reconnect attempts must be between 1 and 10");
        }

        Ok(())
    }
}

/// Security and privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SecuritySettings {
    /// Accept unknown SSH host keys automatically (INSECURE)
    pub accept_unknown_hosts: bool,

    /// Accept changed SSH host keys automatically (VERY INSECURE)
    pub accept_changed_hosts: bool,

    /// Save passwords in vault
    pub save_passwords: bool,

    /// Auto-lock vault timeout in minutes (0 = never)
    pub auto_lock_vault_timeout: u64,

    /// Require confirmation before executing dangerous commands
    pub require_confirmation_dangerous: bool,

    /// Enable notifications
    pub enable_notifications: bool,

    /// Notify on session disconnect
    pub notify_session_disconnect: bool,

    /// Notify on file transfer complete
    pub notify_file_transfer_complete: bool,

    /// Command completion notification threshold in seconds (0 = never)
    pub notify_command_threshold: u64,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        Self {
            accept_unknown_hosts: false,
            accept_changed_hosts: false,
            save_passwords: true,
            auto_lock_vault_timeout: 15,
            require_confirmation_dangerous: true,
            enable_notifications: true,
            notify_session_disconnect: true,
            notify_file_transfer_complete: true,
            notify_command_threshold: 30,
        }
    }
}

/// Keyboard shortcuts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyboardShortcuts {
    pub new_tab: String,
    pub close_tab: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub split_horizontal: String,
    pub split_vertical: String,
    pub toggle_vault: String,
    pub open_settings: String,
    pub open_file_transfer: String,
    pub open_workspace: String,
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        #[cfg(target_os = "macos")]
        {
            Self {
                new_tab: "Cmd+T".to_string(),
                close_tab: "Cmd+W".to_string(),
                next_tab: "Cmd+Tab".to_string(),
                prev_tab: "Cmd+Shift+Tab".to_string(),
                split_horizontal: "Cmd+Shift+H".to_string(),
                split_vertical: "Cmd+Shift+V".to_string(),
                toggle_vault: "Cmd+Shift+K".to_string(),
                open_settings: "Cmd+,".to_string(),
                open_file_transfer: "Cmd+Shift+F".to_string(),
                open_workspace: "Cmd+Shift+W".to_string(),
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            Self {
                new_tab: "Ctrl+T".to_string(),
                close_tab: "Ctrl+W".to_string(),
                next_tab: "Ctrl+Tab".to_string(),
                prev_tab: "Ctrl+Shift+Tab".to_string(),
                split_horizontal: "Ctrl+Shift+H".to_string(),
                split_vertical: "Ctrl+Shift+V".to_string(),
                toggle_vault: "Ctrl+Shift+K".to_string(),
                open_settings: "Ctrl+,".to_string(),
                open_file_transfer: "Ctrl+Shift+F".to_string(),
                open_workspace: "Ctrl+Shift+W".to_string(),
            }
        }
    }
}

/// General application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralSettings {
    /// Check for updates automatically
    pub check_for_updates: bool,

    /// Send anonymous usage analytics
    pub send_analytics: bool,

    /// Restore sessions on startup
    pub restore_sessions_on_startup: bool,

    /// Confirm before exiting application
    pub confirm_before_exit: bool,

    /// Start daemon on app launch
    pub auto_start_daemon: bool,
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            check_for_updates: true,
            send_analytics: false,
            restore_sessions_on_startup: true,
            confirm_before_exit: true,
            auto_start_daemon: true,
        }
    }
}

/// Settings manager with persistence
pub struct SettingsManager {
    settings: Arc<RwLock<AppSettings>>,
    storage: SettingsStorage,
}

impl SettingsManager {
    /// Create a new settings manager
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        let storage = SettingsStorage::new(config_dir)?;

        // Load settings or use defaults
        let settings = match storage.load() {
            Ok(s) => {
                info!("Loaded settings from disk");
                s
            }
            Err(e) => {
                warn!("Failed to load settings, using defaults: {}", e);
                let defaults = AppSettings::default();
                // Try to save defaults
                if let Err(e) = storage.save(&defaults) {
                    warn!("Failed to save default settings: {}", e);
                }
                defaults
            }
        };

        Ok(Self {
            settings: Arc::new(RwLock::new(settings)),
            storage,
        })
    }

    /// Get all settings
    pub async fn get_all(&self) -> AppSettings {
        self.settings.read().await.clone()
    }

    /// Get appearance settings
    pub async fn get_appearance(&self) -> AppearanceSettings {
        self.settings.read().await.appearance.clone()
    }

    /// Get connection settings
    pub async fn get_connection(&self) -> ConnectionSettings {
        self.settings.read().await.connection.clone()
    }

    /// Get security settings
    pub async fn get_security(&self) -> SecuritySettings {
        self.settings.read().await.security.clone()
    }

    /// Get keyboard shortcuts
    pub async fn get_shortcuts(&self) -> KeyboardShortcuts {
        self.settings.read().await.shortcuts.clone()
    }

    /// Get general settings
    pub async fn get_general(&self) -> GeneralSettings {
        self.settings.read().await.general.clone()
    }

    /// Update appearance settings
    pub async fn update_appearance(&self, appearance: AppearanceSettings) -> Result<()> {
        appearance.validate().context("Invalid appearance settings")?;

        let mut settings = self.settings.write().await;
        settings.appearance = appearance;
        self.storage.save(&*settings)?;

        debug!("Updated appearance settings");
        Ok(())
    }

    /// Update connection settings
    pub async fn update_connection(&self, connection: ConnectionSettings) -> Result<()> {
        connection.validate().context("Invalid connection settings")?;

        let mut settings = self.settings.write().await;
        settings.connection = connection;
        self.storage.save(&*settings)?;

        debug!("Updated connection settings");
        Ok(())
    }

    /// Update security settings
    pub async fn update_security(&self, security: SecuritySettings) -> Result<()> {
        let mut settings = self.settings.write().await;
        settings.security = security;
        self.storage.save(&*settings)?;

        debug!("Updated security settings");
        Ok(())
    }

    /// Update keyboard shortcuts
    pub async fn update_shortcuts(&self, shortcuts: KeyboardShortcuts) -> Result<()> {
        let mut settings = self.settings.write().await;
        settings.shortcuts = shortcuts;
        self.storage.save(&*settings)?;

        debug!("Updated keyboard shortcuts");
        Ok(())
    }

    /// Update general settings
    pub async fn update_general(&self, general: GeneralSettings) -> Result<()> {
        let mut settings = self.settings.write().await;
        settings.general = general;
        self.storage.save(&*settings)?;

        debug!("Updated general settings");
        Ok(())
    }

    /// Reset to default settings
    pub async fn reset_to_defaults(&self) -> Result<()> {
        let defaults = AppSettings::default();
        let mut settings = self.settings.write().await;
        *settings = defaults.clone();
        self.storage.save(&defaults)?;

        info!("Reset settings to defaults");
        Ok(())
    }

    /// Export settings to a file
    pub async fn export(&self, path: PathBuf) -> Result<()> {
        let settings = self.settings.read().await;
        self.storage.export(&*settings, path)?;

        info!("Exported settings");
        Ok(())
    }

    /// Import settings from a file
    pub async fn import(&self, path: PathBuf) -> Result<AppSettings> {
        let imported = self.storage.import(path)?;

        // Validate imported settings
        if let Err(e) = imported.appearance.validate() {
            warn!("Invalid appearance settings in import: {}", e);
        }
        if let Err(e) = imported.connection.validate() {
            warn!("Invalid connection settings in import: {}", e);
        }

        // Update current settings
        let mut settings = self.settings.write().await;
        *settings = imported.clone();
        self.storage.save(&imported)?;

        info!("Imported settings");
        Ok(imported)
    }
}
