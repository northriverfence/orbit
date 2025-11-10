//! Terminal session management

use crate::pty::{PtyConfig, PtyHandle};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub id: Uuid,
    pub name: String,
    pub pty_config: PtyConfig,
}

impl SessionConfig {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            pty_config: PtyConfig::default(),
        }
    }
}

pub struct TerminalSession {
    config: SessionConfig,
    pty: PtyHandle,
}

impl TerminalSession {
    pub fn new(config: SessionConfig) -> Result<Self> {
        let pty = PtyHandle::new(config.pty_config.clone())?;
        Ok(Self { config, pty })
    }

    pub fn id(&self) -> &Uuid {
        &self.config.id
    }

    pub fn name(&self) -> &str {
        &self.config.name
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.pty.resize(cols, rows)
    }

    /// Write data to the PTY (send input)
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.pty.write(data)
    }

    /// Read data from the PTY (get output)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.pty.read(buf)
    }

    /// Try to read without blocking
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.pty.try_read(buf)
    }
}
