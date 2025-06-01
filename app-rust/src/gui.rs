// GUI application for HeartIO using egui
use anyhow::Result;
use chrono::{DateTime, Local};
use eframe::egui;
use std::collections::VecDeque;
use std::sync::mpsc;

const MAX_LOG_ENTRIES: usize = 1000;

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
    Debug,
}

impl LogLevel {
    pub fn color(&self) -> egui::Color32 {
        match self {
            LogLevel::Info => egui::Color32::from_rgb(70, 130, 180), // Steel blue
            LogLevel::Warn => egui::Color32::from_rgb(255, 165, 0),  // Orange
            LogLevel::Error => egui::Color32::from_rgb(220, 20, 60), // Crimson
            LogLevel::Debug => egui::Color32::from_rgb(128, 128, 128), // Gray
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warn => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

pub struct HeartIOApp {
    log_entries: VecDeque<LogEntry>,
    log_receiver: mpsc::Receiver<LogEntry>,
    auto_scroll: bool,
    show_debug: bool,
    current_heart_rate: Option<u32>,
    heart_rate_receiver: mpsc::Receiver<u32>,
    connection_status: ConnectionStatus,
    stats: AppStats,
}

#[derive(Debug, Clone)]
pub struct ConnectionStatus {
    pub bluetooth_connected: bool,
    pub osc_connected: bool,
    pub database_connected: bool,
    pub apple_watch_server_running: bool,
}

#[derive(Debug, Clone)]
pub struct AppStats {
    pub total_heart_rates: u32,
    pub session_duration: std::time::Duration,
    pub session_start_time: Option<std::time::Instant>,
    pub last_heart_rate_time: Option<DateTime<Local>>,
    pub avg_heart_rate: f32,
}

impl Default for AppStats {
    fn default() -> Self {
        Self {
            total_heart_rates: 0,
            session_duration: std::time::Duration::new(0, 0),
            session_start_time: None,
            last_heart_rate_time: None,
            avg_heart_rate: 0.0,
        }
    }
}

impl HeartIOApp {
    /// Create a new HeartIO GUI application
    pub fn new(
        log_receiver: mpsc::Receiver<LogEntry>,
        heart_rate_receiver: mpsc::Receiver<u32>,
    ) -> Self {
        Self {
            log_entries: VecDeque::new(),
            log_receiver,
            auto_scroll: true,
            show_debug: false,
            current_heart_rate: None,
            heart_rate_receiver,
            connection_status: ConnectionStatus {
                bluetooth_connected: false,
                osc_connected: false,
                database_connected: false,
                apple_watch_server_running: false,
            },
            stats: AppStats::default(),
        }
    }

    /// Add a log entry to the display
    pub fn add_log_entry(&mut self, entry: LogEntry) {
        self.log_entries.push_back(entry);
        if self.log_entries.len() > MAX_LOG_ENTRIES {
            self.log_entries.pop_front();
        }
    }

    /// Update connection status
    pub fn update_connection_status(&mut self, status: ConnectionStatus) {
        self.connection_status = status;
    }

    /// Update app statistics
    pub fn update_stats(&mut self, stats: AppStats) {
        self.stats = stats;
    }
}

impl eframe::App for HeartIOApp {
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        tracing::info!("Application exiting - performing final cleanup");
        crate::system::SystemUtils::immediate_cleanup();
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle window close events (including cmd+q on macOS)
        if ctx.input(|i| i.viewport().close_requested()) {
            tracing::info!("GUI close requested by user - performing immediate cleanup");
            
            // Perform immediate synchronous cleanup before the application exits
            crate::system::SystemUtils::immediate_cleanup();
        }

        // Process incoming log entries
        while let Ok(entry) = self.log_receiver.try_recv() {
            self.add_log_entry(entry);
        }

        // Update session duration
        let now = std::time::Instant::now();
        if let Some(start) = self.stats.session_start_time {
            self.stats.session_duration = now.duration_since(start);
        } else {
            self.stats.session_start_time = Some(now);
        }

        // Process incoming heart rate data
        while let Ok(heart_rate) = self.heart_rate_receiver.try_recv() {
            self.current_heart_rate = Some(heart_rate);
            self.stats.total_heart_rates += 1;
            self.stats.last_heart_rate_time = Some(Local::now());

            // Update average (simple running average)
            if self.stats.total_heart_rates == 1 {
                self.stats.avg_heart_rate = heart_rate as f32;
            } else {
                let alpha = 0.1; // Smoothing factor
                self.stats.avg_heart_rate =
                    alpha * heart_rate as f32 + (1.0 - alpha) * self.stats.avg_heart_rate;
            }
        }

        // Top panel with status and controls
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("HeartIO");

                ui.separator();

                // Current heart rate display
                if let Some(hr) = self.current_heart_rate {
                    ui.label(
                        egui::RichText::new(format!("{} BPM", hr))
                            .size(18.0)
                            .color(egui::Color32::from_rgb(220, 20, 60)),
                    );
                } else {
                    ui.label(
                        egui::RichText::new("-- BPM")
                            .size(18.0)
                            .color(egui::Color32::GRAY),
                    );
                }

                ui.separator();

                // Connection status indicators
                self.draw_connection_status(ui);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.checkbox(&mut self.show_debug, "Show Debug");
                    ui.checkbox(&mut self.auto_scroll, "Auto Scroll");
                });
            });
        });

        // Side panel with statistics
        egui::SidePanel::right("stats_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.heading("Statistics");
                ui.separator();

                egui::Grid::new("stats_grid")
                    .num_columns(2)
                    .spacing([40.0, 4.0])
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Total Readings:");
                        ui.label(self.stats.total_heart_rates.to_string());
                        ui.end_row();

                        ui.label("Average BPM:");
                        ui.label(format!("{:.1}", self.stats.avg_heart_rate));
                        ui.end_row();

                        ui.label("Session Time:");
                        ui.label(format!("{:.0}s", self.stats.session_duration.as_secs()));
                        ui.end_row();

                        if let Some(last_time) = &self.stats.last_heart_rate_time {
                            ui.label("Last Reading:");
                            ui.label(last_time.format("%H:%M:%S").to_string());
                            ui.end_row();
                        }
                    });

                ui.separator();
                ui.heading("Connection");

                self.draw_detailed_connection_status(ui);
            });

        // Central panel with logs
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Logs");

            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(self.auto_scroll)
                .show(ui, |ui| {
                    for entry in &self.log_entries {
                        if !self.show_debug && entry.level == LogLevel::Debug {
                            continue;
                        }

                        ui.horizontal(|ui| {
                            ui.label(entry.level.icon());
                            ui.label(
                                egui::RichText::new(entry.timestamp.format("%H:%M:%S").to_string())
                                    .size(11.0)
                                    .color(egui::Color32::GRAY),
                            );
                            ui.label(
                                egui::RichText::new(&entry.message).color(entry.level.color()),
                            );
                        });
                    }
                });
        });

        // Request repaint for real-time updates
        ctx.request_repaint_after(std::time::Duration::from_millis(100));
    }
}

