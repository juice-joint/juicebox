use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::{installer::InstallerStep, utils::{is_systemd_networkd_active, is_systemd_resolved_active}};

pub struct VerificationStep;

impl InstallerStep for VerificationStep {
    fn execute(&self) -> Result<()> {
        info!("Step 8: Verifying installation...");

        self.check_systemd_services()?;
        self.check_service_enablement()?;
        self.test_autoap_binary()?;

        info!("✓ Installation verification completed");
        Ok(())
    }
}

impl VerificationStep {
    pub fn new() -> Self {
        Self
    }

    fn check_systemd_services(&self) -> Result<()> {
        // Check that systemd-networkd is still running
        if !is_systemd_networkd_active()? {
            return Err(anyhow::anyhow!("systemd-networkd is not active after installation"));
        }
        info!("systemd-networkd is active ✓");

        // Check that systemd-resolved is running
        if !is_systemd_resolved_active()? {
            return Err(anyhow::anyhow!("systemd-resolved is not active after installation"));
        }
        info!("systemd-resolved is active ✓");

        Ok(())
    }

    fn check_service_enablement(&self) -> Result<()> {
        let services_to_check = [
            "wpa_supplicant@wlan0",
            "wpa-autoap@wlan0",
            "wpa-autoap-restore",
            "systemd-networkd",
            "systemd-resolved"
        ];

        for service in &services_to_check {
            let output = std::process::Command::new("systemctl")
                .args(["is-enabled", service])
                .output()
                .context("Failed to check service status")?;
            
            if !output.status.success() {
                warn!("Service {} is not enabled", service);
            } else {
                info!("Service {} is enabled ✓", service);
            }
        }

        Ok(())
    }

    fn test_autoap_binary(&self) -> Result<()> {
        let output = std::process::Command::new("/usr/local/bin/autoap")
            .args(["--help"])
            .output()
            .context("Failed to test autoap binary")?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("autoap binary is not working properly"));
        }

        info!("autoap binary is working ✓");
        Ok(())
    }
}