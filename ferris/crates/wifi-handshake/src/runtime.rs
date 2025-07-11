use anyhow::{Context, Result};
use std::path::Path;
use tokio::fs;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

use crate::config::AutoApConfig;
use crate::utils::{has_connected_stations, is_station_mode, systemctl_command, wpa_cli_command};

#[derive(Debug, Clone)]
pub enum WifiState {
    ApEnabled,
    ApDisabled,
    Connected,
    ApStaConnected,
    ApStaDisconnected,
    Disconnected,
}

impl std::str::FromStr for WifiState {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "AP-ENABLED" => Ok(WifiState::ApEnabled),
            "AP-DISABLED" => Ok(WifiState::ApDisabled),
            "CONNECTED" => Ok(WifiState::Connected),
            "AP-STA-CONNECTED" => Ok(WifiState::ApStaConnected),
            "AP-STA-DISCONNECTED" => Ok(WifiState::ApStaDisconnected),
            "DISCONNECTED" => Ok(WifiState::Disconnected),
            _ => Err(anyhow::anyhow!("Unrecognized WiFi state: {}", s)),
        }
    }
}

pub struct AutoAp {
    config: AutoApConfig,
}

impl AutoAp {
    pub async fn new() -> Result<Self> {
        let config = AutoApConfig::load().await?;
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

        // Normal state change handling: autoap wlan0 STATE [mac]
        if args.len() < 3 {
            return Err(anyhow::anyhow!("State change requires device and state"));
        }
        
        let device = &args[1];
        let state_str = &args[2];
        let mac_address = args.get(3).map(|s| s.as_str());
        
        let state: WifiState = state_str.parse()
            .context("Failed to parse WiFi state")?;
        
        info!("autoAP {} state {:?} {:?}", device, state, mac_address);
        self.handle_state_change(device, state, mac_address).await?;

        Ok(())
    }

    async fn handle_state_change(
        &self,
        device: &str,
        state: WifiState,
        mac_address: Option<&str>,
    ) -> Result<()> {
        self.log_flags().await;

        match state {
            WifiState::ApEnabled => {
                info!("AP enabled, configuring access point");
                self.configure_ap(device).await?;
                
                // Start reconfigure task in background
                let device = device.to_string();
                let enable_wait = self.config.enable_wait;
                tokio::spawn(async move {
                    if let Err(e) = Self::reconfigure_wpa_supplicant_static(&device, enable_wait).await {
                        error!("Failed to reconfigure wpa_supplicant: {}", e);
                    }
                });
            }
            WifiState::ApDisabled => {
                info!("AP disabled");
                // No specific action needed in original script
            }
            WifiState::Connected => {
                info!("Connected to network");
                
                // Verify we're actually connected to a client network, not just AP mode
                if self.is_actually_connected_to_client(device).await? {
                    info!("CONNECTED in station mode, configuring client");
                    self.configure_client(device).await?;
                } else {
                    info!("CONNECTED event received but not actually connected to client network, ignoring");
                }
            }
            WifiState::ApStaDisconnected => {
                if let Some(mac) = mac_address {
                    info!("Station {} disconnected from autoAP", mac);
                } else {
                    info!("Station disconnected from autoAP");
                }
                
                // Start reconfigure task in background
                let device = device.to_string();
                let disconnect_wait = self.config.disconnect_wait;
                tokio::spawn(async move {
                    if let Err(e) = Self::reconfigure_wpa_supplicant_static(&device, disconnect_wait).await {
                        error!("Failed to reconfigure wpa_supplicant: {}", e);
                    }
                });
            }
            WifiState::ApStaConnected => {
                if let Some(mac) = mac_address {
                    info!("Station {} connected to autoAP", mac);
                } else {
                    info!("Station connected to autoAP");
                }
                
                // Cancel any waiting reconfigure since someone connected
                if let Err(e) = self.touch_unlock_file().await {
                    warn!("Failed to create unlock file: {}", e);
                }
            }
            WifiState::Disconnected => {
                info!("Disconnected from network");
                if self.is_client(device).await? {
                    info!("Client disconnected, configuring as AP");
                    self.configure_ap(device).await?;
                }
            }
        }

        Ok(())
    }

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