impl HeartIOApp {
    fn draw_connection_status(&self, ui: &mut egui::Ui) {
        let status_color = |connected: bool| {
            if connected {
                egui::Color32::from_rgb(0, 128, 0) // Green
            } else {
                egui::Color32::from_rgb(128, 128, 128) // Gray
            }
        };

        ui.label(
            egui::RichText::new("BlueTooth")
                .color(status_color(self.connection_status.bluetooth_connected)),
        );
        ui.label(
            egui::RichText::new("OSC").color(status_color(self.connection_status.osc_connected)),
        );

        if self.connection_status.apple_watch_server_running {
            ui.label(egui::RichText::new("AW").color(status_color(true)));
        }
    }

    fn draw_detailed_connection_status(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Bluetooth");
        });

        ui.horizontal(|ui| {
            ui.label("OSC Server");
        });

        if self.connection_status.apple_watch_server_running {
            ui.horizontal(|ui| {
                ui.label("Connected");
                ui.label("Apple Watch");
            });
        }
    }
}

/// Create and run the GUI application
pub async fn run_gui_app(
    log_receiver: mpsc::Receiver<LogEntry>,
    heart_rate_receiver: mpsc::Receiver<u32>,
) -> Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(eframe::icon_data::from_png_bytes(&[]).unwrap_or_default()),
        ..Default::default()
    };

    let app = HeartIOApp::new(log_receiver, heart_rate_receiver);

    eframe::run_native(
        "HeartIO - Heart Rate Monitor",
        options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow::anyhow!("GUI application error: {}", e))?;

    Ok(())
}
