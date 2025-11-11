// Windows Service Integration
//
// This module provides Windows service wrapper for the Orbit daemon,
// allowing it to run as a system service with automatic startup,
// proper lifecycle management, and Windows Event Log integration.

#[cfg(windows)]
pub mod windows_service;

#[cfg(windows)]
pub use windows_service::run_service;

#[cfg(not(windows))]
pub fn run_service() -> Result<(), Box<dyn std::error::Error>> {
    Err("Windows service support is only available on Windows".into())
}
