//! SSH client implementation using russh

use crate::known_hosts::{HostKeyVerification, KnownHosts};
use anyhow::{Context, Result};
use russh::client::{self, AuthResult, Handle, Msg};
use russh::keys::*;
use russh::*;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: AuthMethod,
    /// If true, accept unknown host keys automatically (INSECURE, for development only)
    pub accept_unknown_hosts: bool,
    /// If true, accept changed host keys automatically (VERY INSECURE, for development only)
    pub accept_changed_hosts: bool,
}

pub enum AuthMethod {
    Password(String),
    PublicKey { key_path: String, passphrase: Option<String> },
    Agent,
}

struct Client {
    known_hosts: Arc<Mutex<KnownHosts>>,
    hostname: String,
    port: u16,
    accept_unknown: bool,
    accept_changed: bool,
    fingerprint: Arc<Mutex<Option<String>>>,
}

impl client::Handler for Client {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        let fingerprint = KnownHosts::fingerprint(server_public_key);

        // Store the fingerprint for later retrieval
        *self.fingerprint.lock().unwrap() = Some(fingerprint.clone());

        let verification = {
            let known_hosts = self.known_hosts.lock().unwrap();
            known_hosts.verify(&self.hostname, self.port, server_public_key)
        };

        match verification {
            HostKeyVerification::Trusted => {
                tracing::info!(
                    "Host key verified for {}:{} ({})",
                    self.hostname,
                    self.port,
                    fingerprint
                );
                Ok(true)
            }
            HostKeyVerification::Unknown => {
                tracing::warn!(
                    "Unknown host key for {}:{} ({})",
                    self.hostname,
                    self.port,
                    fingerprint
                );

                if self.accept_unknown {
                    tracing::info!("Auto-accepting unknown host key (development mode)");
                    let mut known_hosts = self.known_hosts.lock().unwrap();
                    if let Err(e) = known_hosts.add(&self.hostname, self.port, server_public_key) {
                        tracing::error!("Failed to add host key: {}", e);
                    }
                    Ok(true)
                } else {
                    tracing::error!("Rejecting unknown host key (set accept_unknown_hosts to accept)");
                    Ok(false)
                }
            }
            HostKeyVerification::Changed { old_key } => {
                tracing::error!(
                    "HOST KEY CHANGED for {}:{}! Possible MITM attack!",
                    self.hostname,
                    self.port
                );
                tracing::error!("Old key: {}", old_key);
                tracing::error!("New key: {}", server_public_key.to_openssh().unwrap_or_default());
                tracing::error!("Fingerprint: {}", fingerprint);

                if self.accept_changed {
                    tracing::warn!("Auto-accepting changed host key (VERY INSECURE - development mode)");
                    let mut known_hosts = self.known_hosts.lock().unwrap();
                    if let Err(e) = known_hosts.update(&self.hostname, self.port, server_public_key) {
                        tracing::error!("Failed to update host key: {}", e);
                    }
                    Ok(true)
                } else {
                    tracing::error!("Rejecting changed host key (set accept_changed_hosts to override)");
                    Ok(false)
                }
            }
        }
    }
}

pub struct SshSession {
    handle: Handle<Client>,
    channel: Channel<Msg>,
    fingerprint: String,
}

impl SshSession {
    /// Get the host key fingerprint (SHA256)
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }
}

