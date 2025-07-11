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
                if is_station_mode(device).await? {
                    info!("CONNECTED in station mode, configuring client");
                    self.configure_client(device).await?;
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

    async fn configure_ap(&self, device: &str) -> Result<()> {
        let network_file = format!("/etc/systemd/network/11-{}.network", device);
        let backup_file = format!("/etc/systemd/network/11-{}.network~", device);

        if Path::new(&network_file).exists() {
            info!("Configuring {} as an Access Point", device);
            
            fs::rename(&network_file, &backup_file).await
                .context("Failed to backup network file")?;

            self.restart_systemd_networkd().await?;
            
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
        
        Ok(())
    }

    /// Called when switching to Client mode (connected to WiFi)
    /// Replace this with your actual functionality
    async fn on_client_mode(&self, device: &str) -> Result<()> {
        info!("Executing Client mode hooks for {}", device);
        
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