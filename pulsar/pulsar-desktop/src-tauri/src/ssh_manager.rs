//! SSH session manager for Tauri backend

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tft_transports::{AuthMethod, SshConfig, SshSession, spawn_ssh_io};
use uuid::Uuid;

#[allow(dead_code)]
pub struct SessionInfo {
    pub id: Uuid,
    pub host: String,
    pub username: String,
    pub fingerprint: String,
    pub input_tx: mpsc::Sender<Vec<u8>>,
    pub output_rx: Arc<RwLock<mpsc::Receiver<Vec<u8>>>>,
}

pub struct SshManager {
    sessions: Arc<RwLock<HashMap<Uuid, SessionInfo>>>,
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn connect(
        &self,
        host: String,
        port: u16,
        username: String,
        auth: AuthMethod,
        cols: u32,
        rows: u32,
    ) -> Result<Uuid> {
        tracing::info!("Connecting to {}@{}:{}", username, host, port);

        let config = SshConfig {
            host: host.clone(),
            port,
            username: username.clone(),
            auth,
            accept_unknown_hosts: true,  // Development mode: auto-accept unknown hosts
            accept_changed_hosts: false, // Production: reject changed keys (security)
        };

        let mut session = SshSession::connect(config).await?;
        let fingerprint = session.fingerprint().to_string();

        session.request_pty(cols, rows).await?;
        session.request_shell().await?;

        let (input_tx, output_rx) = spawn_ssh_io(session);

        let session_id = Uuid::new_v4();
        let session_info = SessionInfo {
            id: session_id,
            host,
            username,
            fingerprint,
            input_tx,
            output_rx: Arc::new(RwLock::new(output_rx)),
        };

        self.sessions.write().await.insert(session_id, session_info);

        tracing::info!("Session {} created successfully", session_id);
        Ok(session_id)
    }

    pub async fn send_input(&self, session_id: Uuid, data: Vec<u8>) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        session
            .input_tx
            .send(data)
            .await
            .map_err(|_| anyhow::anyhow!("Failed to send data to session"))?;

        Ok(())
    }

    pub async fn receive_output(&self, session_id: Uuid) -> Result<Option<Vec<u8>>> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let mut output_rx = session.output_rx.write().await;
        Ok(output_rx.recv().await)
    }

    pub async fn disconnect(&self, session_id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        sessions
            .remove(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        tracing::info!("Session {} disconnected", session_id);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn list_sessions(&self) -> Vec<Uuid> {
        self.sessions.read().await.keys().copied().collect()
    }

    pub async fn get_fingerprint(&self, session_id: Uuid) -> Result<String> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(&session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        Ok(session.fingerprint.clone())
    }
}

impl Default for SshManager {
    fn default() -> Self {
        Self::new()
    }
}
