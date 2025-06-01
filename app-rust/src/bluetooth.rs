// Bluetooth Low Energy heart rate monitoring for HeartIO
use anyhow::{Context, Result};
use btleplug::api::{Central, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, Peripheral};
use futures::stream::StreamExt;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

// Heart Rate Service UUID definitions
// Short form (16-bit): 0x180D
const HEART_RATE_SERVICE_UUID_SHORT: u16 = 0x180D;

// Heart Rate Measurement Characteristic UUID definitions
// Short form (16-bit): 0x2A37
const HEART_RATE_MEASUREMENT_CHAR_UUID_SHORT: u16 = 0x2A37;

// Helper function to check if a UUID represents the heart rate service
fn is_heart_rate_service_uuid(uuid: &Uuid) -> bool {
    let uuid_bytes = uuid.as_u128();

    // Extract the 16-bit service identifier
    let service_id = ((uuid_bytes >> 96) & 0xFFFF) as u16;
    service_id == HEART_RATE_SERVICE_UUID_SHORT
}

// Helper function to check if a UUID represents the heart rate measurement characteristic
fn is_heart_rate_measurement_char_uuid(uuid: &Uuid) -> bool {
    let uuid_bytes = uuid.as_u128();

    let char_id = ((uuid_bytes >> 96) & 0xFFFF) as u16;
    char_id == HEART_RATE_MEASUREMENT_CHAR_UUID_SHORT
}

pub struct BluetoothHeartRateMonitor {
    adapter: Adapter,
    device: Option<Peripheral>,
}

impl BluetoothHeartRateMonitor {
    /// Create a new Bluetooth heart rate monitor
    pub async fn new() -> Result<Self> {
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

        tracing::info!("Bluetooth adapter initialized");

        Ok(Self {
            adapter,
            device: None,
        })
    }

    /// Start scanning and connect to heart rate device
    pub async fn connect(
        &mut self,
        device_name: Option<&str>,
        device_address: Option<&str>,
    ) -> Result<()> {
        tracing::info!("Starting device discovery...");

        // Start scanning
        self.adapter
            .start_scan(ScanFilter::default())
            .await
            .context("Failed to start Bluetooth scan")?;

        let device = if let Some(name) = device_name {
            self.find_device_by_name(name).await?
        } else if let Some(address) = device_address {
            self.find_device_by_address(address).await?
        } else {
            tracing::warn!("No device name or address provided, using guess mode");
            tracing::info!("Searching for heart rate devices...");
            self.find_heart_rate_device().await?
        };

        // Stop scanning
        self.adapter
            .stop_scan()
            .await
            .context("Failed to stop Bluetooth scan")?;

        // Connect to device
        device
            .connect()
            .await
            .context("Failed to connect to heart rate device")?;

        let device_name = device
            .properties()
            .await
            .ok()
            .flatten()
            .and_then(|p| p.local_name)
            .unwrap_or_else(|| "Unknown".to_string());

        tracing::info!("Connected to device: {}", device_name);
        self.device = Some(device);

        Ok(())
    }

