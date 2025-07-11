use anyhow::Result;
use tracing::{info, warn};

use crate::config::AutoApConfig;
use crate::installer::InstallerStep;
use crate::utils::systemctl_command;

pub struct ServiceConfigStep;

impl InstallerStep for ServiceConfigStep {
    fn execute(&self) -> Result<()> {
        info!("Step 7: Configuring systemd services...");

        // Save autoAP configuration
        let autoap_config = AutoApConfig::default();
        autoap_config.save()?;

        // Reload systemd daemon
        systemctl_command(&["daemon-reload"])?;

        self.enable_wpa_supplicant()?;
        self.disable_vanilla_wpa_supplicant()?;
        self.enable_autoap_services()?;

        info!("✓ Service configuration completed");
        Ok(())
    }
}

impl ServiceConfigStep {
    pub fn new() -> Self {
        Self
    }

    fn enable_wpa_supplicant(&self) -> Result<()> {
        info!("Enabling wpa_supplicant@wlan0...");
        systemctl_command(&["enable", "wpa_supplicant@wlan0"])?;
        Ok(())
    }

    fn disable_vanilla_wpa_supplicant(&self) -> Result<()> {
        info!("Disabling (vanilla) wpa_supplicant...");
        systemctl_command(&["disable", "wpa_supplicant"])
            .unwrap_or_else(|e| {
                warn!("Failed to disable wpa_supplicant (may not be enabled): {}", e);
            });
        Ok(())
    }

    fn enable_autoap_services(&self) -> Result<()> {
        info!("Enabling wpa-autoap@wlan0 service...");
        systemctl_command(&["enable", "wpa-autoap@wlan0"])?;

        info!("Enabling wpa-autoap-restore service...");
        systemctl_command(&["enable", "wpa-autoap-restore"])?;
        
        Ok(())
    }
}