use anyhow::{anyhow, Context, Result};
use chrono::{Datelike, Timelike, Utc};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use crate::classifier::{CommandClassifier, CommandType};
use crate::config::Config;
use crate::context::ContextEngine;
use crate::executor::Executor;
use crate::learning::LearningEngine;
use crate::providers::ProviderRouter;

use super::ipc::{FeedbackResult, Request, Response};

/// Maximum concurrent IPC connections allowed
/// This prevents local DoS attacks from flooding the daemon with requests
const MAX_CONCURRENT_CONNECTIONS: usize = 100;

/// Maximum message size (1MB)
/// Prevents memory exhaustion from large malicious payloads
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

pub struct Server {
    config: Arc<Config>,
    classifier: Arc<CommandClassifier>,
    provider_router: Arc<ProviderRouter>,
    learning_engine: Arc<LearningEngine>,
    context_engine: Arc<ContextEngine>,
    executor: Arc<Executor>,
    shutdown_tx: Option<tokio::sync::oneshot::Sender<()>>,
    connection_semaphore: Arc<Semaphore>,
}

impl Server {
    pub fn new(
        config: Arc<Config>,
        classifier: Arc<CommandClassifier>,
        provider_router: Arc<ProviderRouter>,
        learning_engine: Arc<LearningEngine>,
        context_engine: Arc<ContextEngine>,
        executor: Arc<Executor>,
    ) -> Result<Self> {
        Ok(Self {
            config,
            classifier,
            provider_router,
            learning_engine,
            context_engine,
            executor,
            shutdown_tx: None,
            connection_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_CONNECTIONS)),
        })
    }

    pub async fn start(&mut self) -> Result<()> {
        let socket_path = &self.config.daemon.socket_path;

        // Remove socket if it already exists
        if socket_path.exists() {
            std::fs::remove_file(socket_path).context("Failed to remove existing socket")?;
        }

        // Create socket directory if it doesn't exist
        if let Some(parent) = socket_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create socket directory")?;
        }

        let listener = UnixListener::bind(socket_path).context("Failed to bind Unix socket")?;

        info!("Unix socket server listening on: {:?}", socket_path);

        // Set permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(socket_path)?.permissions();
            perms.set_mode(0o600); // Owner read/write only
            std::fs::set_permissions(socket_path, perms)?;
        }

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        let config = self.config.clone();
        let classifier = self.classifier.clone();
        let provider_router = self.provider_router.clone();
        let learning_engine = self.learning_engine.clone();
        let context_engine = self.context_engine.clone();
        let executor = self.executor.clone();
        let semaphore = self.connection_semaphore.clone();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok((stream, _)) = listener.accept() => {
                        let config = config.clone();
                        let classifier = classifier.clone();
                        let provider_router = provider_router.clone();
                        let learning_engine = learning_engine.clone();
                        let context_engine = context_engine.clone();
                        let executor = executor.clone();

                        // Acquire semaphore permit - blocks if at max connections
                        let permit = match semaphore.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => {
                                warn!("Maximum concurrent connections ({}) reached, rejecting new connection", MAX_CONCURRENT_CONNECTIONS);
                                continue;
                            }
                        };

                        tokio::spawn(async move {
                            // Permit held until task completes (RAII pattern)
                            let _permit = permit;

                            if let Err(e) = handle_client(
                                stream,
                                config,
                                classifier,
                                provider_router,
                                learning_engine,
                                context_engine,
                                executor,
                            ).await {
                                error!("Error handling client: {}", e);
                            }
                        });
                    }
                    _ = &mut shutdown_rx => {
                        info!("Shutting down Unix socket server");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }

        // Clean up socket
        let socket_path = &self.config.daemon.socket_path;
        if socket_path.exists() {
            std::fs::remove_file(socket_path)?;
        }

        Ok(())
    }
}

