use anyhow::{Context, Result};
use regex::Regex;
use std::fs;
use tracing::info;

use crate::utils::{backup_file, write_file};

pub struct WpaSupplicantManager {
    config_path: String,
}

impl WpaSupplicantManager {
    pub fn new() -> Self {
        Self {
            config_path: "/etc/wpa_supplicant/wpa_supplicant-wlan0.conf".to_string(),
        }
    }

    pub fn with_config_path(path: String) -> Self {
        Self {
            config_path: path,
        }
    }

    pub fn update_network(&self, ssid: &str, password: &str) -> Result<()> {
        let content = self.read_config()?;
        let updated_content = self.update_or_add_network(&content, ssid, password)?;
        self.write_config(&updated_content)?;
        self.reload_wpa_supplicant()?;
        Ok(())
    }

    fn read_config(&self) -> Result<String> {
        fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read wpa_supplicant config from {}", self.config_path))
    }

    fn write_config(&self, content: &str) -> Result<()> {
        backup_file(&self.config_path)?;
        write_file(&self.config_path, content)
            .with_context(|| format!("Failed to write wpa_supplicant config to {}", self.config_path))
    }

    fn update_or_add_network(&self, content: &str, ssid: &str, password: &str) -> Result<String> {
        let escaped_ssid = ssid.replace('"', "");
        let escaped_password = password.replace('"', "");

        let network_pattern = Regex::new(r"(?s)network=\{[^}]*ssid=['\x22]?([^'\x22\s}]+)['\x22]?[^}]*\}")
            .context("Failed to compile network regex")?;

        let mut found_existing = false;
        let mut result = String::new();
        let mut last_end = 0;

        for mat in network_pattern.find_iter(content) {
            let network_block = mat.as_str();
            
            if self.network_matches_ssid(network_block, &escaped_ssid)? {
                result.push_str(&content[last_end..mat.start()]);
                result.push_str(&self.create_client_network_block(&escaped_ssid, &escaped_password));
                found_existing = true;
            } else {
                result.push_str(&content[last_end..mat.end()]);
            }
            last_end = mat.end();
        }

        result.push_str(&content[last_end..]);

        if !found_existing {
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push('\n');
            result.push_str(&self.create_client_network_block(&escaped_ssid, &escaped_password));
        }

        Ok(result)
    }

    fn network_matches_ssid(&self, network_block: &str, target_ssid: &str) -> Result<bool> {
        let ssid_pattern = Regex::new(r"ssid=['\x22]?([^'\x22\s}]+)['\x22]?")
            .context("Failed to compile SSID regex")?;
        
        if let Some(captures) = ssid_pattern.captures(network_block) {
            if let Some(ssid_match) = captures.get(1) {
                return Ok(ssid_match.as_str() == target_ssid);
            }
        }
        Ok(false)
    }

    fn create_client_network_block(&self, ssid: &str, password: &str) -> String {
        format!(
            r#"network={{
    ssid="{}"
    psk="{}"
    key_mgmt=WPA-PSK
}}
"#,
            ssid, password
        )
    }

    fn reload_wpa_supplicant(&self) -> Result<()> {
        let output = std::process::Command::new("wpa_cli")
            .args(&["-i", "wlan0", "reconfigure"])
            .output()
            .context("Failed to execute wpa_cli reconfigure")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("wpa_cli reconfigure failed: {}", stderr));
        }

        info!("wpa_supplicant configuration reloaded successfully");
        Ok(())
    }
}