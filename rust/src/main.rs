// HeartIO Rust - Heart Rate Monitor Application
// Converts TypeScript HeartIO to native Rust application

mod bluetooth;
mod config;
mod database;
mod gui;
mod heart_rate;
mod osc;
mod server;
mod system;

use anyhow::Result;
use gui::{LogEntry, LogLevel};
use std::sync::mpsc;
use tokio::signal;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Print startup banner
    print_banner();

    // Load configuration
    let config = config::Config::load().await?;
    tracing::info!("Configuration loaded successfully");

    // Create communication channels
    let (log_sender, log_receiver) = mpsc::channel();
    let (gui_heart_rate_sender, gui_heart_rate_receiver) = mpsc::channel();

    // Send initial log entries
    send_initial_logs(&log_sender);

    // Create heart rate monitor
    let mut heart_monitor = heart_rate::HeartRateMonitor::new(
        config,
        log_sender.clone(),
        gui_heart_rate_sender.clone(),
    );

    // Setup signal handlers for graceful shutdown
    let log_sender_signal = log_sender.clone();
    let _shutdown_receiver = {
        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            if let Err(e) = signal::ctrl_c().await {
                tracing::error!("Failed to listen for shutdown signal: {}", e);
            } else {
                let _ = log_sender_signal.send(LogEntry {
                    timestamp: chrono::Local::now(),
                    level: LogLevel::Info,
                    message: "Shutdown signal received".to_string(),
                });
                let _ = shutdown_sender.send(());
            }
        });
        shutdown_receiver
    };

    // Start heart rate monitoring and GUI concurrently
    tracing::info!("Starting HeartIO application...");
    
    // Start heart rate monitor in background task
    tokio::spawn(async move {
        if let Err(e) = heart_monitor.start().await {
            tracing::error!("Heart rate monitor error: {}", e);
        }
    });

    // Run GUI on main thread (blocking call)
    if let Err(e) = gui::run_gui_app(log_receiver, gui_heart_rate_receiver).await {
        tracing::error!("GUI application error: {}", e);
    }

    // Graceful shutdown
    tracing::info!("Initiating graceful shutdown...");

    tracing::info!("HeartIO application terminated");
    Ok(())
}

/// Initialize logging system
fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "heartio_rust=info,btleplug=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Print application banner
fn print_banner() {
    println!("╔══════════════════════════════════════╗");
    println!("║              HeartIO Rust            ║");
    println!("║        Heart Rate Monitor            ║");
    println!("║                                      ║");
    println!("║  Platform: {}                ║", 
        format!("{:>20}", system::SystemUtils::get_platform_info()));
    println!("║  Version: v0.1.0                     ║");
    println!("╚══════════════════════════════════════╝");
    println!();
}

/// Send initial log entries to GUI
fn send_initial_logs(log_sender: &mpsc::Sender<LogEntry>) {
    let _ = log_sender.send(LogEntry {
        timestamp: chrono::Local::now(),
        level: LogLevel::Info,
        message: "HeartIO application starting...".to_string(),
    });

    let _ = log_sender.send(LogEntry {
        timestamp: chrono::Local::now(),
        level: LogLevel::Info,
        message: format!("Platform: {}", system::SystemUtils::get_platform_info()),
    });

    let _ = log_sender.send(LogEntry {
        timestamp: chrono::Local::now(),
        level: LogLevel::Info,
        message: "Loading configuration...".to_string(),
    });
}