    /// Find device by name
    async fn find_device_by_name(&self, target_name: &str) -> Result<Peripheral> {
        let timeout_duration = Duration::from_secs(10);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            let peripherals = self
                .adapter
                .peripherals()
                .await
                .context("Failed to get peripherals")?;

            for peripheral in peripherals {
                if let Ok(Some(properties)) = peripheral.properties().await {
                    if let Some(name) = &properties.local_name {
                        if name.to_lowercase() == target_name.to_lowercase() {
                            tracing::info!("Found device by name: {}", name);
                            return Ok(peripheral);
                        }
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        anyhow::bail!(
            "Device with name '{}' not found within timeout",
            target_name
        );
    }

    /// Find device by address
    async fn find_device_by_address(&self, target_address: &str) -> Result<Peripheral> {
        let timeout_duration = Duration::from_secs(10);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            let peripherals = self
                .adapter
                .peripherals()
                .await
                .context("Failed to get peripherals")?;

            for peripheral in peripherals {
                if let Ok(Some(properties)) = peripheral.properties().await {
                    let address = properties.address.to_string();
                    if address.to_lowercase() == target_address.to_lowercase() {
                        tracing::info!("Found device by address: {}", address);
                        return Ok(peripheral);
                    }
                }
            }

            sleep(Duration::from_millis(500)).await;
        }

        anyhow::bail!(
            "Device with address '{}' not found within timeout",
            target_address
        );
    }

    /// Find any heart rate device
    async fn find_heart_rate_device(&self) -> Result<Peripheral> {
        let timeout_duration = Duration::from_secs(30);
        let start_time = std::time::Instant::now();

        while start_time.elapsed() < timeout_duration {
            let peripherals = self
                .adapter
                .peripherals()
                .await
                .context("Failed to get peripherals")?;

            tracing::debug!("Scanning {} peripherals...", peripherals.len());

            for peripheral in peripherals {
                if let Ok(Some(properties)) = peripheral.properties().await {
                    let device_name = properties.local_name.as_deref().unwrap_or("Unknown");
                    let device_address = properties.address.to_string();

                    tracing::debug!("Device: {} ({})", device_name, device_address);
                    tracing::debug!("  Services: {:?}", properties.services);

                    // Check if any of the advertised services is a heart rate service
                    for service_uuid in &properties.services {
                        if is_heart_rate_service_uuid(service_uuid) {
                            tracing::warn!(
                                "Found heart rate device: {} ({})",
                                device_name,
                                device_address
                            );
                            tracing::warn!("  Heart rate service UUID: {}", service_uuid);
                            tracing::warn!(
                                "Recommended to set HEART_RATE_DEVICE_NAME or HEART_RATE_DEVICE_ADDRESS for stable connection"
                            );
                            return Ok(peripheral);
                        }
                    }
                }
            }

            sleep(Duration::from_millis(1000)).await;
        }

        anyhow::bail!("No heart rate device found within timeout");
    }

    /// Start monitoring heart rate data
    pub async fn start_monitoring<F>(&self, mut callback: F) -> Result<()>
    where
        F: FnMut(u32) + Send + 'static,
    {
        let device = self.device.as_ref().context("No device connected")?;

        tracing::info!("Starting heart rate monitoring...");

        // Wait a bit for the device to stabilize after connection
        sleep(Duration::from_millis(1000)).await;

        // Discover services and characteristics with retry
        let mut retry_count = 0;
        let max_retries = 3;

        while retry_count < max_retries {
            tracing::info!(
                "Discovering services (attempt {}/{})",
                retry_count + 1,
                max_retries
            );

            match device.discover_services().await {
                Ok(_) => {
                    tracing::info!("Services discovered successfully");
                    break;
                }
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(anyhow::anyhow!(
                            "Failed to discover services after {} attempts: {}",
                            max_retries,
                            e
                        ));
                    }
                    tracing::warn!(
                        "Service discovery failed (attempt {}), retrying in 2 seconds: {}",
                        retry_count,
                        e
                    );
                    sleep(Duration::from_millis(2000)).await;
                }
            }
        }

        let services = device.services();
        tracing::info!("Found {} services", services.len());

        for service in &services {
            tracing::debug!(
                "Service: {} with {} characteristics",
                service.uuid,
                service.characteristics.len()
            );
            for characteristic in &service.characteristics {
                tracing::debug!("  Characteristic: {}", characteristic.uuid);
            }
        }

        // Find heart rate service using compatibility check
        let heart_rate_service = services
            .iter()
            .find(|s| is_heart_rate_service_uuid(&s.uuid))
            .context("Heart rate service not found")?;

        tracing::info!("Found heart rate service: {}", heart_rate_service.uuid);

        // Find heart rate measurement characteristic using compatibility check
        let heart_rate_char = heart_rate_service
            .characteristics
            .iter()
            .find(|c| is_heart_rate_measurement_char_uuid(&c.uuid))
            .context("Heart rate measurement characteristic not found")?;

        tracing::info!(
            "Found heart rate measurement characteristic: {}",
            heart_rate_char.uuid
        );

        // Subscribe to notifications
        device
            .subscribe(heart_rate_char)
            .await
            .context("Failed to subscribe to heart rate characteristic")?;

        tracing::info!(
            "Subscribed to heart rate characteristic: {}",
            heart_rate_char.uuid
        );

        // Listen for notifications
        let mut notification_stream = device
            .notifications()
            .await
            .context("Failed to get notification stream")?;

        tracing::info!("Listening for heart rate notifications...");

        while let Some(data) = notification_stream.next().await {
            if is_heart_rate_measurement_char_uuid(&data.uuid) {
                if let Some(heart_rate) = Self::parse_heart_rate_data(&data.value) {
                    tracing::debug!("Heart rate: {}", heart_rate);
                    callback(heart_rate);
                }
            }
        }

        Ok(())
    }

    /// Parse heart rate data from BLE notification
    fn parse_heart_rate_data(data: &[u8]) -> Option<u32> {
        if data.is_empty() {
            return None;
        }

        let flags = data[0];
        let heart_rate = if flags & 0x01 != 0 {
            // 16-bit heart rate value
            if data.len() >= 3 {
                u16::from_le_bytes([data[1], data[2]]) as u32
            } else {
                return None;
            }
        } else {
            // 8-bit heart rate value
            if data.len() >= 2 {
                data[1] as u32
            } else {
                return None;
            }
        };

        if heart_rate > 0 && heart_rate < 300 {
            Some(heart_rate)
        } else {
            None
        }
    }

    /// Disconnect from device
    pub async fn disconnect(&mut self) -> Result<()> {
        if let Some(device) = &self.device {
            device
                .disconnect()
                .await
                .context("Failed to disconnect from device")?;
            tracing::info!("Disconnected from heart rate device");
        }
        self.device = None;
        Ok(())
    }

    /// Check if device is connected
    pub async fn is_connected(&self) -> bool {
        if let Some(device) = &self.device {
            device.is_connected().await.unwrap_or(false)
        } else {
            false
        }
    }
}
