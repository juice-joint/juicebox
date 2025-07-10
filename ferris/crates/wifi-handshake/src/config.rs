use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoApConfig {
    /// Seconds to wait in AP mode when AP enabled before retrying WiFi
    pub enable_wait: u64,
    /// Seconds to wait after last AP client disconnects before retrying WiFi
    pub disconnect_wait: u64,
    /// Debug logging enabled
    pub debug: bool,
}

impl Default for AutoApConfig {
    fn default() -> Self {
        Self {
            enable_wait: 300,
            disconnect_wait: 20,
            debug: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WifiConfig {
    pub country: String,
    pub ssid: String,
    pub psk: String,
}

#[derive(Debug, Clone)]
pub struct ApConfig {
    pub ssid: String,
    pub psk: String,
    pub ip_address: String,
}

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub wifi: WifiConfig,
    pub access_point: ApConfig,
    pub autoap: AutoApConfig,
}

impl AutoApConfig {
    pub async fn load() -> Result<Self> {
        let config_path = "/usr/local/bin/autoAP.conf";
        
        if !Path::new(config_path).exists() {
            info!("Config file not found at {}, using defaults", config_path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_path).await
            .context("Failed to read autoAP config file")?;

        // Parse the bash-style config file
        Self::parse_bash_config(&content)
    }

    pub async fn save(&self) -> Result<()> {
        let config_path = "/usr/local/bin/autoAP.conf";
        
        let content = format!(
            r#"#
# enablewait
#  In AP mode, number of seconds to wait before retrying regular WiFi connection
#
enablewait={}
#
# disconnectwait
#  number of seconds to wait after last AP client disconnects before retrying regular WiFi connection
#
disconnectwait={}
#
# debug logging
#  0:debug logging on
#  1:debug logging off
#
debug={}
"#,
            self.enable_wait,
            self.disconnect_wait,
            if self.debug { 0 } else { 1 }
        );

        fs::write(config_path, content).await
            .context("Failed to write autoAP config file")?;

        Ok(())
    }

    fn parse_bash_config(content: &str) -> Result<Self> {
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "enablewait" => {
                        config.enable_wait = value.trim().parse()
                            .context("Failed to parse enablewait")?;
                    }
                    "disconnectwait" => {
                        config.disconnect_wait = value.trim().parse()
                            .context("Failed to parse disconnectwait")?;
                    }
                    "debug" => {
                        // In bash config: 0=debug on, 1=debug off
                        let debug_val: u32 = value.trim().parse()
                            .context("Failed to parse debug flag")?;
                        config.debug = debug_val == 0;
                    }
                    _ => {
                        warn!("Unknown config key: {}", key);
                    }
                }
            }
        }
        
        Ok(config)
    }
}

impl WifiConfig {
    pub fn sanitize_strings(&mut self) {
        // Remove quotes from SSID and PSK like the bash script does
        self.ssid = self.ssid.replace('"', "");
        self.psk = self.psk.replace('"', "");
    }
}

impl ApConfig {
    pub fn sanitize_strings(&mut self) {
        // Remove quotes from SSID and PSK like the bash script does
        self.ssid = self.ssid.replace('"', "");
        self.psk = self.psk.replace('"', "");
    }
}

impl InstallConfig {
    pub async fn parse_existing_wpa_config() -> Result<Option<WifiConfig>> {
        let possible_paths = [
            "/etc/wpa_supplicant/wpa_supplicant.conf",
            "/etc/wpa_supplicant/wpa_supplicant-wlan0.conf",
        ];

        for path in &possible_paths {
            if Path::new(path).exists() {
                info!("Found existing wpa_supplicant config at: {}", path);
                
                let content = fs::read_to_string(path).await
                    .context("Failed to read wpa_supplicant config")?;
                
                return Ok(Self::extract_wifi_config(&content)?);
            }
        }

        Ok(None)
    }

    fn extract_wifi_config(content: &str) -> Result<Option<WifiConfig>> {
        let mut country = None;
        let mut ssid = None;
        let mut psk = None;

        for line in content.lines() {
            let line = line.trim();
            
            if line.starts_with("country=") {
                country = Some(line.split('=').nth(1).unwrap_or("").trim().to_string());
            } else if line.starts_with("ssid=") {
                let value = line.split('=').nth(1).unwrap_or("").trim();
                ssid = Some(value.trim_matches('"').to_string());
            } else if line.starts_with("psk=") {
                let value = line.split('=').nth(1).unwrap_or("").trim();
                psk = Some(value.trim_matches('"').to_string());
            }
        }

        if let (Some(country), Some(ssid), Some(psk)) = (country, ssid, psk) {
            Ok(Some(WifiConfig { country, ssid, psk }))
        } else {
            Ok(None)
        }
    }
}