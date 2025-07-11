use anyhow::Result;
use dialoguer::Confirm;
use tracing::info;

use crate::{config::ApConfig, installer::InstallerStep};

pub struct ConfirmationStep<'a> {
    config: &'a ApConfig,
}

impl<'a> ConfirmationStep<'a> {
    pub fn new(config: &'a ApConfig) -> Self {
        Self { config }
    }
}

impl<'a> InstallerStep for ConfirmationStep<'a> {
    fn execute(&self) -> Result<()> {
        info!("Step 3: Confirming installation...");

        println!("\n        autoAP Configuration (AP-only mode)");
        println!(" Access Point SSID:     {}", self.config.ssid);
        println!(" Access Point password: {}", self.config.psk);
        println!(" Access Point IP addr:  {}", self.config.ip_address);
        println!();
        println!(" Note: WiFi client configuration will be handled separately");
        println!();

        if !Confirm::new()
            .with_prompt("Are you ready to proceed?")
            .interact()?
        {
            return Err(anyhow::anyhow!("Installation cancelled by user"));
        }

        info!("✓ Installation confirmed");
        Ok(())
    }
}