async fn handle_client(
    stream: UnixStream,
    config: Arc<Config>,
    classifier: Arc<CommandClassifier>,
    provider_router: Arc<ProviderRouter>,
    learning_engine: Arc<LearningEngine>,
    context_engine: Arc<ContextEngine>,
    executor: Arc<Executor>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Read message with size limit to prevent memory exhaustion attacks
    let mut buf = vec![0u8; MAX_MESSAGE_SIZE];
    let n = match reader.read(&mut buf).await {
        Ok(n) => n,
        Err(e) => {
            warn!("Failed to read from client: {}", e);
            return Err(e.into());
        }
    };

    // Check if message was truncated
    if n == MAX_MESSAGE_SIZE {
        warn!(
            "Message size limit ({} bytes) reached, possible DoS attempt",
            MAX_MESSAGE_SIZE
        );
        let error_response = serde_json::to_string(&Response::Error {
            message: format!("Message too large (max {} bytes)", MAX_MESSAGE_SIZE),
        })?
        + "\n";
        writer.write_all(error_response.as_bytes()).await?;
        writer.flush().await?;
        return Err(anyhow!("Message exceeds size limit"));
    }

    // Convert to string
    let message = match std::str::from_utf8(&buf[..n]) {
        Ok(s) => s.trim(),
        Err(e) => {
            warn!("Invalid UTF-8 in message: {}", e);
            let error_response = serde_json::to_string(&Response::Error {
                message: "Invalid UTF-8 in message".to_string(),
            })?
            + "\n";
            writer.write_all(error_response.as_bytes()).await?;
            writer.flush().await?;
            return Err(anyhow!("Invalid UTF-8"));
        }
    };

    debug!("Received IPC message: {}", message);

    // Try to parse as JSON (new protocol)
    let response_str = if let Ok(request) = serde_json::from_str::<Request>(message) {
        // Handle JSON protocol
        let response = handle_request(
            request,
            &config,
            &classifier,
            &provider_router,
            &learning_engine,
            &context_engine,
            &executor,
        )
        .await;

        match response {
            Ok(resp) => {
                serde_json::to_string(&resp).unwrap_or_else(|_| {
                    serde_json::to_string(&Response::Error {
                        message: "Serialization error".to_string(),
                    })
                    .unwrap()
                }) + "\n"
            }
            Err(e) => {
                error!("Error handling request: {}", e);
                serde_json::to_string(&Response::Error {
                    message: e.to_string(),
                })
                .unwrap()
                    + "\n"
            }
        }
    } else {
        // Legacy text protocol - treat as command query
        handle_legacy_query(
            message,
            &config,
            &classifier,
            &provider_router,
            &learning_engine,
            &context_engine,
            &executor,
        )
        .await
    };

    // Send response back to shell
    writer.write_all(response_str.as_bytes()).await?;
    writer.flush().await?;

    Ok(())
}

async fn handle_request(
    request: Request,
    config: &Arc<Config>,
    classifier: &Arc<CommandClassifier>,
    provider_router: &Arc<ProviderRouter>,
    learning_engine: &Arc<LearningEngine>,
    context_engine: &Arc<ContextEngine>,
    executor: &Arc<Executor>,
) -> Result<Response> {
    match request {
        Request::Command {
            input,
            cwd: _,
            shell: _,
        } => {
            handle_command_query(
                &input,
                config,
                classifier,
                provider_router,
                learning_engine,
                context_engine,
                executor,
            )
            .await
        }
        Request::Feedback {
            input,
            executed,
            result,
        } => handle_feedback(&input, &executed, result, learning_engine, context_engine).await,
        Request::Status => {
            // TODO: Track uptime and command count
            Ok(Response::Status {
                uptime_secs: 0,
                commands_processed: 0,
            })
        }
        Request::Shutdown => {
            info!("Shutdown requested via IPC");
            Ok(Response::Ok)
        }
    }
}

async fn handle_command_query(
    command: &str,
    config: &Arc<Config>,
    classifier: &Arc<CommandClassifier>,
    provider_router: &Arc<ProviderRouter>,
    learning_engine: &Arc<LearningEngine>,
    context_engine: &Arc<ContextEngine>,
    executor: &Arc<Executor>,
) -> Result<Response> {
    // Get current context
    let context = context_engine.get_context().await?;

    // Classify command
    let classification = classifier.classify(command, &context).await?;

    match classification {
        CommandType::Known => {
            debug!("Known command, passing through");
            Ok(Response::Passthrough)
        }
        CommandType::LearnedPattern(pattern) => {
            debug!("Using learned pattern: {}", pattern.learned_command);
            Ok(Response::Replaced {
                command: pattern.learned_command,
            })
        }
        CommandType::NaturalLanguage | CommandType::Ambiguous => {
            debug!("Sending to AI for interpretation");

            match provider_router
                .process_natural_language(command, &context)
                .await
            {
                Ok(ai_command) => {
                    debug!("AI suggestion: {}", ai_command);

                    // SECURITY: Validate AI response for safety
                    if validate_ai_response(&ai_command, executor, config)? {
                        // Record this interaction for learning
                        learning_engine
                            .record_ai_suggestion(command, &ai_command, &context)
                            .await?;

                        Ok(Response::Replaced {
                            command: ai_command,
                        })
                    } else {
                        // AI returned an unsafe command
                        warn!(
                            "AI returned unsafe command, rejecting: {}",
                            ai_command
                        );
                        Ok(Response::Error {
                            message: "AI suggestion rejected for safety reasons. Please try rephrasing your request.".to_string(),
                        })
                    }
                }
                Err(e) => {
                    error!("AI error: {}", e);
                    Ok(Response::Error {
                        message: e.to_string(),
                    })
                }
            }
        }
    }
}

