// Desktop notifications module for Pulsar
//
// This module provides:
// - Desktop notification service
// - Integration with settings preferences
// - Notification types and formatting
// - Rate limiting and deduplication

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;

/// Notification types that can be sent
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum NotificationType {
    /// Session was disconnected
    SessionDisconnected {
        session_id: String,
        reason: String,
    },
    /// File transfer completed
    FileTransferComplete {
        filename: String,
        success: bool,
        size_bytes: Option<u64>,
    },
    /// Long-running command completed
    CommandCompleted {
        command: String,
        exit_code: i32,
        duration_secs: u64,
    },
    /// Vault was locked
    VaultLocked {
        reason: String,
    },
    /// Application update available
    UpdateAvailable {
        version: String,
        url: String,
    },
    /// Session reconnected after disconnect
    SessionReconnected {
        session_id: String,
    },
    /// Generic info notification
    Info {
        title: String,
        message: String,
    },
    /// Generic warning notification
    Warning {
        title: String,
        message: String,
    },
    /// Generic error notification
    Error {
        title: String,
        message: String,
    },
}

impl NotificationType {
    /// Get notification title
    pub fn title(&self) -> String {
        match self {
            Self::SessionDisconnected { .. } => "Session Disconnected".to_string(),
            Self::FileTransferComplete { success: true, .. } => "File Transfer Complete".to_string(),
            Self::FileTransferComplete { success: false, .. } => "File Transfer Failed".to_string(),
            Self::CommandCompleted { .. } => "Command Completed".to_string(),
            Self::VaultLocked { .. } => "Vault Locked".to_string(),
            Self::UpdateAvailable { .. } => "Update Available".to_string(),
            Self::SessionReconnected { .. } => "Session Reconnected".to_string(),
            Self::Info { title, .. } => title.clone(),
            Self::Warning { title, .. } => title.clone(),
            Self::Error { title, .. } => title.clone(),
        }
    }

    /// Get notification body/message
    pub fn message(&self) -> String {
        match self {
            Self::SessionDisconnected { session_id, reason } => {
                format!("Session {} disconnected: {}", session_id, reason)
            }
            Self::FileTransferComplete { filename, success: true, size_bytes } => {
                if let Some(size) = size_bytes {
                    format!("Successfully transferred {} ({} bytes)", filename, size)
                } else {
                    format!("Successfully transferred {}", filename)
                }
            }
            Self::FileTransferComplete { filename, success: false, .. } => {
                format!("Failed to transfer {}", filename)
            }
            Self::CommandCompleted { command, exit_code, duration_secs } => {
                let duration_str = format_duration(*duration_secs);
                format!("'{}' completed with exit code {} after {}", command, exit_code, duration_str)
            }
            Self::VaultLocked { reason } => {
                format!("Your vault has been locked: {}", reason)
            }
            Self::UpdateAvailable { version, .. } => {
                format!("Pulsar {} is now available", version)
            }
            Self::SessionReconnected { session_id } => {
                format!("Session {} reconnected successfully", session_id)
            }
            Self::Info { message, .. } => message.clone(),
            Self::Warning { message, .. } => message.clone(),
            Self::Error { message, .. } => message.clone(),
        }
    }

    /// Get notification icon (emoji for now, could be icon path)
    pub fn icon(&self) -> &'static str {
        match self {
            Self::SessionDisconnected { .. } => "⚠️",
            Self::FileTransferComplete { success: true, .. } => "✅",
            Self::FileTransferComplete { success: false, .. } => "❌",
            Self::CommandCompleted { exit_code, .. } => {
                if *exit_code == 0 { "✅" } else { "❌" }
            }
            Self::VaultLocked { .. } => "🔒",
            Self::UpdateAvailable { .. } => "🔔",
            Self::SessionReconnected { .. } => "✅",
            Self::Info { .. } => "ℹ️",
            Self::Warning { .. } => "⚠️",
            Self::Error { .. } => "❌",
        }
    }

    /// Get a deduplication key for this notification type
    /// Returns None if notification should not be deduplicated
    fn dedup_key(&self) -> Option<String> {
        match self {
            // Deduplicate session notifications by session_id
            Self::SessionDisconnected { session_id, .. } => {
                Some(format!("session_disconnected:{}", session_id))
            }
            Self::SessionReconnected { session_id } => {
                Some(format!("session_reconnected:{}", session_id))
            }
            // Deduplicate vault notifications
            Self::VaultLocked { .. } => Some("vault_locked".to_string()),
            // Don't deduplicate file transfers or commands
            Self::FileTransferComplete { .. } => None,
            Self::CommandCompleted { .. } => None,
            // Deduplicate update notifications
            Self::UpdateAvailable { version, .. } => {
                Some(format!("update_available:{}", version))
            }
            // Don't deduplicate generic notifications
            Self::Info { .. } | Self::Warning { .. } | Self::Error { .. } => None,
        }
    }
}

/// Notification record for history and deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NotificationRecord {
    id: String,
    notification_type: NotificationType,
    sent_at: DateTime<Utc>,
}

/// Notification service manages sending notifications
pub struct NotificationService {
    app_handle: AppHandle,
    recent_notifications: Arc<RwLock<HashMap<String, NotificationRecord>>>,
    dedup_window_secs: u64,
}

