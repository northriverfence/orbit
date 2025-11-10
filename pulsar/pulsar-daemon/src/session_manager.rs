//! Session manager for tracking active terminal sessions
//!
//! Provides thread-safe management of multiple concurrent sessions with:
//! - Async session lifecycle (create, attach, detach, terminate)
//! - Session persistence and restoration
//! - Automatic cleanup of dead sessions
//! - Multi-client support (multiple clients can attach to same session)

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use terminal_core::{SessionConfig, TerminalSession};
use tokio::sync::{broadcast, RwLock};
use tokio::time::{sleep, Duration};
use tracing::{debug, error};
use uuid::Uuid;

/// Unique identifier for connected clients
pub type ClientId = Uuid;

/// Session state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionState {
    /// Session is running with active PTY
    Running,
    /// Session is detached (no clients connected)
    Detached,
    /// Session has been stopped/terminated
    Stopped,
}

/// Session type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    /// Local shell session
    Local,
    /// SSH session to remote host
    Ssh { host: String, port: u16 },
    /// Serial port connection
    Serial { device: String },
}

/// Extended session data with lifecycle management
pub struct SessionData {
    pub id: Uuid,
    pub name: String,
    pub session_type: SessionType,
    pub terminal_session: Arc<RwLock<TerminalSession>>,
    pub created_at: DateTime<Utc>,
    pub last_active: Arc<RwLock<DateTime<Utc>>>,
    pub state: Arc<RwLock<SessionState>>,
    /// Set of client IDs currently attached to this session
    pub clients: Arc<RwLock<HashSet<ClientId>>>,
    /// Broadcast channel for PTY output (all attached clients receive)
    pub output_broadcast: broadcast::Sender<Vec<u8>>,
}

/// Lightweight session info for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: Uuid,
    pub name: String,
    pub session_type: SessionType,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub state: SessionState,
    pub num_clients: usize,
}

/// Thread-safe session manager
pub struct SessionManager {
    /// Active sessions indexed by ID
    sessions: Arc<RwLock<HashMap<Uuid, Arc<SessionData>>>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new session
    pub async fn create_session(
        &self,
        name: String,
        session_type: SessionType,
        config: SessionConfig,
    ) -> Result<Uuid> {
        let terminal_session = TerminalSession::new(config)?;
        let id = *terminal_session.id();

        let (output_broadcast, _) = broadcast::channel(1024);

        let session_data = Arc::new(SessionData {
            id,
            name,
            session_type,
            terminal_session: Arc::new(RwLock::new(terminal_session)),
            created_at: Utc::now(),
            last_active: Arc::new(RwLock::new(Utc::now())),
            state: Arc::new(RwLock::new(SessionState::Running)),
            clients: Arc::new(RwLock::new(HashSet::new())),
            output_broadcast: output_broadcast.clone(),
        });

        let mut sessions = self.sessions.write().await;
        sessions.insert(id, Arc::clone(&session_data));

        // Spawn PTY output broadcasting task
        Self::spawn_output_broadcaster(session_data);

        Ok(id)
    }

    /// Spawn a task that reads PTY output and broadcasts to all subscribers
    fn spawn_output_broadcaster(session: Arc<SessionData>) {
        tokio::spawn(async move {
            let session_id = session.id;
            debug!("Starting output broadcaster for session: {}", session_id);

            let mut buffer = vec![0u8; 8192]; // 8KB buffer

            loop {
                // Check if session is stopped
                if let Ok(state) = session.state.try_read() {
                    if *state == SessionState::Stopped {
                        debug!("Session stopped, ending broadcaster: {}", session_id);
                        break;
                    }
                }

                // Try to read from PTY (non-blocking)
                let bytes_read = {
                    let mut terminal = session.terminal_session.write().await;
                    match terminal.try_read(&mut buffer) {
                        Ok(n) if n > 0 => n,
                        Ok(_) => {
                            // No data available, sleep briefly
                            sleep(Duration::from_millis(10)).await;
                            continue;
                        }
                        Err(e) => {
                            error!("PTY read error for session {}: {}", session_id, e);
                            sleep(Duration::from_millis(100)).await;
                            continue;
                        }
                    }
                };

                // Broadcast output to all subscribers (WebSocket clients)
                let data = buffer[..bytes_read].to_vec();
                if let Err(e) = session.output_broadcast.send(data) {
                    // No subscribers, that's ok
                    debug!("No subscribers for session {}: {}", session_id, e);
                }

                // Update last active time
                *session.last_active.write().await = Utc::now();
            }

            debug!("Output broadcaster ended for session: {}", session_id);
        });
    }

