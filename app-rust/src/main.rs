// HeartIO Rust - Heart Rate Monitor Application
// Converts TypeScript HeartIO to native Rust application

mod bluetooth;
mod config;
mod database;
mod gui;
mod heart_rate;
mod osc;
mod server;
mod signals;
mod system;

use anyhow::Result;
use gui::{LogEntry, LogLevel};
use std::sync::{mpsc, Arc};
use tokio::sync::{Mutex, oneshot};
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

    // Create heart rate monitor with Arc for sharing between tasks
    let heart_monitor = Arc::new(Mutex::new(heart_rate::HeartRateMonitor::new(
        config,
        log_sender.clone(),
        gui_heart_rate_sender.clone(),
    )));

    // Setup comprehensive signal handlers for graceful shutdown
    let log_sender_signal = log_sender.clone();
    let heart_monitor_signal = Arc::clone(&heart_monitor);
    let (shutdown_sender, shutdown_receiver) = oneshot::channel();
    
    // Handle multiple shutdown signals
    tokio::spawn(async move {
        if let Err(e) = signals::wait_for_shutdown_signal().await {
            tracing::error!("Error setting up signal handlers: {}", e);
        }
        
        let _ = log_sender_signal.send(LogEntry {
            timestamp: chrono::Local::now(),
            level: LogLevel::Info,
            message: "Shutdown signal received, cleaning up...".to_string(),
        });
        
        // Perform cleanup
        {
            let mut monitor = heart_monitor_signal.lock().await;
            if let Err(e) = monitor.shutdown().await {
                tracing::error!("Error during shutdown: {}", e);
            }
        }
        
        let _ = shutdown_sender.send(());
    });

    // Start heart rate monitoring in background task
    let heart_monitor_clone = Arc::clone(&heart_monitor);
    let heart_monitor_handle = tokio::spawn(async move {
        {
            let mut monitor = heart_monitor_clone.lock().await;
            if let Err(e) = monitor.start().await {
                tracing::error!("Heart rate monitor error: {}", e);
            }
        }
    });

    tracing::info!("Starting HeartIO application...");

    // Run GUI on main thread (blocking call) with graceful shutdown handling
    let gui_result = tokio::select! {
        result = gui::run_gui_app(log_receiver, gui_heart_rate_receiver) => result,
        _ = shutdown_receiver => {
            tracing::info!("Shutdown signal received during GUI execution");
            Ok(())
        }
    };
    
    // Abort heart monitor task and perform cleanup
    heart_monitor_handle.abort();
    let _ = heart_monitor_handle.await;
    
    // Final cleanup - ensure resources are freed
    {
        let mut monitor = heart_monitor.lock().await;
        if let Err(e) = monitor.shutdown().await {
            tracing::error!("Error during final cleanup: {}", e);
        }
    }
    
    if let Err(e) = gui_result {
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
    println!("║  Version : v{}                     ║", PROJECT_VERSION);
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