    async fn is_client(&self, device: &str) -> Result<bool> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        Ok(Path::new(&network_file).exists())
    }

    async fn is_actually_connected_to_client(&self, device: &str) -> Result<bool> {
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
        let has_ip_address = self.check_device_has_ip(device).await.unwrap_or(false);
        
        // Also check that we're not in AP mode by looking at the mode
        let not_in_ap_mode = !status.lines().any(|line| {
            line.contains("mode=AP") || line.contains("wpa_state=INTERFACE_DISABLED")
        });
        
        // We're only truly connected if we have an SSID, IP address, completed state, and not in AP mode
        let is_connected = has_ssid && has_ip_address && wpa_state_completed && not_in_ap_mode;
        
        if self.config.debug {
            debug!("Connection check - SSID: {}, IP: {}, State: {}, Not AP: {}, Connected: {}", 
                   has_ssid, has_ip_address, wpa_state_completed, not_in_ap_mode, is_connected);
            debug!("wpa_cli status output: {}", status);
        }
        
        Ok(is_connected)
    }

    async fn check_device_has_ip(&self, device: &str) -> Result<bool> {
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

        if self.config.debug && has_valid_ip {
            debug!("Device {} has valid IP address", device);
        }

        Ok(has_valid_ip)
    }

    async fn configure_ap(&self, device: &str) -> Result<()> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        let backup_file = format!("/etc/systemd/network/11-{}.network~", device);

        if Path::new(&network_file).exists() {
            info!("Configuring {} as an Access Point", device);
            
            fs::rename(&network_file, &backup_file).await
                .context("Failed to backup network file")?;

            self.restart_systemd_networkd().await?;
            
            // Force wpa_supplicant to reconfigure and switch to AP mode
            info!("Forcing wpa_supplicant to reconfigure for AP mode");
            if let Err(e) = wpa_cli_command(device, &["reconfigure"]).await {
                warn!("wpa_cli reconfigure failed: {}", e);
            }
            
            // Call Rust function directly instead of bash script
            if let Err(e) = self.on_access_point_mode(device).await {
                warn!("Access Point mode hook failed: {}", e);
            }
        }

        Ok(())
    }

    async fn configure_client(&self, device: &str) -> Result<()> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        let backup_file = format!("/etc/systemd/network/11-{}.network~", device);

        if Path::new(&backup_file).exists() {
            info!("Configuring {} as a Wireless Client", device);
            
            fs::rename(&backup_file, &network_file).await
                .context("Failed to restore network file")?;

            self.restart_systemd_networkd().await?;
            
            // Call Rust function directly instead of bash script
            if let Err(e) = self.on_client_mode(device).await {
                warn!("Client mode hook failed: {}", e);
            }
        }

        Ok(())
    }

    /// Called when switching to Access Point mode
    /// Replace this with your actual functionality
    async fn on_access_point_mode(&self, device: &str) -> Result<()> {
        info!("Executing Access Point mode hooks for {}", device);
        
        // Add your custom logic here
        // Examples:
        // - Start captive portal web server
        // - Configure DHCP server settings
        // - Start local services
        // - Update LED indicators
        // - Send notifications
        
        Ok(())
    }

    /// Called when switching to Client mode (connected to WiFi)
    /// Replace this with your actual functionality
    async fn on_client_mode(&self, device: &str) -> Result<()> {
        info!("Executing Client mode hooks for {}", device);
        
        // Add your custom logic here
        // Examples:
        // - Stop captive portal
        // - Sync data to cloud
        // - Update system time via NTP
        // - Start internet-dependent services
        // - Update LED indicators
        // - Send "online" notifications
        
        Ok(())
    }

    async fn restart_systemd_networkd(&self) -> Result<()> {
        // Check if systemd-networkd is enabled/running first
        if !crate::utils::is_systemd_networkd_active().await? {
            warn!("systemd-networkd is not active, attempting to start it");
            systemctl_command(&["start", "systemd-networkd"]).await
                .context("Failed to start systemd-networkd")?;
            
            // Give it a moment to start
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        systemctl_command(&["restart", "systemd-networkd"]).await
    }

    // Static version for use in spawned tasks
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
        if !has_connected_stations(device).await.unwrap_or(true) {
            info!("No stations connected; performing wpa reconfigure");
            if let Err(e) = wpa_cli_command(device, &["reconfigure"]).await {
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

    pub async fn reset(&self) -> Result<()> {
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

        info!("wpa_supplicant online, starting wpa_cli to monitor wpa_supplicant messages");
        
        // Get the current binary path
        let current_exe = std::env::current_exe()
            .context("Failed to get current executable path")?;
        
        // In the original script, this would exec wpa_cli
        // We'll spawn wpa_cli and have it call back to our binary
        let mut child = tokio::process::Command::new("/sbin/wpa_cli")
            .args(["-i", device, "-a", current_exe.to_str().unwrap()])
            .spawn()
            .context("Failed to spawn wpa_cli")?;

        info!("wpa_cli spawned with PID: {}", child.id().unwrap_or(0));
        
        // Wait for the child process
        let status = child.wait().await
            .context("Failed to wait for wpa_cli")?;
            
        if !status.success() {
            return Err(anyhow::anyhow!("wpa_cli exited with error: {}", status));
        }
        
        Ok(())
    }
}