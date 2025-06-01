// Platform-specific system utilities for HeartIO
use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Global state for cleanup on exit
static ATEXIT_REGISTERED: AtomicBool = AtomicBool::new(false);
static CAFFEINATE_PID: AtomicU32 = AtomicU32::new(0);

#[cfg(target_os = "macos")]
fn cleanup_caffeinate() {
    let pid = CAFFEINATE_PID.load(Ordering::Relaxed);
    if pid != 0 {
        tracing::info!("Emergency cleanup: terminating caffeinate process {}", pid);
        // Use synchronous cleanup - no async allowed in exit handlers
        let _ = Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output();
        
        // Give it a moment, then force kill if needed
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = Command::new("kill")
            .arg("-KILL")
            .arg(pid.to_string())
            .output();
            
        CAFFEINATE_PID.store(0, Ordering::Relaxed);
    }
}

#[cfg(not(target_os = "macos"))]
fn cleanup_caffeinate() {
    // No-op on non-macOS platforms
}

// Register exit handlers to ensure cleanup happens
fn register_exit_handlers() {
    if ATEXIT_REGISTERED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
        // Register panic hook for emergency cleanup
        std::panic::set_hook(Box::new(|_| {
            cleanup_caffeinate();
        }));
        
        // Register atexit handler for normal program termination
        extern "C" fn exit_handler() {
            cleanup_caffeinate();
        }
        
        unsafe {
            libc::atexit(exit_handler);
        }
        
        #[cfg(target_os = "macos")]
        {
            // Register signal handlers for immediate termination
            extern "C" fn signal_handler(_: i32) {
                cleanup_caffeinate();
                std::process::exit(0);
            }
            
            unsafe {
                libc::signal(libc::SIGTERM, signal_handler as libc::sighandler_t);
                libc::signal(libc::SIGINT, signal_handler as libc::sighandler_t);
                libc::signal(libc::SIGQUIT, signal_handler as libc::sighandler_t);
            }
        }
    }
}

pub struct SystemUtils {
    #[cfg(target_os = "macos")]
    caffeinate_process: Option<Child>,
}

impl SystemUtils {
    /// Create a new SystemUtils instance
    pub fn new() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            caffeinate_process: None,
        }
    }

    /// Keep system awake (prevent sleep)
    pub fn keep_system_awake(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            if self.caffeinate_process.is_none() {
                // Register exit handlers before starting caffeinate
                register_exit_handlers();
                
                let child = Command::new("caffeinate")
                    .arg("-d") // Prevent display sleep
                    .spawn()
                    .context("Failed to start caffeinate command")?;

                let pid = child.id();
                tracing::info!("System sleep prevention activated (caffeinate PID: {})", pid);
                
                // Store PID for emergency cleanup using atomic operations (safe for signal handlers)
                CAFFEINATE_PID.store(pid, Ordering::Relaxed);
                
                self.caffeinate_process = Some(child);
            } else {
                tracing::debug!("Caffeinate process already running");
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            tracing::debug!("System sleep prevention not implemented for this platform");
        }

        Ok(())
    }

    /// Allow system to sleep again
    pub fn allow_system_sleep(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            let pid = CAFFEINATE_PID.load(Ordering::Relaxed);
            
            if let Some(mut child) = self.caffeinate_process.take() {
                let child_pid = child.id();
                
                // Clear the stored PID first
                CAFFEINATE_PID.store(0, Ordering::Relaxed);
                
                // Try to terminate gracefully first
                if let Err(e) = child.kill() {
                    tracing::warn!("Failed to send kill signal to caffeinate process: {}", e);
                    // If child.kill() fails, try system kill command
                    let _ = Command::new("kill")
                        .arg("-TERM")
                        .arg(child_pid.to_string())
                        .output();
                } else {
                    tracing::debug!("Sent termination signal to caffeinate process (PID: {})", child_pid);
                }
                
                // Wait for the process to exit (with timeout handling)
                match child.wait() {
                    Ok(status) => {
                        tracing::info!("System sleep prevention deactivated (exit status: {})", status);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to wait for caffeinate process: {}", e);
                        // Force kill if graceful shutdown failed
                        let _ = Command::new("kill")
                            .arg("-KILL")
                            .arg(child_pid.to_string())
                            .output();
                    }
                }
            } else if pid != 0 {
                // Process exists but not tracked locally, clean it up
                tracing::info!("Cleaning up orphaned caffeinate process {}", pid);
                CAFFEINATE_PID.store(0, Ordering::Relaxed);
                let _ = Command::new("kill")
                    .arg("-TERM")
                    .arg(pid.to_string())
                    .output();
            } else {
                tracing::debug!("No caffeinate process to terminate");
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            // No action needed on non-macOS platforms
        }

        Ok(())
    }
    
    /// Immediate synchronous cleanup for emergency shutdown
    pub fn immediate_cleanup() {
        cleanup_caffeinate();
    }

    /// Get platform information
    pub fn get_platform_info() -> String {
        let os = std::env::consts::OS;
        let arch = std::env::consts::ARCH;
        format!("{}-{}", os, arch)
    }
}

impl Drop for SystemUtils {
    fn drop(&mut self) {
        if let Err(e) = self.allow_system_sleep() {
            tracing::error!("Failed to restore system sleep settings: {}", e);
        }
    }
}
