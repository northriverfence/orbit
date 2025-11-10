//! Tauri commands for frontend-backend communication

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::io::Write;
use tauri::State;
use tft_transports::AuthMethod;
use uuid::Uuid;

use crate::ssh_manager::SshManager;
use crate::vault::Vault;

#[derive(Debug, Serialize, Deserialize)]
pub struct SshConnectionConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethodDto,
    pub cols: u32,
    pub rows: u32,
    pub credential_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthMethodDto {
    Password { password: String },
    PublicKey { key_path: String, passphrase: Option<String> },
    Agent,
}

impl From<AuthMethodDto> for AuthMethod {
    fn from(dto: AuthMethodDto) -> Self {
        match dto {
            AuthMethodDto::Password { password } => AuthMethod::Password(password),
            AuthMethodDto::PublicKey { key_path, passphrase } => {
                AuthMethod::PublicKey { key_path, passphrase }
            }
            AuthMethodDto::Agent => AuthMethod::Agent,
        }
    }
}

#[tauri::command]
pub async fn connect_ssh(
    config: SshConnectionConfig,
    ssh_manager: State<'_, Arc<SshManager>>,
    vault: State<'_, Vault>,
) -> Result<String, String> {
    tracing::info!(
        "Command: connect_ssh to {}@{}:{}",
        config.username,
        config.host,
        config.port
    );

    // Handle vault keys
    let auth_method = match config.auth_method {
        AuthMethodDto::PublicKey { ref key_path, ref passphrase } if key_path == "<from-vault>" => {
            tracing::info!("Using SSH key from vault");

            // Get credential_id
            let credential_id = config.credential_id.as_ref()
                .ok_or_else(|| "Credential ID required for vault keys".to_string())?;

            // Retrieve credential from vault
            let credential_id_clone = credential_id.clone();
            let credential = vault
                .with_manager(|manager| Box::pin(async move { manager.get_credential(&credential_id_clone).await }))
                .await
                .map_err(|e| format!("Failed to retrieve credential from vault: {}", e))?;

            // Extract SSH key data
            match credential.data {
                crate::vault::DecryptedCredentialData::SshKey(key_data) => {
                    // Write key to temporary file
                    let mut temp_file = tempfile::NamedTempFile::new()
                        .map_err(|e| format!("Failed to create temp file: {}", e))?;

                    temp_file.write_all(key_data.private_key.as_bytes())
                        .map_err(|e| format!("Failed to write key to temp file: {}", e))?;

                    // Get the path and persist the file (keep it around)
                    let (_, temp_path) = temp_file.keep()
                        .map_err(|e| format!("Failed to persist temp file: {}", e))?;

                    let temp_path_str = temp_path.to_string_lossy().to_string();
                    tracing::info!("Wrote vault SSH key to temporary file: {}", temp_path_str);

                    AuthMethod::PublicKey {
                        key_path: temp_path_str,
                        passphrase: key_data.passphrase,
                    }
                }
                _ => return Err("Credential is not an SSH key".to_string()),
            }
        }
        _ => config.auth_method.into(),
    };

    let session_id = ssh_manager
        .connect(
            config.host,
            config.port,
            config.username,
            auth_method,
            config.cols,
            config.rows,
        )
        .await
        .map_err(|e| {
            tracing::error!("SSH connection failed: {}", e);
            format!("Connection failed: {}", e)
        })?;

    Ok(session_id.to_string())
}

#[tauri::command]
pub async fn disconnect_ssh(
    session_id: String,
    ssh_manager: State<'_, Arc<SshManager>>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    ssh_manager
        .disconnect(uuid)
        .await
        .map_err(|e| format!("Disconnect failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn send_input(
    session_id: String,
    data: String,
    ssh_manager: State<'_, Arc<SshManager>>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    ssh_manager
        .send_input(uuid, data.into_bytes())
        .await
        .map_err(|e| format!("Send failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn receive_output(
    session_id: String,
    ssh_manager: State<'_, Arc<SshManager>>,
) -> Result<Option<Vec<u8>>, String> {
    let uuid = Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    ssh_manager
        .receive_output(uuid)
        .await
        .map_err(|e| format!("Receive failed: {}", e))
}

#[tauri::command]
pub async fn resize_terminal(
    session_id: String,
    cols: u32,
    rows: u32,
) -> Result<(), String> {
    // TODO: Implement resize for SSH session
    tracing::debug!("Resize terminal {}: {}x{}", session_id, cols, rows);
    Ok(())
}

#[tauri::command]
pub async fn get_fingerprint(
    session_id: String,
    ssh_manager: State<'_, Arc<SshManager>>,
) -> Result<String, String> {
    let uuid = Uuid::parse_str(&session_id).map_err(|e| format!("Invalid session ID: {}", e))?;

    ssh_manager
        .get_fingerprint(uuid)
        .await
        .map_err(|e| format!("Failed to get fingerprint: {}", e))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentIdentity {
    pub comment: Option<String>,
    pub fingerprint: String,
}

#[tauri::command]
pub async fn check_ssh_agent() -> Result<bool, String> {
    tracing::info!("Checking SSH agent availability");

    // Try to connect to SSH agent
    match russh::keys::agent::client::AgentClient::connect_env().await {
        Ok(_) => {
            tracing::info!("SSH agent is available");
            Ok(true)
        }
        Err(e) => {
            tracing::info!("SSH agent not available: {}", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn list_agent_identities() -> Result<Vec<AgentIdentity>, String> {
    tracing::info!("Listing SSH agent identities");

    // Connect to SSH agent
    let mut agent_client = russh::keys::agent::client::AgentClient::connect_env()
        .await
        .map_err(|e| format!("Failed to connect to SSH agent: {}", e))?;

    // Get list of identities
    let identities = agent_client
        .request_identities()
        .await
        .map_err(|e| format!("Failed to get identities from SSH agent: {}", e))?;

    // Convert to our DTO format
    let result: Vec<AgentIdentity> = identities
        .iter()
        .map(|identity| {
            // Get fingerprint using russh
            let fingerprint = identity.fingerprint(russh::keys::HashAlg::Sha256).to_string();

            AgentIdentity {
                comment: Some(identity.comment().to_string()),
                fingerprint,
            }
        })
        .collect();

    tracing::info!("Found {} identities in SSH agent", result.len());
    Ok(result)
}
