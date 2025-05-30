// Platform-specific system utilities for HeartIO
use anyhow::{Context, Result};
use std::process::{Child, Command};

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

                self.caffeinate_process = Some(child);
                tracing::info!("System sleep prevention activated (caffeinate)");
            }
        }

        Ok(())
    }

    /// Allow system to sleep again
    pub fn allow_system_sleep(&mut self) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            if let Some(mut child) = self.caffeinate_process.take() {
                child.kill().context("Failed to kill caffeinate process")?;
                child
                    .wait()
                    .context("Failed to wait for caffeinate process")?;
                tracing::info!("System sleep prevention deactivated");
            }
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
