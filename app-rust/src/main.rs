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
use tokio::signal::unix;
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
        config.clone(),
        log_sender.clone(),
        gui_heart_rate_sender.clone(),
    );

    // Setup signal handlers for graceful shutdown
    let log_sender_signal = log_sender.clone();
    let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();
    
    // Handle multiple signals for comprehensive coverage
    tokio::spawn(async move {
        let mut sigint = unix::signal(unix::SignalKind::interrupt())
            .expect("Failed to create SIGINT handler");
        let mut sigterm = unix::signal(unix::SignalKind::terminate())
            .expect("Failed to create SIGTERM handler");
        let mut sigquit = unix::signal(unix::SignalKind::quit())
            .expect("Failed to create SIGQUIT handler");
        
        tokio::select! {
            _ = sigint.recv() => {
                let _ = log_sender_signal.send(LogEntry {
                    timestamp: chrono::Local::now(),
                    level: LogLevel::Info,
                    message: "SIGINT (Ctrl+C) received".to_string(),
                });
            }
            _ = sigterm.recv() => {
                let _ = log_sender_signal.send(LogEntry {
                    timestamp: chrono::Local::now(),
                    level: LogLevel::Info,
                    message: "SIGTERM received".to_string(),
                });
            }
            _ = sigquit.recv() => {
                let _ = log_sender_signal.send(LogEntry {
                    timestamp: chrono::Local::now(),
                    level: LogLevel::Info,
                    message: "SIGQUIT received".to_string(),
                });
            }
        }
        
        let _ = shutdown_sender.send(());
    });

    // Start heart rate monitoring and GUI concurrently
    tracing::info!("Starting HeartIO application...");
    
    // Create a separate heart monitor instance for shutdown handling
    let mut heart_monitor_for_shutdown = heart_rate::HeartRateMonitor::new(
        config,
        log_sender.clone(),
        gui_heart_rate_sender.clone(),
    );
    
    let heart_monitor_handle = tokio::spawn(async move {
        if let Err(e) = heart_monitor.start().await {
            tracing::error!("Heart rate monitor error: {}", e);
        }
    });

    // Run GUI on main thread with shutdown handling
    let gui_result = tokio::select! {
        result = gui::run_gui_app(log_receiver, gui_heart_rate_receiver) => {
            tracing::info!("GUI application closed");
            result
        }
        _ = shutdown_receiver => {
            tracing::info!("Shutdown signal received, terminating GUI");
            Ok(())
        }
    };
    
    // Perform graceful shutdown of heart monitor
    tracing::info!("Initiating graceful shutdown...");
    
    // First try to shutdown gracefully
    if let Err(e) = heart_monitor_for_shutdown.shutdown().await {
        tracing::warn!("Error during graceful shutdown: {}", e);
    }
    
    // Then abort the task if it's still running
    heart_monitor_handle.abort();
    let _ = heart_monitor_handle.await;
    
    if let Err(e) = gui_result {
        tracing::error!("GUI application error: {}", e);
    }

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

fn print_table_row(key: &str, value: &str, total_width: usize) {
    let prefix = format!("║  {}: ", key);
    let suffix = " ║";
    let content = format!("{}{}", prefix, value);
    let pad_width = if content.len() + suffix.len() > total_width {
        0
    } else {
        total_width - content.len() - suffix.len()
    };
    println!("{}{}{}", content, " ".repeat(pad_width), suffix);
}

const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Print application banner
fn print_banner() {
    let platform = system::SystemUtils::get_platform_info();

    println!("╔══════════════════════════════════════╗");
    println!("║              HeartIO Rust            ║");
    println!("║        Heart Rate Monitor            ║");
    println!("║                                      ║");
    print_table_row("Platform", &platform, 44);
    println!("║  Version: v{}                     ║", PROJECT_VERSION);
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
