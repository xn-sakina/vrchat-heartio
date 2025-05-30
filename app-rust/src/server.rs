// HTTP server for Apple Watch heart rate data
use anyhow::{Context, Result};
use axum::{
    extract::Query,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Debug, Deserialize)]
pub struct HeartRateQuery {
    pub bpm: Option<u32>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    pub status: String,
    pub message: String,
}

pub struct AppleWatchServer {
    heart_rate_sender: mpsc::UnboundedSender<u32>,
}

impl AppleWatchServer {
    /// Create a new Apple Watch server
    pub fn new(heart_rate_sender: mpsc::UnboundedSender<u32>) -> Self {
        Self { heart_rate_sender }
    }

    /// Start the HTTP server
    pub async fn start(&self, port: u16) -> Result<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        
        let app = Router::new()
            .route("/heart", get(heart_rate_handler))
            .route("/health", get(health_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(TraceLayer::new_for_http())
                    .layer(CorsLayer::permissive()),
            )
            .with_state(self.heart_rate_sender.clone());

        tracing::info!("Apple Watch server starting on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(addr).await
            .context("Failed to bind Apple Watch server")?;
        
        axum::serve(listener, app).await
            .context("Apple Watch server error")?;

        Ok(())
    }
}

/// Handle heart rate data from Apple Watch
async fn heart_rate_handler(
    Query(params): Query<HeartRateQuery>,
    axum::extract::State(sender): axum::extract::State<mpsc::UnboundedSender<u32>>,
) -> Result<Json<ApiResponse>, StatusCode> {
    let bpm = match params.bpm {
        Some(bpm) if bpm > 0 && bpm < 300 => bpm,
        Some(_) => {
            tracing::warn!("Invalid BPM value received");
            return Err(StatusCode::BAD_REQUEST);
        }
        None => {
            tracing::warn!("Missing BPM parameter");
            return Err(StatusCode::BAD_REQUEST);
        }
    };

    // Send heart rate data to main processor
    if let Err(_) = sender.send(bpm) {
        tracing::error!("Failed to send heart rate data to processor");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    tracing::info!("Received heart rate from Apple Watch: {}", bpm);

    Ok(Json(ApiResponse {
        status: "success".to_string(),
        message: format!("Heart rate {} BPM received", bpm),
    }))
}

/// Health check endpoint
async fn health_handler() -> Json<ApiResponse> {
    Json(ApiResponse {
        status: "ok".to_string(),
        message: "Apple Watch server is running".to_string(),
    })
}
