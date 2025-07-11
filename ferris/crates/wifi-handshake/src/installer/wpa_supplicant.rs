use anyhow::{Context, Result};
use std::fs;
use tracing::info;

use crate::config::ApConfig;
use crate::installer::InstallerStep;
use crate::utils::{backup_file, write_file};

pub struct WpaSupplicantStep<'a> {
    config: &'a ApConfig,
}

impl<'a> InstallerStep for WpaSupplicantStep<'a> {
    fn execute(&self) -> Result<()> {
        info!("Step 4: Setting up wpa_supplicant configuration...");

        // Find existing wpa_supplicant config and backup if exists
        let original_config = if std::path::Path::new("/etc/wpa_supplicant/wpa_supplicant.conf").exists() {
            "/etc/wpa_supplicant/wpa_supplicant.conf"
        } else {
            "/etc/wpa_supplicant/wpa_supplicant-wlan0.conf"
        };

        // Backup original config
        if std::path::Path::new(original_config).exists() {
            let backup_path = format!("{}-orig", original_config);
            backup_file(original_config)?;
            
            fs::rename(original_config, &backup_path)
                .context("Failed to backup original wpa_supplicant config")?;
            
            info!("Renamed {} to {}", original_config, backup_path);
        }

        // Create new wpa_supplicant-wlan0.conf with AP config and placeholder for WiFi
        let wpa_config = format!(
            r#"country=US
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
update_config=1
ap_scan=1

# WiFi client networks will be dynamically managed
# Add your WiFi networks here or use a management interface

### autoAP access point ###
network={{
    ssid="{}"
    mode=2
    key_mgmt=WPA-PSK
    psk="{}"
    frequency=2462
}}
"#,
            self.config.ssid.replace('"', ""), 
            self.config.psk.replace('"', "")
        );

        write_file("/etc/wpa_supplicant/wpa_supplicant-wlan0.conf", &wpa_config)?;
        info!("Created /etc/wpa_supplicant/wpa_supplicant-wlan0.conf (AP-only mode)");

        info!("✓ wpa_supplicant configuration completed");
        Ok(())
    }
}

impl<'a> WpaSupplicantStep<'a> {
    pub fn new(config: &'a ApConfig) -> Self {
        Self { config }
    }
}