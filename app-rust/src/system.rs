// Platform-specific system utilities for HeartIO
use anyhow::{Context, Result};
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

// Global state for cleanup on exit
static ATEXIT_REGISTERED: AtomicBool = AtomicBool::new(false);
lazy_static::lazy_static! {
    static ref CAFFEINATE_PID: Arc<Mutex<Option<u32>>> = Arc::new(Mutex::new(None));
}

#[cfg(target_os = "macos")]
fn cleanup_caffeinate() {
    if let Ok(pid_guard) = CAFFEINATE_PID.lock() {
        if let Some(pid) = *pid_guard {
            tracing::info!("Emergency cleanup: terminating caffeinate process {}", pid);
            // Try to kill the process using system kill command
            let _ = Command::new("kill")
                .arg("-TERM")
                .arg(pid.to_string())
                .output();
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
                let child = Command::new("caffeinate")
                    .arg("-d") // Prevent display sleep
                    .spawn()
                    .context("Failed to start caffeinate command")?;

                let pid = child.id();
                tracing::info!("System sleep prevention activated (caffeinate PID: {})", pid);
                
                // Store PID for emergency cleanup
                if let Ok(mut pid_guard) = CAFFEINATE_PID.lock() {
                    *pid_guard = Some(pid);
                }
                
                // Register atexit handler only once
                if !ATEXIT_REGISTERED.load(Ordering::Relaxed) {
                    std::panic::set_hook(Box::new(|_| {
                        cleanup_caffeinate();
                    }));
                    ATEXIT_REGISTERED.store(true, Ordering::Relaxed);
                }
                
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
            if let Some(mut child) = self.caffeinate_process.take() {
                let pid = child.id();
                
                // Clear the stored PID
                if let Ok(mut pid_guard) = CAFFEINATE_PID.lock() {
                    *pid_guard = None;
                }
                
                // Try to terminate gracefully first
                if let Err(e) = child.kill() {
                    tracing::warn!("Failed to send kill signal to caffeinate process: {}", e);
                    // Still try to wait for it
                } else {
                    tracing::debug!("Sent termination signal to caffeinate process (PID: {})", pid);
                }
                
                // Wait for the process to exit (with timeout handling)
                match child.wait() {
                    Ok(status) => {
                        tracing::info!("System sleep prevention deactivated (exit status: {})", status);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to wait for caffeinate process: {}", e);
                        // Process might already be dead, which is fine
                    }
                }
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
