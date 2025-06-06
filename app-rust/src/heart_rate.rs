// Heart rate monitoring and processing for HeartIO
use anyhow::Result;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc as tokio_mpsc;
use tokio::time::interval;

use crate::bluetooth::BluetoothHeartRateMonitor;
use crate::config::Config;
use crate::database::Database;
use crate::gui::{LogEntry, LogLevel, ConnectionStatus, AppStats};
use crate::osc::OscClient;
use crate::server::AppleWatchServer;
use crate::system::SystemUtils;
use crate::xiaomi_band::XiaomiBandMonitor;

pub struct HeartRateMonitor {
    config: Config,
    database: Option<Database>,
    osc_client: Option<OscClient>,
    bluetooth_monitor: Option<BluetoothHeartRateMonitor>,
    xiaomi_band_monitor: Option<XiaomiBandMonitor>,
    system_utils: SystemUtils,
    log_sender: mpsc::Sender<LogEntry>,
    gui_heart_rate_sender: mpsc::Sender<u32>,
    last_send_time: Instant,
    last_receive_time: Option<Instant>,
    start_time: Instant,
    heart_rate_count: u32,
    heart_rate_sum: u32,
}

impl HeartRateMonitor {
    /// Create a new heart rate monitor
    pub fn new(
        config: Config,
        log_sender: mpsc::Sender<LogEntry>,
        gui_heart_rate_sender: mpsc::Sender<u32>,
    ) -> Self {
        Self {
            config,
            database: None,
            osc_client: None,
            bluetooth_monitor: None,
            xiaomi_band_monitor: None,
            system_utils: SystemUtils::new(),
            log_sender,
            gui_heart_rate_sender,
            last_send_time: Instant::now() - Duration::from_secs(10), // Allow immediate first send
            last_receive_time: None,
            start_time: Instant::now(),
            heart_rate_count: 0,
            heart_rate_sum: 0,
        }
    }

    /// Start the heart rate monitoring system
    pub async fn start(&mut self) -> Result<()> {
        self.log_info("Starting HeartIO heart rate monitor...".to_string());

        // Initialize database
        self.init_database().await?;

        // Initialize OSC client
        self.init_osc_client().await?;

        // Keep system awake
        self.keep_system_awake()?;

        // Start monitoring based on configuration
        if self.config.xiaomi_band {
            self.start_xiaomi_band_mode().await?;
        } else if self.config.apple_watch {
            self.start_apple_watch_mode().await?;
        } else {
            self.start_bluetooth_mode().await?;
        }

        Ok(())
    }

    /// Initialize database connection
    async fn init_database(&mut self) -> Result<()> {
        match Database::new().await {
            Ok(db) => {
                self.database = Some(db);
                self.log_info("Database initialized successfully".to_string());
                Ok(())
            }
            Err(e) => {
                self.log_error(format!("Failed to initialize database: {}", e));
                Err(e)
            }
        }
    }

    /// Initialize OSC client
    async fn init_osc_client(&mut self) -> Result<()> {
        match OscClient::new(self.config.osc_host.clone(), self.config.osc_port) {
            Ok(client) => {
                self.osc_client = Some(client);
                self.log_info(format!("OSC client initialized for {}:{}", 
                    self.config.osc_host, self.config.osc_port));
                Ok(())
            }
            Err(e) => {
                self.log_error(format!("Failed to initialize OSC client: {}", e));
                Err(e)
            }
        }
    }

    /// Keep system awake
    fn keep_system_awake(&mut self) -> Result<()> {
        match self.system_utils.keep_system_awake() {
            Ok(_) => {
                self.log_info("System sleep prevention activated".to_string());
                Ok(())
            }
            Err(e) => {
                self.log_warn(format!("Failed to prevent system sleep: {}", e));
                Ok(()) // Non-critical error
            }
        }
    }

    /// Start Apple Watch server mode
    async fn start_apple_watch_mode(&mut self) -> Result<()> {
        self.log_info("Starting Apple Watch server mode...".to_string());

        let (heart_rate_sender, mut heart_rate_receiver) = tokio_mpsc::unbounded_channel();
        
        // Start Apple Watch server
        let server = AppleWatchServer::new(heart_rate_sender);
        let mut server_task = tokio::spawn(async move {
            if let Err(e) = server.start(2333).await {
                tracing::error!("Apple Watch server error: {}", e);
            }
        });

        self.log_info("Apple Watch server started on port 2333".to_string());

        // Start timeout checker
        let mut timeout_task = self.start_timeout_checker().await;

        // Process heart rate data
        loop {
            tokio::select! {
                heart_rate = heart_rate_receiver.recv() => {
                    if let Some(heart_rate) = heart_rate {
                        self.process_heart_rate(heart_rate).await?;
                    }
                }
                _ = &mut timeout_task => {
                    self.log_error("Timeout checker completed".to_string());
                    break;
                }
                _ = &mut server_task => {
                    self.log_error("Apple Watch server stopped".to_string());
                    break;
                }
            }
        }

        Ok(())
    }

