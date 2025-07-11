use anyhow::Result;
use std::fs;
use tracing::info;

use crate::config::ApConfig;
use crate::installer::InstallerStep;
use crate::utils::{backup_file, write_file};

pub struct SystemdNetworkStep<'a> {
    config: &'a ApConfig,
}

impl<'a> InstallerStep for SystemdNetworkStep<'a> {
    fn execute(&self) -> Result<()> {
        info!("Step 5: Creating systemd network files...");

        // Backup existing files
        backup_file("/etc/systemd/network/10-eth0.network")?;
        backup_file("/etc/systemd/network/11-wlan0.network")?;
        backup_file("/etc/systemd/network/12-wlan0AP.network")?;

        // Remove any existing backup file that autoAP creates
        if std::path::Path::new("/etc/systemd/network/11-wlan0.network~").exists() {
            fs::remove_file("/etc/systemd/network/11-wlan0.network~")?;
        }

        self.create_ethernet_config()?;
        self.create_wifi_client_config()?;
        self.create_ap_config()?;

        info!("✓ systemd network configuration completed");
        Ok(())
    }
}

impl<'a> SystemdNetworkStep<'a> {
    pub fn new(config: &'a ApConfig) -> Self {
        Self { config }
    }

    fn create_ethernet_config(&self) -> Result<()> {
        let ethernet_config = r#"[Match]
Name=eth0

[Network]
DHCP=ipv4

[DHCP]
RouteMetric=10
UseDomains=yes
UseDNS=yes

"#;
        write_file("/etc/systemd/network/10-eth0.network", ethernet_config)?;
        info!("Created ethernet network configuration");
        Ok(())
    }

    fn create_wifi_client_config(&self) -> Result<()> {
        let client_config = r#"[Match]
Name=wlan0

[Network]
DHCP=ipv4

[DHCP]
RouteMetric=20
UseDomains=yes
UseDNS=yes

"#;
        write_file("/etc/systemd/network/11-wlan0.network", client_config)?;
        info!("Created WiFi client network configuration");
        Ok(())
    }

    fn create_ap_config(&self) -> Result<()> {
        let ap_config_content = format!(
            r#"[Match]
Name=wlan0

[Network]
DHCPServer=yes
Address={}/24

"#,
            self.config.ip_address
        );
        write_file("/etc/systemd/network/12-wlan0AP.network", &ap_config_content)?;
        info!("Created Access Point network configuration");
        Ok(())
    }
}