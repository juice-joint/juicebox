use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::fs;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};
use wpa_events::{WpaEventMonitor, WpaEvent, WpaState, WpaEventHandler};

use crate::config::AutoApConfig;
use crate::utils::{has_connected_stations, systemctl_command, wpa_cli_command};
use crate::web_server::WebServer;

pub struct AutoAp {
    config: AutoApConfig,
}

impl AutoAp {
    pub async fn new() -> Result<Self> {
        let config = AutoApConfig::load()?;
        Ok(Self { config })
    }

    pub async fn run(&self, args: Vec<String>) -> Result<()> {
        if args.len() < 2 {
            return Err(anyhow::anyhow!("Insufficient arguments"));
        }

        // Handle special cases first
        if args[1] == "reset" {
            info!("Performing reset operation");
            self.reset().await?;
            return Ok(());
        }
        
        if args[1] == "start" {
            if args.len() < 3 {
                return Err(anyhow::anyhow!("Start command requires device name"));
            }
            let device = &args[2];
            info!("Starting autoAP for device: {}", device);
            self.start(device).await?;
            return Ok(());
        }

        // For wpa_cli events, create a monitor and process the event
        let interface = &args[1];
        let handler = AutoApHandler {
            config: self.config.clone(),
        };
        
        let monitor = WpaEventMonitor::new(interface, handler)?;
        monitor.process_event(args).await?;

        Ok(())
    }

    pub async fn reset(&self) -> Result<()> {
        info!("Resetting autoAP state");
        
        let lock_file = "/var/run/autoAP.locked";
        let unlock_file = "/var/run/autoAP.unlock";
        let backup_network = "/etc/systemd/network/11-wlan0.network~";
        let network_file = "/etc/systemd/network/11-wlan0.network";

        // Remove lock files
        if Path::new(lock_file).exists() {
            fs::remove_file(lock_file).await
                .context("Failed to remove lock file")?;
        }

        if Path::new(unlock_file).exists() {
            fs::remove_file(unlock_file).await
                .context("Failed to remove unlock file")?;
        }

        // Restore network file if backup exists
        if Path::new(backup_network).exists() {
            fs::rename(backup_network, network_file).await
                .context("Failed to restore network file")?;
        }

        Ok(())
    }

    pub async fn start(&self, device: &str) -> Result<()> {
        self.reset().await?;

        let wpa_socket_path = format!("/var/run/wpa_supplicant/{}", device);
        
        // Wait for wpa_supplicant to come online
        while !Path::new(&wpa_socket_path).exists() {
            info!("Waiting for wpa_supplicant to come online");
            sleep(Duration::from_millis(500)).await;
        }

        info!("wpa_supplicant online, starting monitor for {}", device);
        
        let handler = AutoApHandler {
            config: self.config.clone(),
        };
        
        let monitor = WpaEventMonitor::new(device, handler)?;
        monitor.start().await?;
        
        Ok(())
    }
}

// Handler that implements the wpa-events EventHandler trait
struct AutoApHandler {
    config: AutoApConfig,
}

#[async_trait]
impl WpaEventHandler for AutoApHandler {
    async fn handle_event(&self, event: WpaEvent) -> Result<()> {
        self.log_flags().await;

        match event.state {
            WpaState::ApEnabled => {
                info!("AP enabled, configuring access point");
                Self::configure_ap(&event.interface).await?;
                
                // Start web server for WiFi configuration
                tokio::spawn(async move {
                    info!("Starting WiFi configuration web server on port 8080");
                    let server = WebServer::new();
                    if let Err(e) = server.start(8080).await {
                        error!("Failed to start web server: {}", e);
                    }
                });
                
                // Start reconfigure task in background
                let device = event.interface.clone();
                let enable_wait = self.config.enable_wait;
                tokio::spawn(async move {
                    if let Err(e) = Self::reconfigure_wpa_supplicant_static(&device, enable_wait).await {
                        error!("Failed to reconfigure wpa_supplicant: {}", e);
                    }
                });
            }
            WpaState::ApDisabled => {
                info!("AP disabled");
            }
            WpaState::Connected => {
                info!("Connected to network");
                
                // Verify we're actually connected to a client network, not just AP mode
                if Self::is_actually_connected_to_client(&event.interface).await? {
                    info!("CONNECTED in station mode, configuring client");
                    Self::configure_client(&event.interface).await?;
                } else {
                    info!("CONNECTED event received but not actually connected to client network, ignoring");
                }
            }
            WpaState::ApStaDisconnected => {
                if let Some(mac) = &event.mac_address {
                    info!("Station {} disconnected from autoAP", mac);
                } else {
                    info!("Station disconnected from autoAP");
                }
                
                // Start reconfigure task in background
                let device = event.interface.clone();
                let disconnect_wait = self.config.disconnect_wait;
                tokio::spawn(async move {
                    if let Err(e) = Self::reconfigure_wpa_supplicant_static(&device, disconnect_wait).await {
                        error!("Failed to reconfigure wpa_supplicant: {}", e);
                    }
                });
            }
            WpaState::ApStaConnected => {
                if let Some(mac) = &event.mac_address {
                    info!("Station {} connected to autoAP", mac);
                } else {
                    info!("Station connected to autoAP");
                }
                
                // Cancel any waiting reconfigure since someone connected
                if let Err(e) = self.touch_unlock_file().await {
                    warn!("Failed to create unlock file: {}", e);
                }
            }
            WpaState::Disconnected => {
                info!("Disconnected from network");
                if Self::is_client(&event.interface).await? {
                    info!("Client disconnected, configuring as AP");
                    Self::configure_ap(&event.interface).await?;
                }
            }
        }

        Ok(())
    }
}

