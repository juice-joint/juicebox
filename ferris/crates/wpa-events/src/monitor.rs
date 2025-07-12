use anyhow::{Context, Result};
use std::path::Path;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::event::WpaEvent;
use crate::handler::WpaEventHandler;

/// Simple monitor for wpa_supplicant events
pub struct WpaEventMonitor<H: WpaEventHandler> {
    interface: String,
    handler: H,
}

impl<H: WpaEventHandler> WpaEventMonitor<H> {
    /// Create a new WPA event monitor
    pub fn new(interface: &str, handler: H) -> Result<Self> {
        Ok(Self {
            interface: interface.to_string(),
            handler,
        })
    }

    /// Start monitoring wpa_supplicant events
    pub async fn start(&self) -> Result<()> {
        self.wait_for_wpa_supplicant().await?;

        info!("Starting wpa_cli monitor for {}", self.interface);

        // Get the current binary path to use as action script
        let current_exe = std::env::current_exe()
            .context("Failed to get current executable path")?;

        // Start wpa_cli with action script
        let mut child = tokio::process::Command::new("/sbin/wpa_cli")
            .args(["-i", &self.interface, "-a", current_exe.to_str().unwrap()])
            .spawn()
            .context("Failed to spawn wpa_cli")?;

        info!("wpa_cli started for {}", self.interface);

        // Wait for the child process
        let status = child.wait().await
            .context("Failed to wait for wpa_cli")?;

        if !status.success() {
            return Err(anyhow::anyhow!("wpa_cli exited with error: {}", status));
        }

        Ok(())
    }

    /// Process a single event from wpa_cli (called as action script)
    pub async fn process_event(&self, args: Vec<String>) -> Result<()> {
        // Handle special control commands
        if args.len() >= 2 {
            match args[1].as_str() {
                "reset" => {
                    info!("Reset command received for {}", self.interface);
                    return self.reset().await;
                }
                "start" if args.len() >= 3 => {
                    let device = &args[2];
                    info!("Start command received for device: {}", device);
                    return self.start().await;
                }
                _ => {}
            }
        }

        // Parse the event
        let event = WpaEvent::from_args(args)?;
        
        // Validate that the event is for our interface
        if event.interface != self.interface {
            warn!("Received event for interface {} but monitoring {}", 
                  event.interface, self.interface);
            return Ok(());
        }

        info!("Processing WPA event: {}", event);

        // Handle the event
        self.handler.handle_event(event).await?;

        Ok(())
    }

    /// Wait for wpa_supplicant to come online
    async fn wait_for_wpa_supplicant(&self) -> Result<()> {
        let wpa_socket_path = format!("/var/run/wpa_supplicant/{}", self.interface);
        
        while !Path::new(&wpa_socket_path).exists() {
            info!("Waiting for wpa_supplicant to come online for {}", self.interface);
            sleep(Duration::from_millis(500)).await;
        }

        Ok(())
    }

    /// Reset state (remove lock files, restore network config)
    async fn reset(&self) -> Result<()> {
        info!("Resetting state for {}", self.interface);
        
        // Remove lock files if they exist
        let lock_files = [
            "/var/run/autoAP.locked",
            "/var/run/autoAP.unlock",
        ];

        for lock_file in &lock_files {
            if Path::new(lock_file).exists() {
                tokio::fs::remove_file(lock_file).await
                    .with_context(|| format!("Failed to remove lock file: {}", lock_file))?;
            }
        }

        // Restore network configuration if backup exists
        let backup_network = format!("/etc/systemd/network/11-{}.network~", self.interface);
        let network_file = format!("/etc/systemd/network/11-{}.network", self.interface);

        if Path::new(&backup_network).exists() {
            tokio::fs::rename(&backup_network, &network_file).await
                .context("Failed to restore network file")?;
        }

        Ok(())
    }
}