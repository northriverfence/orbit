//! Tauri commands for settings management

use crate::settings::*;
use std::path::PathBuf;
use tauri::State;

type CommandResult<T> = Result<T, String>;

/// Get all settings
#[tauri::command]
pub async fn settings_get_all(
    settings: State<'_, SettingsManager>,
) -> CommandResult<AppSettings> {
    Ok(settings.get_all().await)
}

/// Get appearance settings
#[tauri::command]
pub async fn settings_get_appearance(
    settings: State<'_, SettingsManager>,
) -> CommandResult<AppearanceSettings> {
    Ok(settings.get_appearance().await)
}

/// Get connection settings
#[tauri::command]
pub async fn settings_get_connection(
    settings: State<'_, SettingsManager>,
) -> CommandResult<ConnectionSettings> {
    Ok(settings.get_connection().await)
}

/// Get security settings
#[tauri::command]
pub async fn settings_get_security(
    settings: State<'_, SettingsManager>,
) -> CommandResult<SecuritySettings> {
    Ok(settings.get_security().await)
}

/// Get keyboard shortcuts
#[tauri::command]
pub async fn settings_get_shortcuts(
    settings: State<'_, SettingsManager>,
) -> CommandResult<KeyboardShortcuts> {
    Ok(settings.get_shortcuts().await)
}

/// Get general settings
#[tauri::command]
pub async fn settings_get_general(
    settings: State<'_, SettingsManager>,
) -> CommandResult<GeneralSettings> {
    Ok(settings.get_general().await)
}

/// Update appearance settings
#[tauri::command]
pub async fn settings_update_appearance(
    settings: State<'_, SettingsManager>,
    appearance: AppearanceSettings,
) -> CommandResult<()> {
    settings
        .update_appearance(appearance)
        .await
        .map_err(|e| format!("Failed to update appearance settings: {}", e))
}

/// Update connection settings
#[tauri::command]
pub async fn settings_update_connection(
    settings: State<'_, SettingsManager>,
    connection: ConnectionSettings,
) -> CommandResult<()> {
    settings
        .update_connection(connection)
        .await
        .map_err(|e| format!("Failed to update connection settings: {}", e))
}

/// Update security settings
#[tauri::command]
pub async fn settings_update_security(
    settings: State<'_, SettingsManager>,
    security: SecuritySettings,
) -> CommandResult<()> {
    settings
        .update_security(security)
        .await
        .map_err(|e| format!("Failed to update security settings: {}", e))
}

/// Update keyboard shortcuts
#[tauri::command]
pub async fn settings_update_shortcuts(
    settings: State<'_, SettingsManager>,
    shortcuts: KeyboardShortcuts,
) -> CommandResult<()> {
    settings
        .update_shortcuts(shortcuts)
        .await
        .map_err(|e| format!("Failed to update keyboard shortcuts: {}", e))
}

/// Update general settings
#[tauri::command]
pub async fn settings_update_general(
    settings: State<'_, SettingsManager>,
    general: GeneralSettings,
) -> CommandResult<()> {
    settings
        .update_general(general)
        .await
        .map_err(|e| format!("Failed to update general settings: {}", e))
}

/// Reset all settings to defaults
#[tauri::command]
pub async fn settings_reset_to_defaults(
    settings: State<'_, SettingsManager>,
) -> CommandResult<()> {
    settings
        .reset_to_defaults()
        .await
        .map_err(|e| format!("Failed to reset settings: {}", e))
}

/// Export settings to a file
#[tauri::command]
pub async fn settings_export(
    settings: State<'_, SettingsManager>,
    path: String,
) -> CommandResult<()> {
    settings
        .export(PathBuf::from(path))
        .await
        .map_err(|e| format!("Failed to export settings: {}", e))
}

/// Import settings from a file
#[tauri::command]
pub async fn settings_import(
    settings: State<'_, SettingsManager>,
    path: String,
) -> CommandResult<AppSettings> {
    settings
        .import(PathBuf::from(path))
        .await
        .map_err(|e| format!("Failed to import settings: {}", e))
}
