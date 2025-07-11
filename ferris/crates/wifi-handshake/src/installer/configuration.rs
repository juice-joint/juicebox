use anyhow::Result;
use dialoguer::Input;
use tracing::{info, warn};

use crate::config::ApConfig;

pub struct ConfigurationStep;

impl ConfigurationStep {
    pub fn new() -> Self {
        Self
    }

    pub fn gather_config(&self) -> Result<ApConfig> {
        info!("Step 2: Gathering Access Point configuration...");
        
        let ap_config = self.prompt_ap_config()?;
        
        info!("✓ Configuration gathered");
        Ok(ap_config)
    }

    fn prompt_ap_config(&self) -> Result<ApConfig> {
        let ssid: String = Input::new()
            .with_prompt("SSID for Access Point mode")
            .interact_text()?;

        let psk: String = loop {
            let password: String = Input::new()
                .with_prompt("Password for Access Point mode")
                .interact_text()?;
            
            let cleaned_password = password.replace('"', "");
            if cleaned_password.len() < 8 || cleaned_password.len() > 63 {
                warn!("Access Point password must be 8-63 characters long. You entered {} characters.", cleaned_password.len());
                continue;
            }
            break cleaned_password;
        };

        let ip_address: String = Input::new()
            .with_prompt("IPv4 address for Access Point mode")
            .default("192.168.16.1".to_string())
            .interact_text()?;

        let config = ApConfig { ssid, psk, ip_address };
        Ok(config)
    }
}