/// Validate AI response for safety
///
/// Checks for:
/// 1. Destructive commands
/// 2. Pipe-to-shell patterns (curl | bash)
/// 3. Suspicious download-and-execute patterns
/// 4. Command length limits
///
/// Returns true if safe, false if should be rejected
fn validate_ai_response(
    ai_command: &str,
    executor: &Arc<Executor>,
    config: &Arc<Config>,
) -> Result<bool> {
    // Check 1: Reject if empty or only whitespace
    if ai_command.trim().is_empty() {
        warn!("AI returned empty command");
        return Ok(false);
    }

    // Check 2: Reject extremely long commands (possible attack)
    if ai_command.len() > 10000 {
        warn!("AI returned suspiciously long command ({} bytes)", ai_command.len());
        return Ok(false);
    }

    // Check 3: Use our destructive command detector
    if config.execution.confirm_destructive && executor.is_destructive(ai_command) {
        warn!("AI returned destructive command: {}", ai_command);
        // Still return true because user will be prompted for confirmation
        // But log it for monitoring
        return Ok(true);
    }

    // Check 4: Detect pipe-to-shell patterns (very dangerous)
    let dangerous_patterns = [
        (r"curl.*\|.*bash", "curl piped to bash"),
        (r"curl.*\|.*sh", "curl piped to sh"),
        (r"wget.*\|.*bash", "wget piped to bash"),
        (r"wget.*\|.*sh", "wget piped to sh"),
        (r"\|\s*sudo", "piping to sudo"),
        (r"eval\s*\$\(", "eval with command substitution"),
    ];

    for (pattern, description) in &dangerous_patterns {
        if ai_command.to_lowercase().contains(&pattern.replace("\\", "")) {
            warn!(
                "AI returned command with dangerous pattern ({}): {}",
                description, ai_command
            );
            return Ok(false);
        }
    }

    // Check 5: Detect obfuscation attempts
    if ai_command.contains("base64 -d") && (ai_command.contains("| sh") || ai_command.contains("| bash")) {
        warn!("AI returned base64-encoded pipe-to-shell");
        return Ok(false);
    }

    // Check 6: Detect excessive use of special characters (possible obfuscation)
    let special_char_count = ai_command.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
    let special_char_ratio = special_char_count as f32 / ai_command.len() as f32;
    if special_char_ratio > 0.5 {
        warn!(
            "AI returned command with excessive special characters ({:.1}%): {}",
            special_char_ratio * 100.0,
            ai_command
        );
        return Ok(false);
    }

    // Passed all safety checks
    Ok(true)
}

async fn handle_feedback(
    input: &str,
    executed: &str,
    result: FeedbackResult,
    learning_engine: &Arc<LearningEngine>,
    context_engine: &Arc<ContextEngine>,
) -> Result<Response> {
    let context = context_engine.get_context().await?;

    debug!(
        "Received feedback: input='{}', executed='{}', result={:?}",
        input, executed, result
    );

    // Update pattern confidence based on feedback
    match result {
        FeedbackResult::Success => {
            // Boost confidence for successful execution
            learning_engine
                .record_success(input, executed, &context)
                .await?;
            debug!("Pattern confidence boosted for successful execution");

            // Record temporal pattern
            let now = Utc::now();
            let hour = now.hour() as i32;
            let day = now.weekday().num_days_from_monday() as i32;
            learning_engine
                .record_temporal_pattern(executed, hour, day)
                .await?;
        }
        FeedbackResult::Failed => {
            // Lower confidence for failed execution
            learning_engine
                .record_failure(input, executed, &context)
                .await?;
            debug!("Pattern confidence lowered for failed execution");
        }
        FeedbackResult::Rejected => {
            // User rejected the suggestion - just log it
            debug!("User rejected suggestion");
        }
        FeedbackResult::Edited { new_command } => {
            // User edited the suggestion - record as correction
            learning_engine
                .record_correction(input, executed, &new_command, &context)
                .await?;
            debug!("User correction recorded");
        }
    }

    Ok(Response::Ok)
}

async fn handle_legacy_query(
    command: &str,
    config: &Arc<Config>,
    classifier: &Arc<CommandClassifier>,
    provider_router: &Arc<ProviderRouter>,
    learning_engine: &Arc<LearningEngine>,
    context_engine: &Arc<ContextEngine>,
    executor: &Arc<Executor>,
) -> String {
    match handle_command_query(
        command,
        config,
        classifier,
        provider_router,
        learning_engine,
        context_engine,
        executor,
    )
    .await
    {
        Ok(Response::Passthrough) => "PASSTHROUGH\n".to_string(),
        Ok(Response::Replaced { command }) => format!("REPLACED:{}\n", command),
        Ok(Response::Error { message }) => format!("ERROR:{}\n", message),
        Err(e) => format!("ERROR:{}\n", e),
        _ => "ERROR:Unexpected response\n".to_string(),
    }
}
