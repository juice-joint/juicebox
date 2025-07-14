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
    let manager = WpaSupplicantManager::new();
    
    // Update config file but don't reload yet - so user can see success page
    match manager.update_network_without_reload(&config.ssid, &config.password) {
        Ok(()) => {
            info!("WiFi configuration updated for SSID: {}", config.ssid);
            
            // Spawn background task to reload wpa_supplicant after delay
            // This gives user time to see success page before AP disconnects
            let ssid_clone = config.ssid.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                if let Err(e) = manager.reload_wpa_supplicant_only() {
                    error!("Failed to reload wpa_supplicant: {}", e);
                } else {
                    info!("wpa_supplicant reloaded for SSID: {}", ssid_clone);
                }
            });
            
            Ok(Html(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>WiFi Configuration - Success</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 600px;
            margin: 50px auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
            text-align: center;
        }
        .success-icon {
            font-size: 64px;
            color: #28a745;
            margin-bottom: 20px;
        }
        h1 {
            color: #333;
            margin-bottom: 20px;
        }
        .success {
            background: #d4edda;
            border: 1px solid #c3e6cb;
            color: #155724;
            padding: 20px;
            border-radius: 8px;
            margin: 20px 0;
            text-align: left;
        }
        .button {
            background: #007bff;
            color: white;
            padding: 12px 24px;
            text-decoration: none;
            border-radius: 5px;
            display: inline-block;
            margin: 20px 10px 0 10px;
            font-size: 16px;
            transition: background-color 0.3s;
        }
        .button:hover {
            background: #0056b3;
        }
        .secondary-button {
            background: #6c757d;
        }
        .secondary-button:hover {
            background: #545b62;
        }
        .info {
            background: #e1f5fe;
            border: 1px solid #b3e5fc;
            color: #01579b;
            padding: 15px;
            border-radius: 5px;
            margin: 20px 0;
            text-align: left;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="success-icon">✅</div>
        <h1>WiFi Configuration Successful!</h1>
        
        <div class="success">
            <strong>Success!</strong> Your WiFi network has been configured successfully.
            <br><br>
            The system is now attempting to connect to the configured network. 
            This process may take a few moments.
        </div>
        
        <div class="info">
            <strong>What happens next?</strong>
            <ul style="margin: 10px 0;">
                <li>The device will attempt to connect to your WiFi network</li>
                <li>If successful, the access point mode will be disabled</li>
                <li>You can monitor the connection status through your network settings</li>
            </ul>
        </div>
        
        <a href="/" class="button">Configure Another Network</a>
        <a href="javascript:window.close()" class="button secondary-button">Close Window</a>
    </div>
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