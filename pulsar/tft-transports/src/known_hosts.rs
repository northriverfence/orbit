//! SSH known_hosts management for host key verification
//!
//! This module handles:
//! - Loading and parsing known_hosts file
//! - Verifying server host keys
//! - Adding new host keys
//! - Updating changed host keys (with user confirmation)

use anyhow::{Context, Result};
use russh::keys::{PublicKey, HashAlg};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Result of host key verification
#[derive(Debug, Clone, PartialEq)]
pub enum HostKeyVerification {
    /// Host key matches known_hosts
    Trusted,
    /// Host key not found in known_hosts (new host)
    Unknown,
    /// Host key changed (potential MITM attack)
    Changed { old_key: String },
}

/// Known hosts manager
pub struct KnownHosts {
    path: PathBuf,
    hosts: HashMap<String, PublicKey>,
}

impl KnownHosts {
    /// Load known_hosts from standard location (~/.ssh/known_hosts)
    pub fn load() -> Result<Self> {
        let path = Self::default_path()?;
        Self::load_from(&path)
    }

    /// Load known_hosts from specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        let mut hosts = HashMap::new();

        if path.exists() {
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read known_hosts from {}", path.display()))?;

            for line in content.lines() {
                let line = line.trim();

                // Skip comments and empty lines
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Parse line: hostname[,hostname2,...] keytype base64key [comment]
                if let Some((hostname_part, key_part)) = line.split_once(' ') {
                    // Handle comma-separated hostnames
                    let hostnames: Vec<&str> = hostname_part.split(',').collect();

                    // Parse the public key
                    if let Ok(public_key) = PublicKey::from_openssh(key_part) {
                        for hostname in hostnames {
                            // Remove hashed hosts (starting with |1|)
                            if hostname.starts_with('|') {
                                continue;
                            }

                            hosts.insert(hostname.to_string(), public_key.clone());
                        }
                    }
                }
            }

            tracing::info!("Loaded {} known hosts from {}", hosts.len(), path.display());
        } else {
            tracing::info!("known_hosts file not found at {}, will create on first use", path.display());
        }

        Ok(Self {
            path: path.to_path_buf(),
            hosts,
        })
    }

    /// Get default known_hosts path (~/.ssh/known_hosts)
    fn default_path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Failed to determine home directory")?;
        Ok(home.join(".ssh").join("known_hosts"))
    }

    /// Verify a host key
    pub fn verify(&self, hostname: &str, port: u16, key: &PublicKey) -> HostKeyVerification {
        // Check both "hostname" and "hostname:port" formats
        let host_key = format!("{}:{}", hostname, port);

        // Try hostname:port first
        if let Some(known_key) = self.hosts.get(&host_key) {
            if known_key == key {
                return HostKeyVerification::Trusted;
            } else {
                return HostKeyVerification::Changed {
                    old_key: known_key.to_openssh().unwrap_or_default(),
                };
            }
        }

        // Try hostname only (standard port 22)
        if let Some(known_key) = self.hosts.get(hostname) {
            if known_key == key {
                return HostKeyVerification::Trusted;
            } else {
                return HostKeyVerification::Changed {
                    old_key: known_key.to_openssh().unwrap_or_default(),
                };
            }
        }

        HostKeyVerification::Unknown
    }

    /// Add a host key to known_hosts
    pub fn add(&mut self, hostname: &str, port: u16, key: &PublicKey) -> Result<()> {
        let host_entry = if port == 22 {
            hostname.to_string()
        } else {
            format!("{}:{}", hostname, port)
        };

        self.hosts.insert(host_entry.clone(), key.clone());
        self.save()?;

        tracing::info!("Added host key for {} to known_hosts", host_entry);
        Ok(())
    }

    /// Update a changed host key
    pub fn update(&mut self, hostname: &str, port: u16, key: &PublicKey) -> Result<()> {
        let host_entry = if port == 22 {
            hostname.to_string()
        } else {
            format!("{}:{}", hostname, port)
        };

        tracing::warn!("Updating host key for {} (key changed!)", host_entry);
        self.hosts.insert(host_entry.clone(), key.clone());
        self.save()?;

        Ok(())
    }

    /// Remove a host key
    pub fn remove(&mut self, hostname: &str, port: u16) -> Result<()> {
        let host_entry = if port == 22 {
            hostname.to_string()
        } else {
            format!("{}:{}", hostname, port)
        };

        self.hosts.remove(&host_entry);
        self.save()?;

        tracing::info!("Removed host key for {} from known_hosts", host_entry);
        Ok(())
    }

    /// Save known_hosts to file
    fn save(&self) -> Result<()> {
        // Ensure .ssh directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory {}", parent.display()))?;
        }

        let mut lines = Vec::new();

        for (hostname, key) in &self.hosts {
            let key_str = key.to_openssh()
                .context("Failed to convert key to OpenSSH format")?;
            lines.push(format!("{} {}", hostname, key_str));
        }

        lines.sort(); // Keep file sorted for readability

        let content = lines.join("\n") + "\n";
        fs::write(&self.path, content)
            .with_context(|| format!("Failed to write known_hosts to {}", self.path.display()))?;

        tracing::debug!("Saved {} host keys to {}", self.hosts.len(), self.path.display());
        Ok(())
    }

    /// Get fingerprint of a public key (SHA256)
    pub fn fingerprint(key: &PublicKey) -> String {
        key.fingerprint(HashAlg::Sha256).to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_known_hosts_load_save() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        // Create empty known_hosts
        let mut known_hosts = KnownHosts::load_from(path).unwrap();
        assert_eq!(known_hosts.hosts.len(), 0);

        // Add a dummy entry (we can't easily create a real PublicKey in tests,
        // so this test would need actual SSH keys to be comprehensive)
    }
}
