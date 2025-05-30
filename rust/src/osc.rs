// OSC message handling for HeartIO
use anyhow::{Context, Result};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::UdpSocket;
use std::time::Duration;
use tokio::time::timeout;

const MESSAGE_MAX_LENGTH: usize = 144;
const MESSAGE_PATH: &str = "/chatbox/input";

pub struct OscClient {
    socket: UdpSocket,
    host: String,
    port: u16,
}

impl OscClient {
    /// Create a new OSC client
    pub fn new(host: String, port: u16) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")
            .context("Failed to bind UDP socket for OSC client")?;
        
        tracing::info!("OSC client configured for {}:{}", host, port);
        
        Ok(Self { socket, host, port })
    }

    /// Send OSC message with text
    pub async fn send_message(&self, text: &str) -> Result<()> {
        if text.len() > MESSAGE_MAX_LENGTH {
            anyhow::bail!(
                "Message length {} exceeds maximum of {} characters",
                text.len(),
                MESSAGE_MAX_LENGTH
            );
        }

        let msg = OscMessage {
            addr: MESSAGE_PATH.to_string(),
            args: vec![
                OscType::String(text.to_string()),
                OscType::Bool(true),  // immediate send
                OscType::Bool(false), // disable SFX
            ],
        };

        let packet = OscPacket::Message(msg);
        let encoded = rosc::encoder::encode(&packet)
            .context("Failed to encode OSC message")?;

        let target_addr = format!("{}:{}", self.host, self.port);
        
        // Use tokio::task::spawn_blocking for the blocking UDP send
        let socket = self.socket.try_clone()
            .context("Failed to clone UDP socket")?;
        
        tokio::task::spawn_blocking(move || {
            socket.send_to(&encoded, &target_addr)
        })
        .await
        .context("Failed to spawn blocking task")?
        .context("Failed to send OSC message")?;

        tracing::info!("Sent OSC message: {}", text);
        Ok(())
    }

    /// Test connection by sending a ping message
    pub async fn test_connection(&self) -> Result<()> {
        timeout(
            Duration::from_secs(5),
            self.send_message("HeartIO Connection Test")
        )
        .await
        .context("OSC connection test timed out")?
        .context("OSC connection test failed")?;
        
        tracing::info!("OSC connection test successful");
        Ok(())
    }
}
