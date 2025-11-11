pub mod ipc;
pub mod ipc_common;

#[cfg(unix)]
pub mod ipc_unix;

#[cfg(windows)]
pub mod ipc_windows;

pub mod server;

use crate::classifier::CommandClassifier;
use crate::config::Config;
use crate::context::ContextEngine;
use crate::executor::Executor;
use crate::learning::LearningEngine;
use crate::license::LicenseManager;
use crate::monitor::ProactiveMonitor;
use crate::providers::ProviderRouter;
use anyhow::Result;
use std::sync::Arc;

pub use server::Server;

pub struct Daemon {
    config: Arc<Config>,
    server: Server,
    #[allow(dead_code)]
    classifier: Arc<CommandClassifier>,
    #[allow(dead_code)]
    provider_router: Arc<ProviderRouter>,
    #[allow(dead_code)]
    learning_engine: Arc<LearningEngine>,
    #[allow(dead_code)]
    context_engine: Arc<ContextEngine>,
    #[allow(dead_code)]
    executor: Arc<Executor>,
    monitor: Option<ProactiveMonitor>,
    license_manager: Option<LicenseManager>,
}

impl Daemon {
    pub async fn new(config: Config) -> Result<Self> {
        let config = Arc::new(config);

        // Initialize components
        let learning_engine = Arc::new(LearningEngine::new(config.clone()).await?);

        let classifier =
            Arc::new(CommandClassifier::new(config.clone(), learning_engine.clone()).await?);

        let provider_router = Arc::new(ProviderRouter::new(config.clone()).await?);

        let context_engine = Arc::new(ContextEngine::new(config.clone()).await?);

        let executor = Arc::new(Executor::new(config.clone()).await?);

        // Initialize monitor if enabled
        let monitor = if config.monitoring.enabled {
            Some(ProactiveMonitor::new(config.clone(), learning_engine.clone()).await?)
        } else {
            None
        };

        // Initialize license manager if not in dev mode
        let license_manager = if !config.is_development_mode() {
            Some(LicenseManager::new(&config)?)
        } else {
            None
        };

        // Create Unix socket server
        let server = Server::new(
            config.clone(),
            classifier.clone(),
            provider_router.clone(),
            learning_engine.clone(),
            context_engine.clone(),
            executor.clone(),
        )?;

        Ok(Self {
            config,
            server,
            classifier,
            provider_router,
            learning_engine,
            context_engine,
            executor,
            monitor,
            license_manager,
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        // Start license validation task if applicable
        if let Some(license_manager) = &self.license_manager {
            let lm = license_manager.clone();
            let interval_hours = self.config.license.validation_interval_hours;

            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(interval_hours * 3600))
                        .await;

                    if let Err(e) = lm.validate().await {
                        tracing::error!("License re-validation failed: {}", e);
                        eprintln!("❌ License validation failed. Orbit will stop.");
                        std::process::exit(1);
                    }

                    tracing::info!("✓ License re-validated");
                }
            });
        }

        // Start proactive monitor if enabled
        if let Some(monitor) = &self.monitor {
            let mon = monitor.clone();
            tokio::spawn(async move {
                if let Err(e) = mon.run().await {
                    tracing::error!("Monitor error: {}", e);
                }
            });
        }

        // Start Unix socket server
        self.server.start().await?;

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.server.stop().await?;
        Ok(())
    }
}
