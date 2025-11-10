//! PTY (Pseudo-Terminal) management

use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, PtyPair, PtySize, MasterPty};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PtyConfig {
    pub cols: u16,
    pub rows: u16,
    pub shell: Option<String>,
}

impl Default for PtyConfig {
    fn default() -> Self {
        Self {
            cols: 80,
            rows: 24,
            shell: None,
        }
    }
}

/// Handle to a PTY (pseudo-terminal)
pub struct PtyHandle {
    master: Mutex<Box<dyn MasterPty + Send>>,
    reader: Mutex<Box<dyn Read + Send>>,
    writer: Mutex<Box<dyn Write + Send>>,
}

impl PtyHandle {
    /// Create a new PTY with the given configuration
    pub fn new(config: PtyConfig) -> Result<Self> {
        let pty_system = portable_pty::native_pty_system();

        // Create PTY
        let pair = pty_system
            .openpty(PtySize {
                rows: config.rows,
                cols: config.cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to open PTY")?;

        // Determine shell
        let shell = config.shell.unwrap_or_else(|| {
            std::env::var("SHELL").unwrap_or_else(|_| {
                #[cfg(target_os = "windows")]
                return "cmd.exe".to_string();
                #[cfg(not(target_os = "windows"))]
                return "/bin/sh".to_string();
            })
        });

        // Spawn shell in PTY
        let mut cmd = CommandBuilder::new(&shell);
        cmd.env("TERM", "xterm-256color");

        pair.slave
            .spawn_command(cmd)
            .context("Failed to spawn shell in PTY")?;

        // Extract reader and writer from master
        let mut master = pair.master;
        let reader = master
            .try_clone_reader()
            .context("Failed to clone PTY reader")?;
        let writer = master
            .take_writer()
            .context("Failed to take PTY writer")?;

        Ok(Self {
            master: Mutex::new(master),
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        })
    }

    /// Resize the PTY
    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock master PTY: {}", e))?
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("Failed to resize PTY")
    }

    /// Write data to PTY (send input)
    pub fn write(&mut self, data: &[u8]) -> Result<usize> {
        self.writer
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock writer: {}", e))?
            .write(data)
            .context("Failed to write to PTY")
    }

    /// Read data from PTY (get output)
    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader
            .lock()
            .map_err(|e| anyhow::anyhow!("Failed to lock reader: {}", e))?
            .read(buf)
            .context("Failed to read from PTY")
    }

    /// Try to read without blocking
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Use the same method as read() since portable-pty doesn't provide
        // a non-blocking read directly. The caller should handle this with timeouts.
        self.read(buf)
    }
}
