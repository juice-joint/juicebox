use axum::{http::StatusCode, response::IntoResponse, Json};
use local_ip_address::local_ip;
use serde::Serialize;
use tracing::debug;
use std::path::Path;

#[derive(Serialize)]
struct ServerIpResponse {
    ip: String,
}

#[derive(Serialize)]
struct AutoApStatusResponse {
    is_running: bool,
    web_server_port: Option<u16>,
}

pub async fn server_ip() -> Result<impl IntoResponse, StatusCode> {
    let my_local_ip = match local_ip() {
        Ok(ip) => ip,
        Err(_) => {
            debug!("Could not determine local IP address - likely no network connection");
            return Err(StatusCode::SERVICE_UNAVAILABLE);
        }
    };

    debug!("my local ip {:?}", my_local_ip);

    Ok((
        StatusCode::OK,
        Json(ServerIpResponse {
            ip: my_local_ip.to_string(),
        }),
    ))
}

pub async fn autoap_status() -> Result<impl IntoResponse, StatusCode> {    
    // Check if autoap is running by looking for runtime indicators
    let is_running =
        // Check for autoap runtime files (lock files, service status, etc.)
        Path::new("/var/run/autoAP.locked").exists() ||
        Path::new("/var/run/autoAP.unlock").exists() ||
        is_autoap_service_active();

    // Try to detect the web server port if autoap is running
    let web_server_port = if is_running {
        detect_autoap_web_server_port().await
    } else {
        None
    };

    debug!("AutoAP status - running: {}, port: {:?}", is_running, web_server_port);

    Ok((
        StatusCode::OK,
        Json(AutoApStatusResponse {
            is_running,
            web_server_port,
        }),
    ))
}

fn is_autoap_service_active() -> bool {
    // Check if the wpa-autoap@wlan0 service is active
    let output = std::process::Command::new("systemctl")
        .args(["is-active", "wpa-autoap@wlan0"])
        .output();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

async fn detect_autoap_web_server_port() -> Option<u16> {
    // First try the default port
    if test_port_connectivity(8080).await {
        return Some(8080);
    }
    
    // If not on default port, try to detect from process list
    // Look for autoap processes with port arguments
    if let Ok(output) = std::process::Command::new("ps")
        .args(["aux"])
        .output()
    {
        let ps_output = String::from_utf8_lossy(&output.stdout);
        for line in ps_output.lines() {
            if line.contains("autoap") && (line.contains("start") || line.contains("web")) {
                // Try to extract port from command line
                if let Some(port) = extract_port_from_command_line(line) {
                    if test_port_connectivity(port).await {
                        return Some(port);
                    }
                }
            }
        }
    }
    
    // If we can't detect the port, return the default as a fallback
    Some(8080)
}

async fn test_port_connectivity(port: u16) -> bool {
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::time::timeout;
    
    let addr = format!("127.0.0.1:{}", port);
    match timeout(Duration::from_millis(500), TcpStream::connect(addr)).await {
        Ok(Ok(_)) => true,
        _ => false,
    }
}

fn extract_port_from_command_line(command_line: &str) -> Option<u16> {
    // Look for patterns like "--port 8081" or "-p 8081"
    let words: Vec<&str> = command_line.split_whitespace().collect();
    for i in 0..words.len() {
        if (words[i] == "--port" || words[i] == "-p") && i + 1 < words.len() {
            if let Ok(port) = words[i + 1].parse::<u16>() {
                return Some(port);
            }
        }
        // Also check for --port=8081 format
        if words[i].starts_with("--port=") {
            if let Some(port_str) = words[i].strip_prefix("--port=") {
                if let Ok(port) = port_str.parse::<u16>() {
                    return Some(port);
                }
            }
        }
    }
    None
}
