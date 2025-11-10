//! Application state management

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct AppState {
    pub sessions: Arc<RwLock<HashMap<Uuid, SessionInfo>>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

pub struct SessionInfo {
    pub id: Uuid,
    pub host: String,
    pub username: String,
}