impl NotificationService {
    /// Create a new notification service
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            recent_notifications: Arc::new(RwLock::new(HashMap::new())),
            dedup_window_secs: 60, // Don't send duplicate notifications within 60 seconds
        }
    }

    /// Send a notification
    ///
    /// This will check settings to see if notifications are enabled and respect user preferences.
    pub async fn send(&self, notification: NotificationType) -> Result<()> {
        // Check if notifications should be sent based on settings
        if !self.should_send_notification(&notification).await {
            tracing::debug!(
                notification_type = ?notification,
                "Notification suppressed by settings"
            );
            return Ok(());
        }

        // Check for recent duplicate
        if let Some(dedup_key) = notification.dedup_key() {
            let mut recent = self.recent_notifications.write().await;

            if let Some(record) = recent.get(&dedup_key) {
                let elapsed = Utc::now().signed_duration_since(record.sent_at);
                if elapsed.num_seconds() < self.dedup_window_secs as i64 {
                    tracing::debug!(
                        notification_type = ?notification,
                        dedup_key = %dedup_key,
                        elapsed_secs = elapsed.num_seconds(),
                        "Notification deduplicated"
                    );
                    return Ok(());
                }
            }

            // Record this notification
            recent.insert(
                dedup_key.clone(),
                NotificationRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    notification_type: notification.clone(),
                    sent_at: Utc::now(),
                },
            );
        }

        // Send the notification
        self.send_native_notification(&notification).await?;

        tracing::info!(
            notification_type = ?notification,
            "Notification sent"
        );

        Ok(())
    }

    /// Check if a notification should be sent based on settings
    async fn should_send_notification(&self, notification: &NotificationType) -> bool {
        // Try to get settings
        let settings_manager = match self.app_handle.try_state::<crate::settings::SettingsManager>() {
            Some(mgr) => mgr,
            None => {
                tracing::warn!("Settings manager not found, allowing notification");
                return true;
            }
        };

        let security = settings_manager.get_security().await;

        // Check if notifications are enabled globally
        if !security.enable_notifications {
            return false;
        }

        // Check specific notification preferences
        match notification {
            NotificationType::SessionDisconnected { .. } |
            NotificationType::SessionReconnected { .. } => {
                security.notify_session_disconnect
            }
            NotificationType::FileTransferComplete { .. } => {
                security.notify_file_transfer_complete
            }
            NotificationType::CommandCompleted { duration_secs, .. } => {
                if security.notify_command_threshold == 0 {
                    false
                } else {
                    *duration_secs >= security.notify_command_threshold as u64
                }
            }
            // Always allow these notification types if notifications are enabled
            NotificationType::VaultLocked { .. } |
            NotificationType::UpdateAvailable { .. } |
            NotificationType::Info { .. } |
            NotificationType::Warning { .. } |
            NotificationType::Error { .. } => true,
        }
    }

    /// Send a native OS notification using Tauri
    async fn send_native_notification(&self, notification: &NotificationType) -> Result<()> {
        use tauri_plugin_notification::NotificationExt;

        let title = notification.title();
        let body = notification.message();
        let icon = notification.icon();

        // Build notification with icon in the body (since Tauri doesn't support emoji icons directly)
        let body_with_icon = format!("{} {}", icon, body);

        // Use Tauri's notification plugin
        self.app_handle
            .notification()
            .builder()
            .title(title)
            .body(body_with_icon)
            .show()?;

        Ok(())
    }

    /// Clear old notification records (cleanup)
    pub async fn cleanup_old_records(&self) {
        let mut recent = self.recent_notifications.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(self.dedup_window_secs as i64 * 2);

        recent.retain(|_, record| record.sent_at > cutoff);

        tracing::debug!(
            remaining = recent.len(),
            "Cleaned up old notification records"
        );
    }
}

/// Format duration in human-readable format
fn format_duration(secs: u64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        let mins = secs / 60;
        let rem_secs = secs % 60;
        if rem_secs == 0 {
            format!("{}m", mins)
        } else {
            format!("{}m {}s", mins, rem_secs)
        }
    } else {
        let hours = secs / 3600;
        let rem_mins = (secs % 3600) / 60;
        if rem_mins == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h {}m", hours, rem_mins)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_titles() {
        let notif = NotificationType::SessionDisconnected {
            session_id: "test".to_string(),
            reason: "timeout".to_string(),
        };
        assert_eq!(notif.title(), "Session Disconnected");

        let notif = NotificationType::FileTransferComplete {
            filename: "test.txt".to_string(),
            success: true,
            size_bytes: Some(1024),
        };
        assert_eq!(notif.title(), "File Transfer Complete");
    }

    #[test]
    fn test_notification_messages() {
        let notif = NotificationType::CommandCompleted {
            command: "ls".to_string(),
            exit_code: 0,
            duration_secs: 125,
        };
        let msg = notif.message();
        assert!(msg.contains("ls"));
        assert!(msg.contains("exit code 0"));
        assert!(msg.contains("2m 5s"));
    }

    #[test]
    fn test_dedup_keys() {
        let notif1 = NotificationType::SessionDisconnected {
            session_id: "sess1".to_string(),
            reason: "timeout".to_string(),
        };
        assert_eq!(notif1.dedup_key(), Some("session_disconnected:sess1".to_string()));

        let notif2 = NotificationType::FileTransferComplete {
            filename: "test.txt".to_string(),
            success: true,
            size_bytes: None,
        };
        assert_eq!(notif2.dedup_key(), None);
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(60), "1m");
        assert_eq!(format_duration(90), "1m 30s");
        assert_eq!(format_duration(3600), "1h");
        assert_eq!(format_duration(3720), "1h 2m");
        assert_eq!(format_duration(7325), "2h 2m");
    }
}