impl SshSession {
    pub async fn connect(config: SshConfig) -> Result<Self> {
        // Load known_hosts
        let known_hosts = Arc::new(Mutex::new(
            KnownHosts::load().context("Failed to load known_hosts")?,
        ));

        let client_config = client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(3600)),
            ..<_>::default()
        };

        let fingerprint_holder = Arc::new(Mutex::new(None));

        let handler = Client {
            known_hosts,
            hostname: config.host.clone(),
            port: config.port,
            accept_unknown: config.accept_unknown_hosts,
            accept_changed: config.accept_changed_hosts,
            fingerprint: Arc::clone(&fingerprint_holder),
        };

        let mut session = client::connect(
            Arc::new(client_config),
            (config.host.as_str(), config.port),
            handler,
        )
        .await
        .context("Failed to connect to SSH server")?;

        // Authenticate
        let auth_result = match config.auth {
            AuthMethod::Password(password) => {
                session
                    .authenticate_password(config.username, password)
                    .await
                    .context("Password authentication failed")?
            }
            AuthMethod::PublicKey { key_path, passphrase } => {
                let key = load_secret_key(&key_path, passphrase.as_deref())
                    .context("Failed to load SSH key")?;

                let key_with_alg = PrivateKeyWithHashAlg::new(
                    Arc::new(key),
                    None, // Use default hash algorithm
                );

                session
                    .authenticate_publickey(config.username, key_with_alg)
                    .await
                    .context("Public key authentication failed")?
            }
            AuthMethod::Agent => {
                // Connect to SSH agent
                let mut agent_client = russh::keys::agent::client::AgentClient::connect_env()
                    .await
                    .context("Failed to connect to SSH agent")?;

                // Get list of identities from agent
                let identities = agent_client
                    .request_identities()
                    .await
                    .context("Failed to get identities from SSH agent")?;

                if identities.is_empty() {
                    anyhow::bail!("No identities available in SSH agent");
                }

                // Try each identity until one works
                let mut last_error = None;
                let mut auth_result = None;

                for identity in &identities {
                    let comment = identity.comment();
                    tracing::debug!("Trying agent key: {}", comment);

                    match session
                        .authenticate_publickey_with(
                            &config.username,
                            identity.clone(),
                            None, // Use default hash algorithm
                            &mut agent_client
                        )
                        .await
                    {
                        Ok(result) => {
                            if matches!(result, AuthResult::Success) {
                                tracing::info!("SSH agent authentication successful with key: {}", comment);
                                auth_result = Some(result);
                                break;
                            } else {
                                tracing::debug!("Agent key failed with result: {:?}", result);
                                last_error = Some(format!("Authentication failed with result: {:?}", result));
                            }
                        }
                        Err(e) => {
                            tracing::debug!("Agent key failed with error: {}", e);
                            last_error = Some(format!("{}", e));
                        }
                    }
                }

                auth_result.ok_or_else(|| {
                    let err_msg = last_error.unwrap_or_else(|| "No keys worked".to_string());
                    anyhow::anyhow!("SSH agent authentication failed: {}", err_msg)
                })?
            }
        };

        if !matches!(auth_result, AuthResult::Success) {
            anyhow::bail!("SSH authentication failed: {:?}", auth_result);
        }

        tracing::info!("SSH authentication successful");

        // Open a channel
        let channel = session
            .channel_open_session()
            .await
            .context("Failed to open SSH channel")?;

        // Retrieve the stored fingerprint
        let fingerprint = fingerprint_holder
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| "Unknown".to_string());

        Ok(Self {
            handle: session,
            channel,
            fingerprint,
        })
    }

    pub async fn request_pty(&mut self, cols: u32, rows: u32) -> Result<()> {
        let term = std::env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string());

        self.channel
            .request_pty(
                false,      // want_reply
                &term,      // terminal type
                cols,       // columns
                rows,       // rows
                0,          // pixel width (not used)
                0,          // pixel height (not used)
                &[],        // terminal modes
            )
            .await
            .context("Failed to request PTY")?;

        Ok(())
    }

    pub async fn request_shell(&mut self) -> Result<()> {
        self.channel
            .request_shell(true)
            .await
            .context("Failed to request shell")?;

        Ok(())
    }

    pub async fn resize(&mut self, cols: u32, rows: u32) -> Result<()> {
        self.channel
            .window_change(cols, rows, 0, 0)
            .await
            .context("Failed to resize PTY")?;

        Ok(())
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<()> {
        self.channel
            .data(data)
            .await
            .context("Failed to write to SSH channel")?;

        Ok(())
    }

    pub async fn read(&mut self) -> Result<Option<Vec<u8>>> {
        match self.channel.wait().await {
            Some(ChannelMsg::Data { ref data }) => Ok(Some(data.to_vec())),
            Some(ChannelMsg::ExtendedData { ref data, ext: 1 }) => {
                // Stderr data
                Ok(Some(data.to_vec()))
            }
            Some(ChannelMsg::Eof) => Ok(None),
            Some(ChannelMsg::ExitStatus { exit_status }) => {
                tracing::info!("SSH session exited with status: {}", exit_status);
                Ok(None)
            }
            None => Ok(None),
            msg => {
                tracing::debug!("Unhandled SSH message: {:?}", msg);
                Ok(None)
            }
        }
    }

    pub async fn close(self) -> Result<()> {
        self.channel.eof().await?;
        self.handle.disconnect(Disconnect::ByApplication, "", "en").await?;
        Ok(())
    }
}

/// Spawns a task to handle SSH I/O with mpsc channels
pub fn spawn_ssh_io(
    mut session: SshSession,
) -> (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>) {
    let (input_tx, mut input_rx) = mpsc::channel::<Vec<u8>>(100);
    let (output_tx, output_rx) = mpsc::channel::<Vec<u8>>(100);

    tokio::spawn(async move {
        loop {
            tokio::select! {
                // Handle input from frontend
                Some(data) = input_rx.recv() => {
                    if let Err(e) = session.write(&data).await {
                        tracing::error!("Failed to write to SSH: {}", e);
                        break;
                    }
                }

                // Handle output from SSH
                result = session.read() => {
                    match result {
                        Ok(Some(data)) => {
                            if output_tx.send(data).await.is_err() {
                                tracing::error!("Frontend disconnected");
                                break;
                            }
                        }
                        Ok(None) => {
                            tracing::info!("SSH session ended");
                            break;
                        }
                        Err(e) => {
                            tracing::error!("SSH read error: {}", e);
                            break;
                        }
                    }
                }
            }
        }

        tracing::info!("SSH I/O task terminating");
        let _ = session.close().await;
    });

    (input_tx, output_rx)
}
