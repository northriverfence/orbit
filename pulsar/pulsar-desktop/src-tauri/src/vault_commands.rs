use crate::vault::{
    CertificateData, CredentialSummary, CredentialType, DecryptedCredential,
    DecryptedCredentialData, PasswordData, SshKeyData, Vault, VaultState,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Error result for Tauri commands
type CommandResult<T> = Result<T, String>;

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

/// Get the current vault state
#[tauri::command]
pub async fn vault_get_state(vault: State<'_, Vault>) -> CommandResult<String> {
    vault
        .with_manager(|manager| {
            Box::pin(async move {
                let state = manager.get_state().await;
                Ok(match state {
                    VaultState::Uninitialized => "uninitialized",
                    VaultState::Locked => "locked",
                    VaultState::Unlocked => "unlocked",
                }
                .to_string())
            })
        })
        .await
        .map_err(map_err)
}

/// Check if vault is initialized
#[tauri::command]
pub async fn vault_is_initialized(vault: State<'_, Vault>) -> CommandResult<bool> {
    vault
        .with_manager(|manager| Box::pin(async move { Ok(manager.is_initialized().await) }))
        .await
        .map_err(map_err)
}

/// Check if vault is unlocked
#[tauri::command]
pub async fn vault_is_unlocked(vault: State<'_, Vault>) -> CommandResult<bool> {
    vault
        .with_manager(|manager| Box::pin(async move { Ok(manager.is_unlocked().await) }))
        .await
        .map_err(map_err)
}

/// Initialize vault with a master password
#[tauri::command]
pub async fn vault_initialize(
    vault: State<'_, Vault>,
    master_password: String,
) -> CommandResult<()> {
    vault
        .with_manager(|manager| {
            Box::pin(async move { manager.initialize(&master_password).await })
        })
        .await
        .map_err(map_err)
}

/// Unlock vault with master password
#[tauri::command]
pub async fn vault_unlock(vault: State<'_, Vault>, master_password: String) -> CommandResult<()> {
    vault
        .with_manager(|manager| Box::pin(async move { manager.unlock(&master_password).await }))
        .await
        .map_err(map_err)
}

/// Lock vault
#[tauri::command]
pub async fn vault_lock(vault: State<'_, Vault>) -> CommandResult<()> {
    vault
        .with_manager(|manager| Box::pin(async move { manager.lock().await }))
        .await
        .map_err(map_err)
}

/// Request to store a credential
#[derive(Debug, Serialize, Deserialize)]
pub struct StoreCredentialRequest {
    pub name: String,
    pub data: DecryptedCredentialData,
    pub tags: Vec<String>,
    pub username: Option<String>,
    pub host_pattern: Option<String>,
}

/// Store a credential
#[tauri::command]
pub async fn vault_store_credential(
    vault: State<'_, Vault>,
    request: StoreCredentialRequest,
) -> CommandResult<String> {
    vault
        .with_manager(|manager| {
            Box::pin(async move {
                manager
                    .store_credential(
                        request.name,
                        request.data,
                        request.tags,
                        request.username,
                        request.host_pattern,
                    )
                    .await
            })
        })
        .await
        .map_err(map_err)
}

/// Store an SSH key
#[tauri::command]
pub async fn vault_store_ssh_key(
    vault: State<'_, Vault>,
    name: String,
    private_key: String,
    public_key: Option<String>,
    passphrase: Option<String>,
    tags: Vec<String>,
    username: Option<String>,
    host_pattern: Option<String>,
) -> CommandResult<String> {
    let data = DecryptedCredentialData::SshKey(SshKeyData {
        private_key,
        public_key,
        passphrase,
    });

    vault
        .with_manager(|manager| {
            Box::pin(async move {
                manager
                    .store_credential(name, data, tags, username, host_pattern)
                    .await
            })
        })
        .await
        .map_err(map_err)
}

/// Store a password
#[tauri::command]
pub async fn vault_store_password(
    vault: State<'_, Vault>,
    name: String,
    password: String,
    username: Option<String>,
    tags: Vec<String>,
    host_pattern: Option<String>,
) -> CommandResult<String> {
    let data = DecryptedCredentialData::Password(PasswordData {
        password,
        username: username.clone(),
    });

    vault
        .with_manager(|manager| {
            Box::pin(async move {
                manager
                    .store_credential(name, data, tags, username, host_pattern)
                    .await
            })
        })
        .await
        .map_err(map_err)
}

/// Store a certificate
#[tauri::command]
pub async fn vault_store_certificate(
    vault: State<'_, Vault>,
    name: String,
    certificate: String,
    private_key: Option<String>,
    passphrase: Option<String>,
    tags: Vec<String>,
    username: Option<String>,
    host_pattern: Option<String>,
) -> CommandResult<String> {
    let data = DecryptedCredentialData::Certificate(CertificateData {
        certificate,
        private_key,
        passphrase,
    });

    vault
        .with_manager(|manager| {
            Box::pin(async move {
                manager
                    .store_credential(name, data, tags, username, host_pattern)
                    .await
            })
        })
        .await
        .map_err(map_err)
}

/// Get a decrypted credential by ID
#[tauri::command]
pub async fn vault_get_credential(
    vault: State<'_, Vault>,
    id: String,
) -> CommandResult<DecryptedCredential> {
    vault
        .with_manager(|manager| Box::pin(async move { manager.get_credential(&id).await }))
        .await
        .map_err(map_err)
}

/// List all credentials (summaries only)
#[tauri::command]
pub async fn vault_list_credentials(
    vault: State<'_, Vault>,
) -> CommandResult<Vec<CredentialSummary>> {
    vault
        .with_manager(|manager| Box::pin(async move { manager.list_credentials().await }))
        .await
        .map_err(map_err)
}

/// List credentials by type
#[tauri::command]
pub async fn vault_list_credentials_by_type(
    vault: State<'_, Vault>,
    credential_type: String,
) -> CommandResult<Vec<CredentialSummary>> {
    let cred_type = match credential_type.as_str() {
        "ssh_key" => CredentialType::SshKey,
        "password" => CredentialType::Password,
        "certificate" => CredentialType::Certificate,
        _ => return Err(format!("Invalid credential type: {}", credential_type)),
    };

    vault
        .with_manager(|manager| {
            Box::pin(async move { manager.list_credentials_by_type(cred_type).await })
        })
        .await
        .map_err(map_err)
}

/// Find credentials by host pattern
#[tauri::command]
pub async fn vault_find_credentials_by_host(
    vault: State<'_, Vault>,
    host: String,
) -> CommandResult<Vec<CredentialSummary>> {
    vault
        .with_manager(|manager| {
            Box::pin(async move { manager.find_credentials_by_host(&host).await })
        })
        .await
        .map_err(map_err)
}

/// Delete a credential
#[tauri::command]
pub async fn vault_delete_credential(vault: State<'_, Vault>, id: String) -> CommandResult<()> {
    vault
        .with_manager(|manager| Box::pin(async move { manager.delete_credential(&id).await }))
        .await
        .map_err(map_err)
}
