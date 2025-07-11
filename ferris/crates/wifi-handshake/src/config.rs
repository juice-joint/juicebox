use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
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
pub struct ApConfig {
    pub ssid: String,
    pub psk: String,
    pub ip_address: String,
}

impl AutoApConfig {
    pub fn load() -> Result<Self> {
        let config_path = "/usr/local/bin/autoAP.conf";
        
        if !Path::new(config_path).exists() {
            info!("Config file not found at {}, using defaults", config_path);
            return Ok(Self::default());
        }

        let content = fs::read_to_string(config_path)
            .context("Failed to read autoAP config file")?;

        // Parse the bash-style config file
        Self::parse_bash_config(&content)
    }

    pub fn save(&self) -> Result<()> {
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

        fs::write(config_path, content)
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