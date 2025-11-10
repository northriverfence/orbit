//! Settings persistence layer
//!
//! Handles reading and writing settings to/from disk in TOML format.
//! Uses atomic writes to prevent corruption.

use super::AppSettings;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tracing::{debug, info};

/// Settings storage manager
pub struct SettingsStorage {
    settings_path: PathBuf,
}

impl SettingsStorage {
    /// Create a new settings storage
    pub fn new(config_dir: PathBuf) -> Result<Self> {
        // Ensure config directory exists
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;

        let settings_path = config_dir.join("pulsar_settings.toml");

        Ok(Self { settings_path })
    }

    /// Load settings from disk
    pub fn load(&self) -> Result<AppSettings> {
        if !self.settings_path.exists() {
            debug!("Settings file does not exist, using defaults");
            return Ok(AppSettings::default());
        }

        let contents = fs::read_to_string(&self.settings_path)
            .context("Failed to read settings file")?;

        let settings: AppSettings = toml::from_str(&contents)
            .context("Failed to parse settings file")?;

        debug!("Loaded settings from {:?}", self.settings_path);
        Ok(settings)
    }

    /// Save settings to disk (atomic write)
    pub fn save(&self, settings: &AppSettings) -> Result<()> {
        // Serialize to TOML
        let toml_string = toml::to_string_pretty(settings)
            .context("Failed to serialize settings")?;

        // Write to temporary file first
        let temp_path = self.settings_path.with_extension("toml.tmp");
        fs::write(&temp_path, toml_string)
            .context("Failed to write temporary settings file")?;

        // Atomic rename (overwrites existing file)
        fs::rename(&temp_path, &self.settings_path)
            .context("Failed to rename settings file")?;

        debug!("Saved settings to {:?}", self.settings_path);
        Ok(())
    }

    /// Export settings to a specific path
    pub fn export(&self, settings: &AppSettings, path: PathBuf) -> Result<()> {
        let toml_string = toml::to_string_pretty(settings)
            .context("Failed to serialize settings for export")?;

        fs::write(&path, toml_string)
            .context("Failed to write exported settings")?;

        info!("Exported settings to {:?}", path);
        Ok(())
    }

    /// Import settings from a specific path
    pub fn import(&self, path: PathBuf) -> Result<AppSettings> {
        let contents = fs::read_to_string(&path)
            .context("Failed to read settings file for import")?;

        let settings: AppSettings = toml::from_str(&contents)
            .context("Failed to parse imported settings file")?;

        info!("Imported settings from {:?}", path);
        Ok(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SettingsStorage::new(temp_dir.path().to_path_buf()).unwrap();

        // Create settings with custom values
        let mut settings = AppSettings::default();
        settings.appearance.font_size = 16;
        settings.connection.default_port = 2222;

        // Save
        storage.save(&settings).unwrap();

        // Load
        let loaded = storage.load().unwrap();

        assert_eq!(loaded.appearance.font_size, 16);
        assert_eq!(loaded.connection.default_port, 2222);
    }

    #[test]
    fn test_storage_export_and_import() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SettingsStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let mut settings = AppSettings::default();
        settings.appearance.theme = "dark".to_string();

        // Export
        let export_path = temp_dir.path().join("exported.toml");
        storage.export(&settings, export_path.clone()).unwrap();

        // Import
        let imported = storage.import(export_path).unwrap();

        assert_eq!(imported.appearance.theme, "dark");
    }

    #[test]
    fn test_load_nonexistent_uses_defaults() {
        let temp_dir = TempDir::new().unwrap();
        let storage = SettingsStorage::new(temp_dir.path().to_path_buf()).unwrap();

        let loaded = storage.load().unwrap();

        // Should get defaults
        assert_eq!(loaded.appearance.font_size, 14);
        assert_eq!(loaded.connection.default_port, 22);
    }
}
