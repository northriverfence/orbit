// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;
use tauri::Manager;

mod autostart_commands;
mod commands;
mod daemon_client;
mod daemon_commands;
mod notifications;
mod notification_commands;
mod settings;
mod settings_commands;
mod ssh_manager;
mod vault;
mod vault_commands;

use autostart_commands::AutoStartState;
use daemon_client::DaemonClient;
use notifications::NotificationService;
use settings::SettingsManager;
use ssh_manager::SshManager;
use vault::Vault;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"))
        )
        .init();

    tracing::info!("Starting Pulsar Desktop v{}", env!("CARGO_PKG_VERSION"));

    // Create SSH manager (legacy, for backward compatibility)
    let ssh_manager = Arc::new(SshManager::new());

    // Create daemon client
    let socket_path = dirs::config_dir()
        .expect("Could not find config directory")
        .join("orbit")
        .join("pulsar.sock");

    let daemon_client = Arc::new(DaemonClient::new(socket_path));

    tracing::info!("Daemon client initialized");

    // Initialize vault
    let vault_db_path = dirs::config_dir()
        .expect("Could not find config directory")
        .join("orbit")
        .join("pulsar_vault.db");

    let vault = Vault::new();
    vault.init(vault_db_path).await.expect("Failed to initialize vault");

    tracing::info!("Vault initialized");

    // Initialize settings
    let config_dir = dirs::config_dir()
        .expect("Could not find config directory")
        .join("orbit");

    let settings_manager = SettingsManager::new(config_dir)
        .expect("Failed to initialize settings");

    tracing::info!("Settings initialized");

    // Initialize auto-start state
    let autostart_state = AutoStartState::new();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .manage(ssh_manager)
        .manage(daemon_client)
        .manage(vault)
        .manage(settings_manager)
        .manage(autostart_state)
        .setup(|app| {
            // Initialize notification service after app is set up
            let app_handle = app.handle().clone();
            let notification_service = NotificationService::new(app_handle);
            app.manage(notification_service);

            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Legacy SSH commands (direct connection)
            commands::connect_ssh,
            commands::disconnect_ssh,
            commands::send_input,
            commands::receive_output,
            commands::resize_terminal,
            commands::get_fingerprint,
            commands::check_ssh_agent,
            commands::list_agent_identities,
            // New daemon commands (via pulsar-daemon)
            daemon_commands::daemon_create_local_session,
            daemon_commands::daemon_create_ssh_session,
            daemon_commands::daemon_list_sessions,
            daemon_commands::daemon_attach_session,
            daemon_commands::daemon_detach_session,
            daemon_commands::daemon_terminate_session,
            daemon_commands::daemon_resize_terminal,
            daemon_commands::daemon_send_input,
            daemon_commands::daemon_receive_output,
            daemon_commands::daemon_get_status,
            daemon_commands::daemon_check_connection,
            // Workspace commands
            daemon_commands::workspace_create,
            daemon_commands::workspace_get,
            daemon_commands::workspace_list,
            daemon_commands::workspace_update,
            daemon_commands::workspace_delete,
            daemon_commands::workspace_save_snapshot,
            daemon_commands::workspace_list_snapshots,
            daemon_commands::workspace_restore_snapshot,
            // Vault commands
            vault_commands::vault_get_state,
            vault_commands::vault_is_initialized,
            vault_commands::vault_is_unlocked,
            vault_commands::vault_initialize,
            vault_commands::vault_unlock,
            vault_commands::vault_lock,
            vault_commands::vault_store_credential,
            vault_commands::vault_store_ssh_key,
            vault_commands::vault_store_password,
            vault_commands::vault_store_certificate,
            vault_commands::vault_get_credential,
            vault_commands::vault_list_credentials,
            vault_commands::vault_list_credentials_by_type,
            vault_commands::vault_find_credentials_by_host,
            vault_commands::vault_delete_credential,
            // Settings commands
            settings_commands::settings_get_all,
            settings_commands::settings_get_appearance,
            settings_commands::settings_get_connection,
            settings_commands::settings_get_security,
            settings_commands::settings_get_shortcuts,
            settings_commands::settings_get_general,
            settings_commands::settings_update_appearance,
            settings_commands::settings_update_connection,
            settings_commands::settings_update_security,
            settings_commands::settings_update_shortcuts,
            settings_commands::settings_update_general,
            settings_commands::settings_reset_to_defaults,
            settings_commands::settings_export,
            settings_commands::settings_import,
            // Notification commands
            notification_commands::notify_session_disconnected,
            notification_commands::notify_session_reconnected,
            notification_commands::notify_file_transfer_complete,
            notification_commands::notify_command_completed,
            notification_commands::notify_vault_locked,
            notification_commands::notify_update_available,
            notification_commands::notify_info,
            notification_commands::notify_warning,
            notification_commands::notify_error,
            notification_commands::notify_test,
            notification_commands::notifications_cleanup,
            // Auto-start commands
            autostart_commands::autostart_set_daemon_path,
            autostart_commands::autostart_is_installed,
            autostart_commands::autostart_is_enabled,
            autostart_commands::autostart_install,
            autostart_commands::autostart_uninstall,
            autostart_commands::autostart_enable,
            autostart_commands::autostart_disable,
            autostart_commands::autostart_start,
            autostart_commands::autostart_stop,
            autostart_commands::autostart_get_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
