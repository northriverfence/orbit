// Windows Service Implementation for Orbit Daemon
//
// This module implements Windows service control handlers, lifecycle management,
// and event log integration for running Orbit as a Windows service.

#[cfg(windows)]
use std::ffi::OsString;
#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use std::time::Duration;

#[cfg(windows)]
use windows_service::{
    define_windows_service,
    service::{
        ServiceControl, ServiceControlAccept, ServiceExitCode, ServiceState, ServiceStatus,
        ServiceType,
    },
    service_control_handler::{self, ServiceControlHandlerResult},
    service_dispatcher,
};

#[cfg(windows)]
use tracing::{error, info, warn};

// Service name registered with Windows
const SERVICE_NAME: &str = "OrbitDaemon";
const SERVICE_DISPLAY_NAME: &str = "Orbit AI Terminal Daemon";
const SERVICE_DESCRIPTION: &str = "AI-powered shell assistant daemon providing intelligent command suggestions and natural language interpretation";

// Service state machine
#[cfg(windows)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DaemonState {
    Stopped,
    Starting,
    Running,
    Stopping,
}

// Global service state (protected by mutex)
#[cfg(windows)]
static SERVICE_STATE: Mutex<Option<Arc<Mutex<DaemonState>>>> = Mutex::new(None);

/// Main entry point for Windows service
///
/// This function is called by the Windows Service Control Manager (SCM)
/// when the service is started. It sets up the service control handler
/// and runs the main daemon loop.
#[cfg(windows)]
pub fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    // Register service with Windows SCM
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}

/// Windows service main function (FFI wrapper)
///
/// This is called by the Windows Service Control Manager.
/// It's wrapped by define_windows_service! macro to handle FFI.
#[cfg(windows)]
define_windows_service!(ffi_service_main, service_main);

/// Service main function
///
/// This is the actual entry point after FFI wrapping.
/// It sets up the control handler and runs the daemon.
#[cfg(windows)]
fn service_main(_arguments: Vec<OsString>) {
    if let Err(e) = run_service_impl() {
        error!("Service error: {}", e);
        // Log to Windows Event Log
        log_event(
            EventType::Error,
            &format!("Orbit service failed to start: {}", e),
        );
    }
}

/// Service implementation
///
/// Sets up the control handler, reports status to SCM, and runs the daemon.
#[cfg(windows)]
fn run_service_impl() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize service state
    let state = Arc::new(Mutex::new(DaemonState::Stopped));
    *SERVICE_STATE.lock().unwrap() = Some(state.clone());

    // Define service control handler
    let event_handler = move |control_event| -> ServiceControlHandlerResult {
        match control_event {
            ServiceControl::Stop => {
                info!("Service stop requested");
                log_event(EventType::Information, "Orbit service stopping");

                // Update state
                *state.lock().unwrap() = DaemonState::Stopping;

                // Signal daemon to stop (implementation in daemon module)
                // This will be called by the service control handler

                ServiceControlHandlerResult::NoError
            }

            ServiceControl::Interrogate => {
                // SCM is checking our status
                ServiceControlHandlerResult::NoError
            }

            ServiceControl::Shutdown => {
                info!("System shutdown requested");
                log_event(EventType::Information, "System shutdown - stopping Orbit");

                *state.lock().unwrap() = DaemonState::Stopping;

                ServiceControlHandlerResult::NoError
            }

            _ => {
                warn!("Unhandled service control: {:?}", control_event);
                ServiceControlHandlerResult::NotImplemented
            }
        }
    };

    // Register control handler with Windows SCM
    let status_handle = service_control_handler::register(SERVICE_NAME, event_handler)?;

    // Report initial status: Starting
    info!("Orbit service starting");
    log_event(EventType::Information, "Orbit service starting");

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::StartPending,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(5),
        process_id: None,
    })?;

    // Update state
    *state.lock().unwrap() = DaemonState::Starting;

    // Initialize daemon
    // Note: Actual daemon initialization happens here
    // For now, we'll use a placeholder

    // Report status: Running
    info!("Orbit service running");
    log_event(EventType::Information, "Orbit service started successfully");

    *state.lock().unwrap() = DaemonState::Running;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP | ServiceControlAccept::SHUTDOWN,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    // Main service loop
    // This would typically call into the daemon's main loop
    // For now, we'll use a simple loop that checks for stop signal

    loop {
        let current_state = *state.lock().unwrap();

        match current_state {
            DaemonState::Stopping => {
                info!("Service stopping");
                break;
            }
            DaemonState::Running => {
                // Daemon is running - this is where we'd call daemon.run()
                // Sleep briefly to avoid busy-waiting
                std::thread::sleep(Duration::from_millis(100));
            }
            _ => {
                error!("Unexpected service state: {:?}", current_state);
                break;
            }
        }
    }

    // Report status: Stopped
    info!("Orbit service stopped");
    log_event(EventType::Information, "Orbit service stopped");

    *state.lock().unwrap() = DaemonState::Stopped;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Stopped,
        controls_accepted: ServiceControlAccept::empty(),
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::default(),
        process_id: None,
    })?;

    Ok(())
}

