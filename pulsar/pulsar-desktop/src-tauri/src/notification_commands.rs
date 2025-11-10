// Tauri commands for notification system

use crate::notifications::{NotificationService, NotificationType};
use tauri::State;

/// Result type for commands
type CommandResult<T> = Result<T, String>;

/// Send a session disconnected notification
#[tauri::command]
pub async fn notify_session_disconnected(
    service: State<'_, NotificationService>,
    session_id: String,
    reason: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::SessionDisconnected { session_id, reason })
        .await
        .map_err(|e| e.to_string())
}

/// Send a session reconnected notification
#[tauri::command]
pub async fn notify_session_reconnected(
    service: State<'_, NotificationService>,
    session_id: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::SessionReconnected { session_id })
        .await
        .map_err(|e| e.to_string())
}

/// Send a file transfer complete notification
#[tauri::command]
pub async fn notify_file_transfer_complete(
    service: State<'_, NotificationService>,
    filename: String,
    success: bool,
    size_bytes: Option<u64>,
) -> CommandResult<()> {
    service
        .send(NotificationType::FileTransferComplete {
            filename,
            success,
            size_bytes,
        })
        .await
        .map_err(|e| e.to_string())
}

/// Send a command completed notification
#[tauri::command]
pub async fn notify_command_completed(
    service: State<'_, NotificationService>,
    command: String,
    exit_code: i32,
    duration_secs: u64,
) -> CommandResult<()> {
    service
        .send(NotificationType::CommandCompleted {
            command,
            exit_code,
            duration_secs,
        })
        .await
        .map_err(|e| e.to_string())
}

/// Send a vault locked notification
#[tauri::command]
pub async fn notify_vault_locked(
    service: State<'_, NotificationService>,
    reason: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::VaultLocked { reason })
        .await
        .map_err(|e| e.to_string())
}

/// Send an update available notification
#[tauri::command]
pub async fn notify_update_available(
    service: State<'_, NotificationService>,
    version: String,
    url: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::UpdateAvailable { version, url })
        .await
        .map_err(|e| e.to_string())
}

/// Send a generic info notification
#[tauri::command]
pub async fn notify_info(
    service: State<'_, NotificationService>,
    title: String,
    message: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::Info { title, message })
        .await
        .map_err(|e| e.to_string())
}

/// Send a generic warning notification
#[tauri::command]
pub async fn notify_warning(
    service: State<'_, NotificationService>,
    title: String,
    message: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::Warning { title, message })
        .await
        .map_err(|e| e.to_string())
}

/// Send a generic error notification
#[tauri::command]
pub async fn notify_error(
    service: State<'_, NotificationService>,
    title: String,
    message: String,
) -> CommandResult<()> {
    service
        .send(NotificationType::Error { title, message })
        .await
        .map_err(|e| e.to_string())
}

/// Test notification (for settings preview)
#[tauri::command]
pub async fn notify_test(
    service: State<'_, NotificationService>,
) -> CommandResult<()> {
    service
        .send(NotificationType::Info {
            title: "Test Notification".to_string(),
            message: "This is a test notification from Pulsar Desktop".to_string(),
        })
        .await
        .map_err(|e| e.to_string())
}

/// Cleanup old notification records
#[tauri::command]
pub async fn notifications_cleanup(
    service: State<'_, NotificationService>,
) -> CommandResult<()> {
    service.cleanup_old_records().await;
    Ok(())
}