    /// Get a session by ID
    pub async fn get_session(&self, id: Uuid) -> Result<Arc<SessionData>> {
        let sessions = self.sessions.read().await;
        sessions
            .get(&id)
            .cloned()
            .ok_or_else(|| anyhow!("Session not found: {}", id))
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<SessionInfo> {
        let sessions = self.sessions.read().await;
        let mut infos = Vec::new();

        for session in sessions.values() {
            infos.push(SessionInfo {
                id: session.id,
                name: session.name.clone(),
                session_type: session.session_type.clone(),
                created_at: session.created_at,
                last_active: *session.last_active.read().await,
                state: session.state.read().await.clone(),
                num_clients: session.clients.read().await.len(),
            });
        }

        infos
    }

    /// Attach a client to a session
    pub async fn attach_client(&self, session_id: Uuid, client_id: ClientId) -> Result<()> {
        let session = self.get_session(session_id).await?;

        // Add client to session
        session.clients.write().await.insert(client_id);

        // Update state to Running if it was Detached
        let mut state = session.state.write().await;
        if *state == SessionState::Detached {
            *state = SessionState::Running;
        }

        // Update last active time
        *session.last_active.write().await = Utc::now();

        Ok(())
    }

    /// Detach a client from a session
    pub async fn detach_client(&self, session_id: Uuid, client_id: ClientId) -> Result<()> {
        let session = self.get_session(session_id).await?;

        // Remove client from session
        session.clients.write().await.remove(&client_id);

        // If no clients left, mark as Detached
        if session.clients.read().await.is_empty() {
            *session.state.write().await = SessionState::Detached;
        }

        // Update last active time
        *session.last_active.write().await = Utc::now();

        Ok(())
    }

    /// Terminate a session
    pub async fn terminate_session(&self, id: Uuid) -> Result<()> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.remove(&id) {
            // Mark as stopped
            *session.state.write().await = SessionState::Stopped;

            // Clear all clients
            session.clients.write().await.clear();

            Ok(())
        } else {
            Err(anyhow!("Session not found: {}", id))
        }
    }

    /// Clean up dead/stopped sessions
    pub async fn cleanup_dead_sessions(&self) {
        let mut sessions = self.sessions.write().await;

        // Find sessions that are stopped
        let dead_ids: Vec<Uuid> = sessions
            .iter()
            .filter_map(|(id, session)| {
                // Check if state is Stopped (using try_read to avoid blocking)
                if let Ok(state) = session.state.try_read() {
                    if *state == SessionState::Stopped {
                        return Some(*id);
                    }
                }
                None
            })
            .collect();

        // Remove dead sessions
        for id in dead_ids {
            sessions.remove(&id);
        }
    }

    /// Get number of active sessions
    pub async fn count_sessions(&self) -> usize {
        self.sessions.read().await.len()
    }

    /// Get number of active clients across all sessions
    pub async fn count_clients(&self) -> usize {
        let sessions = self.sessions.read().await;
        let mut total = 0;

        for session in sessions.values() {
            total += session.clients.read().await.len();
        }

        total
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert_eq!(manager.count_sessions().await, 0);
    }

    #[tokio::test]
    async fn test_create_session() {
        let manager = SessionManager::new();
        let config = SessionConfig::new("test".to_string());

        let id = manager
            .create_session("test-session".to_string(), SessionType::Local, config)
            .await
            .unwrap();

        assert_eq!(manager.count_sessions().await, 1);

        let session = manager.get_session(id).await.unwrap();
        assert_eq!(session.name, "test-session");
        assert_eq!(*session.state.read().await, SessionState::Running);
    }

    #[tokio::test]
    async fn test_list_sessions() {
        let manager = SessionManager::new();

        let config1 = SessionConfig::new("test1".to_string());
        let config2 = SessionConfig::new("test2".to_string());

        manager
            .create_session("session-1".to_string(), SessionType::Local, config1)
            .await
            .unwrap();
        manager
            .create_session("session-2".to_string(), SessionType::Local, config2)
            .await
            .unwrap();

        let sessions = manager.list_sessions().await;
        assert_eq!(sessions.len(), 2);
        assert!(sessions.iter().any(|s| s.name == "session-1"));
        assert!(sessions.iter().any(|s| s.name == "session-2"));
    }

    #[tokio::test]
    async fn test_attach_detach_client() {
        let manager = SessionManager::new();
        let config = SessionConfig::new("test".to_string());

        let session_id = manager
            .create_session("test-session".to_string(), SessionType::Local, config)
            .await
            .unwrap();

        let client_id = Uuid::new_v4();

        // Attach client
        manager.attach_client(session_id, client_id).await.unwrap();
        assert_eq!(manager.count_clients().await, 1);

        let session = manager.get_session(session_id).await.unwrap();
        assert!(session.clients.read().await.contains(&client_id));
        assert_eq!(*session.state.read().await, SessionState::Running);

        // Detach client
        manager.detach_client(session_id, client_id).await.unwrap();
        assert_eq!(manager.count_clients().await, 0);
        assert_eq!(*session.state.read().await, SessionState::Detached);
    }

    #[tokio::test]
    async fn test_terminate_session() {
        let manager = SessionManager::new();
        let config = SessionConfig::new("test".to_string());

        let id = manager
            .create_session("test-session".to_string(), SessionType::Local, config)
            .await
            .unwrap();

        assert_eq!(manager.count_sessions().await, 1);

        manager.terminate_session(id).await.unwrap();
        assert_eq!(manager.count_sessions().await, 0);
    }

    #[tokio::test]
    async fn test_cleanup_dead_sessions() {
        let manager = SessionManager::new();
        let config = SessionConfig::new("test".to_string());

        let id = manager
            .create_session("test-session".to_string(), SessionType::Local, config)
            .await
            .unwrap();

        // Terminate session (marks as Stopped and removes from map)
        manager.terminate_session(id).await.unwrap();

        // Cleanup should not panic (session already removed)
        manager.cleanup_dead_sessions().await;
        assert_eq!(manager.count_sessions().await, 0);
    }
}
