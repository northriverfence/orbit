// launchd implementation for macOS

use super::DaemonStatus;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const PLIST_NAME: &str = "com.singulio.orbitd.plist";

/// Get LaunchAgents directory
fn get_launchagents_dir() -> Result<PathBuf> {
    let home = dirs::home_dir()
        .context("Could not find home directory")?;
    Ok(home.join("Library").join("LaunchAgents"))
}

/// Get plist file path
fn get_plist_path() -> Result<PathBuf> {
    Ok(get_launchagents_dir()?.join(PLIST_NAME))
}

/// Generate launchd plist content
fn generate_plist(daemon_path: &Path) -> String {
    let home = dirs::home_dir()
        .and_then(|p| p.to_str().map(|s| s.to_string()))
        .unwrap_or_else(|| "~".to_string());

    let log_dir = format!("{}/Library/Logs", home);

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.singulio.orbitd</string>

    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>

    <key>StandardOutPath</key>
    <string>{}/orbitd.log</string>

    <key>StandardErrorPath</key>
    <string>{}/orbitd.error.log</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
        <key>ORBIT_CONFIG_DIR</key>
        <string>{}/Library/Application Support/orbit</string>
        <key>ORBIT_DATA_DIR</key>
        <string>{}/Library/Application Support/orbit</string>
    </dict>

    <key>ProcessType</key>
    <string>Background</string>

    <key>ThrottleInterval</key>
    <integer>5</integer>

    <key>Nice</key>
    <integer>0</integer>
</dict>
</plist>
"#,
        daemon_path.display(),
        log_dir,
        log_dir,
        home,
        home
    )
}

/// Check if launchd plist is installed
pub fn launchd_is_installed() -> Result<bool> {
    let plist_path = get_plist_path()?;
    Ok(plist_path.exists())
}

/// Check if launchd service is enabled (loaded)
pub fn launchd_is_enabled() -> Result<bool> {
    if !launchd_is_installed()? {
        return Ok(false);
    }

    let output = Command::new("launchctl")
        .args(["list", "com.singulio.orbitd"])
        .output()
        .context("Failed to check launchd service status")?;

    Ok(output.status.success())
}

/// Install launchd plist
pub fn launchd_install(daemon_path: &Path) -> Result<()> {
    // Create LaunchAgents directory if it doesn't exist
    let launchagents_dir = get_launchagents_dir()?;
    fs::create_dir_all(&launchagents_dir)
        .context("Failed to create LaunchAgents directory")?;

    // Create log directory
    let home = dirs::home_dir()
        .context("Could not find home directory")?;
    let log_dir = home.join("Library").join("Logs");
    fs::create_dir_all(&log_dir)
        .context("Failed to create log directory")?;

    // Write plist file
    let plist_path = get_plist_path()?;
    let plist_content = generate_plist(daemon_path);
    fs::write(&plist_path, plist_content)
        .context("Failed to write launchd plist file")?;

    tracing::info!("Installed launchd plist at {}", plist_path.display());
    Ok(())
}

/// Uninstall launchd plist
pub fn launchd_uninstall() -> Result<()> {
    // Unload first
    let _ = launchd_disable();

    // Remove plist file
    let plist_path = get_plist_path()?;
    if plist_path.exists() {
        fs::remove_file(&plist_path)
            .context("Failed to remove launchd plist file")?;
    }

    Ok(())
}

/// Enable (load) launchd service
pub fn launchd_enable() -> Result<()> {
    let plist_path = get_plist_path()?;

    let output = Command::new("launchctl")
        .args(["load", plist_path.to_str().unwrap()])
        .output()
        .context("Failed to load launchd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to load launchd service: {}", stderr);
    }

    Ok(())
}

/// Disable (unload) launchd service
pub fn launchd_disable() -> Result<()> {
    let plist_path = get_plist_path()?;

    let output = Command::new("launchctl")
        .args(["unload", plist_path.to_str().unwrap()])
        .output()
        .context("Failed to unload launchd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Don't fail if service is not loaded
        if !stderr.contains("Could not find specified service") {
            anyhow::bail!("Failed to unload launchd service: {}", stderr);
        }
    }

    Ok(())
}

/// Start launchd service
pub fn launchd_start() -> Result<()> {
    let output = Command::new("launchctl")
        .args(["start", "com.singulio.orbitd"])
        .output()
        .context("Failed to start launchd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to start launchd service: {}", stderr);
    }

    Ok(())
}

/// Stop launchd service
pub fn launchd_stop() -> Result<()> {
    let output = Command::new("launchctl")
        .args(["stop", "com.singulio.orbitd"])
        .output()
        .context("Failed to stop launchd service")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to stop launchd service: {}", stderr);
    }

    Ok(())
}

/// Get launchd service status
pub fn launchd_status() -> Result<DaemonStatus> {
    let installed = launchd_is_installed()?;
    let enabled = if installed {
        launchd_is_enabled().unwrap_or(false)
    } else {
        false
    };

    let running = if enabled {
        // Check if process is running via launchctl list
        let output = Command::new("launchctl")
            .args(["list", "com.singulio.orbitd"])
            .output()
            .context("Failed to check launchd service status")?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // If PID is present and not "-", service is running
            !stdout.contains("\"-\"")
        } else {
            false
        }
    } else {
        false
    };

    Ok(DaemonStatus {
        installed,
        enabled,
        running,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plist_generation() {
        let daemon_path = Path::new("/usr/local/bin/orbitd");
        let content = generate_plist(daemon_path);

        assert!(content.contains("<string>/usr/local/bin/orbitd</string>"));
        assert!(content.contains("<key>RunAtLoad</key>"));
        assert!(content.contains("<key>KeepAlive</key>"));
    }

    #[test]
    fn test_launchagents_dir() {
        let dir = get_launchagents_dir();
        assert!(dir.is_ok());
    }
}