    /// Start Bluetooth monitoring mode
    async fn start_bluetooth_mode(&mut self) -> Result<()> {
        self.log_info("Starting Bluetooth monitoring mode...".to_string());

        // Initialize Bluetooth monitor
        let bluetooth_monitor = BluetoothHeartRateMonitor::new().await?;
        
        // Connect to device
        let device_name = self.config.heart_rate_device_name.as_deref();
        let device_address = self.config.heart_rate_device_address.as_deref();
        
        // Use a separate variable to connect, then store it
        let mut connected_monitor = bluetooth_monitor;
        connected_monitor.connect(device_name, device_address).await?;
        self.log_info("Connected to Bluetooth heart rate device".to_string());

        // Store the bluetooth monitor to prevent it from being dropped
        self.bluetooth_monitor = Some(connected_monitor);

        // Start timeout checker
        let _timeout_task = self.start_timeout_checker().await;

        // Start monitoring with callback
        let (heart_rate_sender, mut heart_rate_receiver) = tokio_mpsc::unbounded_channel();
        
        // Take the bluetooth monitor out of self to move it into the task
        if let Some(bluetooth_monitor) = self.bluetooth_monitor.take() {
            let mut monitoring_task = tokio::spawn(async move {
                if let Err(e) = bluetooth_monitor.start_monitoring(move |heart_rate| {
                    let _ = heart_rate_sender.send(heart_rate);
                }).await {
                    tracing::error!("Bluetooth monitoring error: {}", e);
                }
            });

            // Process heart rate data
            loop {
                tokio::select! {
                    heart_rate = heart_rate_receiver.recv() => {
                        if let Some(heart_rate) = heart_rate {
                            self.process_heart_rate(heart_rate).await?;
                        } else {
                            // Channel closed, break the loop
                            break;
                        }
                    }
                    result = &mut monitoring_task => {
                        match result {
                            Ok(()) => self.log_info("Bluetooth monitoring completed".to_string()),
                            Err(e) => self.log_error(format!("Bluetooth monitoring task error: {}", e)),
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }

    /// Start Xiaomi Band monitoring mode
    async fn start_xiaomi_band_mode(&mut self) -> Result<()> {
        self.log_info("Starting Xiaomi Band monitoring mode...".to_string());
        self.log_info("Listening for Xiaomi Smart Band advertisements...".to_string());

        let (heart_rate_sender, mut heart_rate_receiver) = tokio_mpsc::unbounded_channel();
        
        // Create Xiaomi Band monitor
        let mut xiaomi_monitor = XiaomiBandMonitor::new(heart_rate_sender).await?;
        
        // Start monitoring in a separate task
        let mut monitoring_task = tokio::spawn(async move {
            if let Err(e) = xiaomi_monitor.start_monitoring().await {
                tracing::error!("Xiaomi Band monitoring error: {}", e);
            }
        });

        self.log_info("Xiaomi Band monitor started. Waiting for advertisements...".to_string());

        // Start timeout checker
        let mut timeout_task = self.start_timeout_checker().await;

        // Process heart rate data
        loop {
            tokio::select! {
                heart_rate = heart_rate_receiver.recv() => {
                    if let Some(heart_rate) = heart_rate {
                        self.process_heart_rate(heart_rate).await?;
                    } else {
                        // Channel closed, break the loop
                        break;
                    }
                }
                _ = &mut timeout_task => {
                    self.log_error("Timeout checker completed".to_string());
                    break;
                }
                _ = &mut monitoring_task => {
                    self.log_error("Xiaomi Band monitor stopped".to_string());
                    break;
                }
            }
        }

        Ok(())
    }

    /// Process incoming heart rate data
    async fn process_heart_rate(&mut self, heart_rate: u32) -> Result<()> {
        self.last_receive_time = Some(Instant::now());
        self.heart_rate_count += 1;
        self.heart_rate_sum += heart_rate;

        self.log_debug(format!("Received heart rate: {} BPM", heart_rate));

        // Send to GUI
        let _ = self.gui_heart_rate_sender.send(heart_rate);

        // Save to database
        if let Some(db) = &self.database {
            if let Err(e) = db.insert_heart_rate(heart_rate as i32).await {
                self.log_error(format!("Failed to save heart rate to database: {}", e));
            }
        }

        // Send OSC message (with rate limiting)
        self.send_osc_message(heart_rate).await?;

        Ok(())
    }

    /// Send OSC message with rate limiting
    async fn send_osc_message(&mut self, heart_rate: u32) -> Result<()> {
        let now = Instant::now();
        let gap = now.duration_since(self.last_send_time);

        if gap < Duration::from_millis(1500) {
            self.log_debug("OSC send rate limited, skipping".to_string());
            return Ok(());
        }

        if let Some(text) = self.config.get_heart_rate_text(heart_rate) {
            if let Some(osc_client) = &self.osc_client {
                match osc_client.send_message(&text).await {
                    Ok(_) => {
                        self.last_send_time = now;
                        self.log_info(format!("Sent OSC message: {}", text));
                    }
                    Err(e) => {
                        self.log_error(format!("Failed to send OSC message: {}", e));
                    }
                }
            }
        } else {
            self.log_error(format!("Invalid heart rate value: {}", heart_rate));
        }

        Ok(())
    }

    /// Start timeout checker task
    async fn start_timeout_checker(&self) -> tokio::task::JoinHandle<()> {
        let log_sender = self.log_sender.clone();
        
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                
                let _ = log_sender.send(LogEntry {
                    timestamp: chrono::Local::now(),
                    level: LogLevel::Debug,
                    message: "Checking for timeout...".to_string(),
                });
            }
        })
    }

    /// Get current connection status
    pub fn get_connection_status(&self) -> ConnectionStatus {
        ConnectionStatus {
            bluetooth_connected: self.bluetooth_monitor.is_some(),
            osc_connected: self.osc_client.is_some(),
            database_connected: self.database.is_some(),
            apple_watch_server_running: self.config.apple_watch || self.config.xiaomi_band,
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> AppStats {
        AppStats {
            total_heart_rates: self.heart_rate_count,
            session_duration: self.start_time.elapsed(),
            session_start_time: Some(self.start_time),
            last_heart_rate_time: self.last_receive_time.map(|_| chrono::Local::now()),
            avg_heart_rate: if self.heart_rate_count > 0 {
                self.heart_rate_sum as f32 / self.heart_rate_count as f32
            } else {
                0.0
            },
        }
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        self.log_info("Shutting down HeartIO...".to_string());

        // Allow system to sleep
        if let Err(e) = self.system_utils.allow_system_sleep() {
            self.log_warn(format!("Failed to restore system sleep settings: {}", e));
        }

        // Disconnect Bluetooth
        if let Some(mut bluetooth_monitor) = self.bluetooth_monitor.take() {
            if let Err(e) = bluetooth_monitor.disconnect().await {
                self.log_warn(format!("Failed to disconnect Bluetooth device: {}", e));
            }
        }

        // Stop Xiaomi Band monitor
        if let Some(mut xiaomi_monitor) = self.xiaomi_band_monitor.take() {
            if let Err(e) = xiaomi_monitor.stop().await {
                self.log_warn(format!("Failed to stop Xiaomi Band monitor: {}", e));
            }
        }

        // Close database
        if let Some(database) = self.database.take() {
            database.close().await;
        }

        self.log_info("HeartIO shutdown complete".to_string());
        Ok(())
    }

    // Logging helper methods
    fn log_info(&self, message: String) {
        let _ = self.log_sender.send(LogEntry {
            timestamp: chrono::Local::now(),
            level: LogLevel::Info,
            message,
        });
    }

    fn log_warn(&self, message: String) {
        let _ = self.log_sender.send(LogEntry {
            timestamp: chrono::Local::now(),
            level: LogLevel::Warn,
            message,
        });
    }

    fn log_error(&self, message: String) {
        let _ = self.log_sender.send(LogEntry {
            timestamp: chrono::Local::now(),
            level: LogLevel::Error,
            message,
        });
    }

    fn log_debug(&self, message: String) {
        let _ = self.log_sender.send(LogEntry {
            timestamp: chrono::Local::now(),
            level: LogLevel::Debug,
            message,
        });
    }
}
