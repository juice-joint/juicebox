use anyhow::Result;
use tracing::info;

mod system_check;
mod configuration;
mod confirmation;
mod wpa_supplicant;
mod systemd_network;
mod autoap;
mod service_config;
mod verify;

use system_check::SystemCheckStep;
use configuration::ConfigurationStep;
use confirmation::ConfirmationStep;
use wpa_supplicant::WpaSupplicantStep;
use systemd_network::SystemdNetworkStep;
use autoap::SystemdServicesStep;
use service_config::ServiceConfigStep;
use verify::VerificationStep;

pub struct Installer;

impl Installer {
    pub fn new() -> Self {
        Self
    }

    pub async fn install(&self) -> Result<()> {
        info!("Starting autoAP installation...");

        // Step 1: Check system requirements
        let system_check = SystemCheckStep::new();
        system_check.execute()?;

        // Step 2: Gather configuration
        let config_step = ConfigurationStep::new();
        let ap_config = config_step.gather_config()?;

        // Step 3: Confirm installation
        let confirmation_step = ConfirmationStep::new(&ap_config);
        confirmation_step.execute()?;

        // Step 4: Setup wpa_supplicant
        let wpa_step = WpaSupplicantStep::new(&ap_config);
        wpa_step.execute()?;

        // Step 5: Setup systemd network
        let network_step = SystemdNetworkStep::new(&ap_config);
        network_step.execute()?;

        // Step 6: Setup systemd services
        let services_step = SystemdServicesStep::new();
        services_step.execute()?;

        // Step 7: Configure services
        let service_config_step = ServiceConfigStep::new();
        service_config_step.execute()?;

        // Step 8: Verify installation
        let verification_step = VerificationStep::new();
        verification_step.execute()?;

        info!("autoAP installation completed successfully!");
        info!("Please reboot the system for the configuration changes to take effect");

        Ok(())
    }
}

pub trait InstallerStep {
    fn execute(&self) -> Result<()>;
}