// ============================================================================
// Windows Event Log Integration
// ============================================================================

/// Event types for Windows Event Log
#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
enum EventType {
    Error,
    Warning,
    Information,
}

/// Log event to Windows Event Log
///
/// This function writes events to the Windows Event Log, which can be viewed
/// in Event Viewer under Windows Logs > Application.
///
/// Note: This requires the service to be properly registered with the Event Log.
/// The installation script should handle this registration.
#[cfg(windows)]
fn log_event(event_type: EventType, message: &str) {
    // In a full implementation, this would use the Windows Event Log API
    // For now, we'll just log to tracing which can be configured to write to Event Log

    match event_type {
        EventType::Error => error!("[Event Log] {}", message),
        EventType::Warning => warn!("[Event Log] {}", message),
        EventType::Information => info!("[Event Log] {}", message),
    }

    // TODO: Implement proper Windows Event Log API calls
    // This would use RegisterEventSource, ReportEvent, etc.
    // See: https://docs.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-reporteventa
}

// ============================================================================
// Service Management Functions
// ============================================================================

/// Get current service state
#[cfg(windows)]
pub fn get_service_state() -> DaemonState {
    SERVICE_STATE
        .lock()
        .unwrap()
        .as_ref()
        .map(|state| *state.lock().unwrap())
        .unwrap_or(DaemonState::Stopped)
}

/// Check if service is running
#[cfg(windows)]
pub fn is_service_running() -> bool {
    matches!(get_service_state(), DaemonState::Running)
}

// ============================================================================
// Service Installation (via external tools)
// ============================================================================

/// Service installation information
///
/// This struct contains the parameters needed for service installation.
/// It should be used by external installation tools (PowerShell scripts, installers).
#[cfg(windows)]
pub struct ServiceInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub binary_path: String,
    pub start_type: ServiceStartType,
}

#[cfg(windows)]
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum ServiceStartType {
    Auto,      // Start automatically on boot
    Manual,    // Start manually
    Disabled,  // Cannot be started
}

#[cfg(windows)]
impl ServiceInfo {
    /// Create service info with default values
    pub fn default(binary_path: String) -> Self {
        Self {
            name: SERVICE_NAME.to_string(),
            display_name: SERVICE_DISPLAY_NAME.to_string(),
            description: SERVICE_DESCRIPTION.to_string(),
            binary_path,
            start_type: ServiceStartType::Auto,
        }
    }

    /// Get PowerShell command to install service
    pub fn get_install_command(&self) -> String {
        let start_type = match self.start_type {
            ServiceStartType::Auto => "Automatic",
            ServiceStartType::Manual => "Manual",
            ServiceStartType::Disabled => "Disabled",
        };

        format!(
            r#"New-Service -Name "{}" -BinaryPathName '{}' -DisplayName "{}" -Description "{}" -StartupType {}"#,
            self.name, self.binary_path, self.display_name, self.description, start_type
        )
    }

    /// Get PowerShell command to uninstall service
    pub fn get_uninstall_command(&self) -> String {
        format!(
            r#"Stop-Service -Name "{}" -ErrorAction SilentlyContinue; Remove-Service -Name "{}""#,
            self.name, self.name
        )
    }

    /// Get PowerShell command to start service
    pub fn get_start_command(&self) -> String {
        format!(r#"Start-Service -Name "{}""#, self.name)
    }

    /// Get PowerShell command to stop service
    pub fn get_stop_command(&self) -> String {
        format!(r#"Stop-Service -Name "{}""#, self.name)
    }

    /// Get PowerShell command to check service status
    pub fn get_status_command(&self) -> String {
        format!(r#"Get-Service -Name "{}""#, self.name)
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    #[test]
    fn test_service_info_creation() {
        let info = ServiceInfo::default("C:\\Program Files\\Orbit\\orbitd.exe".to_string());

        assert_eq!(info.name, SERVICE_NAME);
        assert_eq!(info.display_name, SERVICE_DISPLAY_NAME);
        assert_eq!(info.description, SERVICE_DESCRIPTION);
        assert!(info.binary_path.contains("orbitd.exe"));
    }

    #[test]
    fn test_install_command_generation() {
        let info = ServiceInfo::default("C:\\Program Files\\Orbit\\orbitd.exe".to_string());
        let cmd = info.get_install_command();

        assert!(cmd.contains("New-Service"));
        assert!(cmd.contains(&info.name));
        assert!(cmd.contains(&info.binary_path));
        assert!(cmd.contains("Automatic"));
    }

    #[test]
    fn test_uninstall_command_generation() {
        let info = ServiceInfo::default("C:\\test\\orbitd.exe".to_string());
        let cmd = info.get_uninstall_command();

        assert!(cmd.contains("Stop-Service"));
        assert!(cmd.contains("Remove-Service"));
        assert!(cmd.contains(&info.name));
    }

    #[test]
    fn test_service_state_default() {
        // Service should be stopped by default
        assert_eq!(get_service_state(), DaemonState::Stopped);
        assert!(!is_service_running());
    }
}
