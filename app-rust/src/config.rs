// Configuration management for HeartIO
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(rename = "OSC_HOST")]
    pub osc_host: String,
    #[serde(rename = "OSC_PORT")]
    pub osc_port: u16,
    #[serde(rename = "HEART_RATE_DEVICE_NAME")]
    pub heart_rate_device_name: Option<String>,
    #[serde(rename = "HEART_RATE_DEVICE_ADDRESS")]
    pub heart_rate_device_address: Option<String>,
    #[serde(rename = "APPLE_WATCH")]
    pub apple_watch: bool,
    #[serde(rename = "HEART_RATE_LABEL")]
    pub heart_rate_label: HashMap<String, Vec<String>>,
}

impl Default for Config {
    fn default() -> Self {
        let mut heart_rate_label = HashMap::new();
        
        // Default heart rate labels with configurable thresholds
        heart_rate_label.insert("70".to_string(), vec!["♡ {{bpm}}".to_string()]);
        heart_rate_label.insert("80".to_string(), vec!["❤️ {{bpm}}".to_string()]);
        heart_rate_label.insert("100".to_string(), vec!["💕 {{bpm}} 💕".to_string()]);
        heart_rate_label.insert("130".to_string(), vec!["❤️💕 {{bpm}} 💕❤️".to_string()]);
        heart_rate_label.insert("150".to_string(), vec![
            "❤️❤️❤️ {{bpm}} ❤️❤️❤️".to_string(),
            "💕💕💕 {{bpm}} 💕💕💕".to_string(),
        ]);
        heart_rate_label.insert("999".to_string(), vec![
            "❤️❤️❤️❤️ {{bpm}} ❤️❤️❤️❤️".to_string(),
            "💕💕💕💕 {{bpm}} 💕💕💕💕".to_string(),
            "LOVE ❤️ {{bpm}} ❤️ LOVE".to_string(),
        ]);

        Self {
            osc_host: "127.0.0.1".to_string(),
            osc_port: 9000,
            heart_rate_device_name: None,
            heart_rate_device_address: None,
            apple_watch: false,
            heart_rate_label,
        }
    }
}

impl Config {
    /// Get the path to the config file (same directory as executable)
    pub fn config_path() -> Result<PathBuf> {
        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;
        let exe_dir = exe_path.parent().context("Failed to get executable directory")?;
        Ok(exe_dir.join("heartio.config.json"))
    }

    /// Load configuration from heartio.config.json or create default if not exists
    pub async fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = tokio::fs::read_to_string(&config_path)
                .await
                .context("Failed to read config file")?;
            let config: Config = serde_json::from_str(&content)
                .context("Failed to parse config file")?;
            tracing::info!("Loaded configuration from {}", config_path.display());
            Ok(config)
        } else {
            let config = Self::default();
            config.save().await?;
            tracing::info!("Created default configuration at {}", config_path.display());
            Ok(config)
        }
    }

    /// Save configuration to heartio.config.json
    pub async fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        tokio::fs::write(&config_path, content)
            .await
            .context("Failed to write config file")?;
        tracing::info!("Saved configuration to {}", config_path.display());
        Ok(())
    }

    /// Get heart rate text based on BPM and configured thresholds
    pub fn get_heart_rate_text(&self, bpm: u32) -> Option<String> {
        // Find the appropriate threshold
        let thresholds: Vec<u32> = self.heart_rate_label.keys()
            .filter_map(|k| k.parse().ok())
            .collect();
        
        let mut sorted_thresholds = thresholds.clone();
        sorted_thresholds.sort();
        
        let threshold = sorted_thresholds.iter()
            .find(|&&t| bpm < t)
            .or_else(|| sorted_thresholds.last())?;
        
        let labels = self.heart_rate_label.get(&threshold.to_string())?;
        
        if labels.is_empty() {
            return None;
        }
        
        // Randomly select a label if multiple are available
        let label = if labels.len() == 1 {
            &labels[0]
        } else {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let index = rng.gen_range(0..labels.len());
            &labels[index]
        };
        
        Some(label.replace("{{bpm}}", &bpm.to_string()))
    }
}
