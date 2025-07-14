use anyhow::{Context, Result};
use axum::{
    extract::Form,
    http::StatusCode,
    response::{Html, Json},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

use crate::wpa_manager::WpaSupplicantManager;

#[derive(Debug, Deserialize)]
pub struct WifiConfig {
    ssid: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse {
    success: bool,
    message: String,
}

pub struct WebServer;

impl WebServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(&self, port: u16) -> Result<()> {
        let app = Router::new()
            .route("/", get(serve_config_page))
            .route("/configure", post(configure_wifi))
            .route("/api/configure", post(api_configure_wifi))
            .route("/api/status", get(api_status))
            .layer(CorsLayer::permissive());

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        info!("Web server starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await
            .context("Failed to bind to address")?;

        axum::serve(listener, app).await
            .context("Failed to start web server")?;

        Ok(())
    }
}

async fn serve_config_page() -> Html<String> {
    let html_content = std::fs::read_to_string("static/config.html")
        .unwrap_or_else(|_| {
            r#"<!DOCTYPE html>
<html><head><title>WiFi Config</title></head>
<body>
<h1>WiFi Configuration</h1>
<form action="/configure" method="POST">
<label>SSID: <input type="text" name="ssid" required></label><br><br>
<label>Password: <input type="password" name="password" required></label><br><br>
<button type="submit">Configure</button>
</form>
</body></html>"#.to_string()
        });
    Html(html_content)
}

async fn configure_wifi(Form(config): Form<WifiConfig>) -> Result<Html<&'static str>, StatusCode> {
    match WpaSupplicantManager::new().update_network(&config.ssid, &config.password) {
        Ok(()) => {
            info!("WiFi configuration updated for SSID: {}", config.ssid);
            Ok(Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>WiFi Configuration - Success</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
        .success { background: #d4edda; border: 1px solid #c3e6cb; color: #155724; padding: 15px; border-radius: 5px; margin: 20px 0; }
        .button { background: #007bff; color: white; padding: 10px 20px; text-decoration: none; border-radius: 5px; display: inline-block; margin: 10px 0; }
    </style>
</head>
<body>
    <h1>WiFi Configuration Successful</h1>
    <div class="success">
        <strong>Success!</strong> WiFi network configuration has been updated.
        <br>The system will attempt to connect to the configured network.
    </div>
    <a href="/" class="button">Configure Another Network</a>
</body>
</html>
            "#))
        }
        Err(e) => {
            error!("Failed to configure WiFi: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn api_configure_wifi(Json(config): Json<WifiConfig>) -> Json<ApiResponse> {
    match WpaSupplicantManager::new().update_network(&config.ssid, &config.password) {
        Ok(()) => {
            info!("WiFi configuration updated via API for SSID: {}", config.ssid);
            Json(ApiResponse {
                success: true,
                message: format!("WiFi network '{}' configured successfully", config.ssid),
            })
        }
        Err(e) => {
            error!("Failed to configure WiFi via API: {}", e);
            Json(ApiResponse {
                success: false,
                message: format!("Failed to configure WiFi: {}", e),
            })
        }
    }
}

async fn api_status() -> Json<ApiResponse> {
    Json(ApiResponse {
        success: true,
        message: "WiFi configuration server is running".to_string(),
    })
}