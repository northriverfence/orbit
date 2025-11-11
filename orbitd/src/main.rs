use anyhow::Result;
use tokio::signal;
use tracing::{error, info};

mod classifier;
mod config;
mod context;
mod credentials;
mod daemon;
mod embeddings;
mod executor;
mod learning;
mod license;
mod monitor;
mod prompts;
mod providers;

use crate::config::Config;
use crate::daemon::Daemon;
use crate::license::LicenseManager;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("🛸 Orbit Daemon starting...");

    // Load configuration
    let config = Config::load().await?;
    info!("Configuration loaded");

    // Validate license (CRITICAL - must pass before any operation)
    if !config.is_development_mode() {
        info!("Validating license...");
        let license_manager = LicenseManager::new(&config)?;

        if let Err(e) = license_manager.validate().await {
            error!("License validation failed: {}", e);
            eprintln!("❌ License validation failed: {}", e);
            eprintln!("For development mode, set ORBIT_DEV_MODE=1");
            eprintln!("Visit: https://singulio.com/orbit/licensing");
            std::process::exit(1);
        }

        info!("✓ License valid");
    } else {
        info!("Running in development mode - license validation skipped");
    }

    // Initialize daemon
    let mut daemon = Daemon::new(config).await?;
    info!("Daemon initialized");

    // Start daemon
    daemon.start().await?;
    info!("✓ Orbit Daemon started (PID: {})", std::process::id());

    // Wait for shutdown signal
    tokio::select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down...");
        }
        _ = wait_for_term_signal() => {
            info!("Received TERM signal, shutting down...");
        }
    }

    // Graceful shutdown
    daemon.stop().await?;
    info!("Daemon stopped gracefully");

    Ok(())
}

#[cfg(unix)]
async fn wait_for_term_signal() {
    use futures::stream::StreamExt;
    use signal_hook::consts::SIGTERM;
    use signal_hook_tokio::Signals;

    let mut signals = Signals::new([SIGTERM]).expect("Failed to create signal handler");

    signals.next().await;
}

#[cfg(not(unix))]
async fn wait_for_term_signal() {
    // On non-Unix systems, just wait forever
    std::future::pending::<()>().await
}
