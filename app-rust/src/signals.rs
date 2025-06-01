// Cross-platform signal handling for HeartIO
use anyhow::Result;
use tokio::signal;

/// Setup cross-platform signal handlers
pub async fn wait_for_shutdown_signal() -> Result<()> {
    #[cfg(unix)]
    {
        use signal::unix::{signal, SignalKind};
        
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;
        
        tokio::select! {
            _ = sigint.recv() => {
                tracing::info!("Received SIGINT (Ctrl+C)");
            }
            _ = sigterm.recv() => {
                tracing::info!("Received SIGTERM");
            }
        }
    }
    
    #[cfg(windows)]
    {
        let mut ctrl_c = signal::windows::ctrl_c()?;
        let mut ctrl_break = signal::windows::ctrl_break()?;
        let mut ctrl_close = signal::windows::ctrl_close()?;
        let mut ctrl_logoff = signal::windows::ctrl_logoff()?;
        let mut ctrl_shutdown = signal::windows::ctrl_shutdown()?;
        
        tokio::select! {
            _ = ctrl_c.recv() => {
                tracing::info!("Received Ctrl+C");
            }
            _ = ctrl_break.recv() => {
                tracing::info!("Received Ctrl+Break");
            }
            _ = ctrl_close.recv() => {
                tracing::info!("Received Ctrl+Close");
            }
            _ = ctrl_logoff.recv() => {
                tracing::info!("Received Ctrl+Logoff");
            }
            _ = ctrl_shutdown.recv() => {
                tracing::info!("Received Ctrl+Shutdown");
            }
        }
    }
    
    Ok(())
}
