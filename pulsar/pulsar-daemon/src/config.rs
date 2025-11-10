//! Daemon configuration

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
    pub socket_path: PathBuf,
    pub database_path: PathBuf,
    pub log_level: String,
    pub websocket_port: u16,
    pub grpc_port: u16,
    pub webtransport_port: u16,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .expect("Could not find config directory")
            .join("orbit");

        Self {
            socket_path: config_dir.join("pulsar.sock"),
            database_path: config_dir.join("pulsar.db"),
            log_level: "info".to_string(),
            websocket_port: 3030,
            grpc_port: 50051,
            webtransport_port: 4433,
        }
    }
}

impl DaemonConfig {
    pub fn load() -> Result<Self> {
        // TODO: Load from file
        Ok(Self::default())
    }

    pub fn save(&self) -> Result<()> {
        // TODO: Save to file
        Ok(())
    }
}