impl AutoApHandler {
    async fn log_flags(&self) {
        if !self.config.debug {
            return;
        }

        let lock_status = if Path::new("/var/run/autoAP.locked").exists() {
            match fs::metadata("/var/run/autoAP.locked").await {
                Ok(metadata) => format!("Found: {:?}", metadata.modified()),
                Err(e) => format!("Error reading: {}", e),
            }
        } else {
            "Not found".to_string()
        };

        let unlock_status = if Path::new("/var/run/autoAP.unlock").exists() {
            match fs::metadata("/var/run/autoAP.unlock").await {
                Ok(metadata) => format!("Found: {:?}", metadata.modified()),
                Err(e) => format!("Error reading: {}", e),
            }
        } else {
            "Not found".to_string()
        };

        debug!("autoAP: Lock status 1: {}", lock_status);
        debug!("autoAP: Lock status 2: {}", unlock_status);
    }

    async fn is_client(device: &str) -> Result<bool> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        Ok(Path::new(&network_file).exists())
    }

    async fn is_actually_connected_to_client(device: &str) -> Result<bool> {
        // Check if we're in station mode using wpa_cli status
        let output = tokio::process::Command::new("/sbin/wpa_cli")
            .args(["-i", device, "status"])
            .output()
            .await
            .context("Failed to get wpa_cli status")?;

        if !output.status.success() {
            return Ok(false);
        }

        let status = String::from_utf8_lossy(&output.stdout);
        
        // Look for actual client connection indicators
        let has_ssid = status.lines().any(|line| {
            line.starts_with("ssid=") && !line.trim_end().ends_with("=")
        });
        
        let wpa_state_completed = status.lines().any(|line| {
            line.starts_with("wpa_state=") && line.contains("COMPLETED")
        });
        
        // Check if we have an IP address using ip command (works with systemd-resolved)
        let has_ip_address = Self::check_device_has_ip(device).await.unwrap_or(false);
        
        // Also check that we're not in AP mode by looking at the mode
        let not_in_ap_mode = !status.lines().any(|line| {
            line.contains("mode=AP") || line.contains("wpa_state=INTERFACE_DISABLED")
        });
        
        // We're only truly connected if we have an SSID, IP address, completed state, and not in AP mode
        let is_connected = has_ssid && has_ip_address && wpa_state_completed && not_in_ap_mode;
        
        if is_connected {
            debug!("Connection check - SSID: {}, IP: {}, State: {}, Not AP: {}, Connected: {}", 
                   has_ssid, has_ip_address, wpa_state_completed, not_in_ap_mode, is_connected);
            debug!("wpa_cli status output: {}", status);
        }
        
        Ok(is_connected)
    }

    async fn check_device_has_ip(device: &str) -> Result<bool> {
        let output = tokio::process::Command::new("ip")
            .args(["addr", "show", device])
            .output()
            .await
            .context("Failed to get device IP address")?;

        if !output.status.success() {
            return Ok(false);
        }

        let ip_output = String::from_utf8_lossy(&output.stdout);
        
        // Look for inet addresses that aren't link-local (169.254.x.x) or loopback
        let has_valid_ip = ip_output.lines().any(|line| {
            if line.trim().starts_with("inet ") && !line.contains("127.0.0.1") {
                // Extract the IP address part
                if let Some(ip_part) = line.trim().split_whitespace().nth(1) {
                    if let Some(ip) = ip_part.split('/').next() {
                        // Skip link-local addresses (169.254.x.x)
                        return !ip.starts_with("169.254.");
                    }
                }
            }
            false
        });

        if has_valid_ip {
            debug!("Device {} has valid IP address", device);
        }

        Ok(has_valid_ip)
    }

    async fn configure_ap(device: &str) -> Result<()> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        let backup_file = format!("/etc/systemd/network/11-{}.network~", device);

        if Path::new(&network_file).exists() {
            info!("Configuring {} as an Access Point", device);
            
            fs::rename(&network_file, &backup_file).await
                .context("Failed to backup network file")?;

            Self::restart_systemd_networkd().await?;
            
            // Force wpa_supplicant to reconfigure and switch to AP mode
            info!("Forcing wpa_supplicant to reconfigure for AP mode");
            match Self::wpa_cli_command_with_output(device, &["reconfigure"]).await {
                Ok(output) => {
                    info!("wpa_cli reconfigure succeeded: {}", output);
                }
                Err(e) => {
                    error!("wpa_cli reconfigure failed: {}", e);
                    return Err(anyhow::anyhow!("wpa_cli reconfigure failed - will be retried by systemd: {}", e));
                }
            }
            
            // Call hook
            if let Err(e) = Self::on_access_point_mode(device).await {
                warn!("Access Point mode hook failed: {}", e);
            }
        }

        Ok(())
    }

    async fn configure_client(device: &str) -> Result<()> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        let backup_file = format!("/etc/systemd/network/11-{}.network~", device);

        if Path::new(&backup_file).exists() {
            info!("Configuring {} as a Wireless Client", device);
            
            fs::rename(&backup_file, &network_file).await
                .context("Failed to restore network file")?;

            Self::restart_systemd_networkd().await?;
            
            // Call hook
            if let Err(e) = Self::on_client_mode(device).await {
                warn!("Client mode hook failed: {}", e);
            }
        }

        Ok(())
    }

    async fn on_access_point_mode(device: &str) -> Result<()> {
        info!("Executing Access Point mode hooks for {}", device);
        Ok(())
    }

    async fn on_client_mode(device: &str) -> Result<()> {
        info!("Executing Client mode hooks for {}", device);
        Ok(())
    }

    async fn restart_systemd_networkd() -> Result<()> {
        if !crate::utils::is_systemd_networkd_active()? {
            warn!("systemd-networkd is not active, attempting to start it");
            systemctl_command(&["start", "systemd-networkd"])
                .context("Failed to start systemd-networkd")?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        systemctl_command(&["restart", "systemd-networkd"])
    }

    async fn wpa_cli_command_with_output(device: &str, args: &[&str]) -> Result<String> {
        let output = tokio::process::Command::new("/sbin/wpa_cli")
            .arg("-i")
            .arg(device)
            .args(args)
            .output()
            .await
            .context("Failed to execute wpa_cli command")?;

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "wpa_cli command failed with exit code: {} - stdout: '{}' - stderr: '{}'",
                output.status.code().unwrap_or(-1),
                stdout,
                stderr
            ));
        }

        if !stderr.is_empty() {
            warn!("wpa_cli command stderr: {}", stderr);
        }

        Ok(stdout)
    }

    async fn reconfigure_wpa_supplicant_static(device: &str, wait_seconds: u64) -> Result<()> {
        let lock_file = "/var/run/autoAP.locked";
        let unlock_file = "/var/run/autoAP.unlock";

        if Path::new(lock_file).exists() {
            info!("Reconfigure already locked. Unlocking...");
            fs::File::create(unlock_file).await
                .context("Failed to create unlock file")?;
            return Ok(());
        }

        // Create lock file
        fs::File::create(lock_file).await
            .context("Failed to create lock file")?;
        
        // Remove unlock file if it exists
        if Path::new(unlock_file).exists() {
            fs::remove_file(unlock_file).await
                .context("Failed to remove unlock file")?;
        }

        info!("Starting reconfigure wait loop for {} seconds", wait_seconds);

        for _i in 0..=wait_seconds {
            sleep(Duration::from_secs(1)).await;
            
            if Path::new(unlock_file).exists() {
                info!("Reconfigure wait unlocked");
                let _ = fs::remove_file(unlock_file).await;
                let _ = fs::remove_file(lock_file).await;
                return Ok(());
            }
        }

        // Completed loop, check for reconfigure
        let _ = fs::remove_file(unlock_file).await;
        let _ = fs::remove_file(lock_file).await;

        info!("Checking wpa reconfigure after wait loop");
        
        // Check if any stations are connected
        if !has_connected_stations(device).unwrap_or(true) {
            info!("No stations connected; performing wpa reconfigure");
            if let Err(e) = wpa_cli_command(device, &["reconfigure"]) {
                error!("wpa_cli reconfigure failed: {}", e);
            }
        }

        Ok(())
    }

    async fn touch_unlock_file(&self) -> Result<()> {
        fs::File::create("/var/run/autoAP.unlock").await
            .context("Failed to create unlock file")?;
        Ok(())
    }
}