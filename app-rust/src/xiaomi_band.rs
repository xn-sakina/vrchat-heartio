// Xiaomi Band heart rate monitoring via BLE advertisements
use anyhow::{Context, Result};
use btleplug::api::{Central, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::sleep;

/// Xiaomi Band advertisement monitor for heart rate data
pub struct XiaomiBandMonitor {
    adapter: Adapter,
    last_seen: HashMap<String, Instant>,
    heart_rate_sender: mpsc::UnboundedSender<u32>,
    running: bool,
    device_addr: Option<String>,
}

impl XiaomiBandMonitor {
    /// Create a new Xiaomi Band monitor
    pub async fn new(heart_rate_sender: mpsc::UnboundedSender<u32>) -> Result<Self> {
        let manager = Manager::new()
            .await
            .context("Failed to create Bluetooth manager")?;

        let adapters = manager
            .adapters()
            .await
            .context("Failed to get Bluetooth adapters")?;

        let adapter = adapters
            .into_iter()
            .next()
            .context("No Bluetooth adapter found")?;

        Ok(Self {
            adapter,
            last_seen: HashMap::new(),
            heart_rate_sender,
            running: false,
            device_addr: None,
        })
    }

    /// Check if Bluetooth is available
    pub async fn check_bluetooth_availability(&self) -> Result<bool> {
        match self.adapter.start_scan(ScanFilter::default()).await {
            Ok(_) => {
                // Stop the scan immediately after starting
                let _ = self.adapter.stop_scan().await;
                Ok(true)
            }
            Err(e) => {
                tracing::error!("Bluetooth error: {}", e);
                Ok(false)
            }
        }
    }

    /// Start monitoring for Xiaomi Band advertisements
    pub async fn start_monitoring(&mut self) -> Result<()> {
        if !self.check_bluetooth_availability().await? {
            return Err(anyhow::anyhow!(
                "Bluetooth is not available or disabled. Please enable Bluetooth and try again."
            ));
        }

        tracing::info!("Starting Xiaomi Band advertisement monitoring...");
        self.running = true;

        // Start scanning for BLE advertisements
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .context("Failed to start BLE scan")?;

        tracing::info!("Scanner started. Waiting for Xiaomi band advertisements...");

        // Get the event stream
        let mut events = self.adapter.events().await?;

        // Process advertisements
        while self.running {
            tokio::select! {
                event = events.next() => {
                    if let Some(event) = event {
                        if self.device_addr.is_some() {
                            let addr = self.device_addr.as_ref().unwrap();
                            if let btleplug::api::CentralEvent::ManufacturerDataAdvertisement { id, manufacturer_data } = &event {
                                if id.to_string() == *addr {
                                    // send bpm
                                    self.send_bpm(&manufacturer_data, id.to_string().as_str()).await;
                                }
                            }
                        } else {
                            if let btleplug::api::CentralEvent::DeviceUpdated(id) = event {
                                if let Ok(peripheral) = self.adapter.peripheral(&id).await {
                                    self.handle_advertisement(&peripheral).await;
                                }
                            }
                        }

                    }
                }
                _ = sleep(Duration::from_millis(100)) => {
                    // Continue processing
                }
            }
        }

        // Stop scanning
        let _ = self.adapter.stop_scan().await;
        tracing::info!("Xiaomi Band monitoring stopped");
        Ok(())
    }

    /// Handle a BLE advertisement
    async fn handle_advertisement(&mut self, peripheral: &impl btleplug::api::Peripheral) {
        let now = Instant::now();
        let addr = peripheral.address().to_string();

        // Rate limiting - only process each device once per second
        if let Some(last_time) = self.last_seen.get(&addr) {
            if now.duration_since(*last_time) < Duration::from_secs(1) {
                return;
            }
        }
        self.last_seen.insert(addr.clone(), now);

        // Get device properties
        if let Ok(properties) = peripheral.properties().await {
            if let Some(properties) = properties {
                let name = properties.local_name.unwrap_or_default();

                // Check if this is a Xiaomi Smart Band
                if name.contains("Xiaomi Smart Band") {
                    // Get manufacturer data
                    let manufacturer_data = properties.manufacturer_data;
                    // send heart rate data if available
                    self.send_bpm(&manufacturer_data, addr.as_str()).await;
                }
            }
        }
    }

    pub async fn send_bpm(&mut self, manufacturer_data: &HashMap<u16, Vec<u8>>, addr: &str) {
        for (_, value) in manufacturer_data.iter() {
            if value.len() >= 4 {
                let heart_rate = value[3] as u32;
                if heart_rate > 0 && heart_rate < 300 {
                    // save device address if not already set
                    if self.device_addr.is_none() {
                        self.device_addr = Some(addr.to_string());
                        tracing::info!("Detected Xiaomi Band at address: {}", addr);
                    }

                    tracing::info!("[{}] Received heart rate: {} bpm", addr, heart_rate);

                    // Send heart rate to the channel
                    if let Err(e) = self.heart_rate_sender.send(heart_rate) {
                        tracing::error!("Failed to send heart rate: {}", e);
                    }
                }
            } else {
                tracing::debug!("[{}] Manufacturer data too short: {:?}", addr, value);
            }
        }
        if manufacturer_data.is_empty() {
            tracing::debug!("[{}] No manufacturer data in advertisement", addr);
        }
    }

    /// Stop monitoring
    pub async fn stop(&mut self) -> Result<()> {
        tracing::info!("Stopping Xiaomi Band monitor...");
        self.running = false;

        // Stop scanning
        if let Err(e) = self.adapter.stop_scan().await {
            tracing::warn!("Error stopping scan: {}", e);
        }

        tracing::info!("Xiaomi Band monitor stopped");
        Ok(())
    }
}
