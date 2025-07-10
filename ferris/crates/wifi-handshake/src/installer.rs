use anyhow::{Context, Result};
use dialoguer::{Confirm, Input};
use tracing::{info, warn};

use crate::config::{ApConfig, AutoApConfig, InstallConfig, WifiConfig};
use crate::utils::{backup_file, is_systemd_networkd_active, make_executable, systemctl_command, write_file};

pub struct Installer;

impl Installer {
    pub fn new() -> Self {
        Self
    }

    pub async fn install(&self) -> Result<()> {
        info!("Starting autoAP installation...");

        // Check systemd-networkd
        self.check_systemd_networkd().await?;

        // Gather configuration
        let config = self.gather_configuration().await?;

        // Confirm installation
        self.confirm_installation(&config).await?;

        // Perform installation
        self.perform_installation(&config).await?;

        info!("autoAP installation completed successfully!");
        info!("Please reboot the system for the configuration changes to take effect");

        Ok(())
    }

    async fn check_systemd_networkd(&self) -> Result<()> {
        if !is_systemd_networkd_active().await? {
            warn!("This system is not configured to use systemd-networkd");
            warn!("You must switch to using systemd-networkd to use autoAP");
            warn!("You can use /usr/local/bin/rpi-networkconfig to reconfigure your networking");
            warn!("rpi-networkconfig will configure wlan0 and eth0 to be DHCP-enabled");
            warn!("This can be done after install-autoAP has completed.");

            if !Confirm::new()
                .with_prompt("Do you want to continue with autoAP installation?")
                .interact()?
            {
                return Err(anyhow::anyhow!("Installation cancelled by user"));
            }
        }

        Ok(())
    }

    async fn gather_configuration(&self) -> Result<InstallConfig> {
        info!("Gathering configuration...");

        // Try to get existing WiFi configuration
        let existing_wifi = InstallConfig::parse_existing_wpa_config().await?;

        let wifi_config = if let Some(mut wifi) = existing_wifi {
            info!("Found existing WiFi configuration");
            wifi.sanitize_strings();

            // Ask if user wants to use existing config
            if Confirm::new()
                .with_prompt(format!(
                    "Use existing WiFi config? (Country: {}, SSID: {})",
                    wifi.country, wifi.ssid
                ))
                .interact()?
            {
                wifi
            } else {
                self.prompt_wifi_config().await?
            }
        } else {
            info!("No existing WiFi configuration found");
            self.prompt_wifi_config().await?
        };

        let ap_config = self.prompt_ap_config().await?;
        let autoap_config = AutoApConfig::default();

        Ok(InstallConfig {
            wifi: wifi_config,
            access_point: ap_config,
            autoap: autoap_config,
        })
    }

    async fn prompt_wifi_config(&self) -> Result<WifiConfig> {
        let country: String = Input::new()
            .with_prompt("Your Country")
            .default("US".to_string())
            .interact_text()?;

        let ssid: String = Input::new()
            .with_prompt("Your WiFi SSID")
            .interact_text()?;

        let psk: String = loop {
            let password: String = Input::new()
                .with_prompt("Your WiFi password")
                .interact_text()?;
            
            let cleaned_password = password.replace('"', "");
            if cleaned_password.len() < 8 || cleaned_password.len() > 63 {
                warn!("WiFi password must be 8-63 characters long. You entered {} characters.", cleaned_password.len());
                continue;
            }
            break cleaned_password;
        };

        let config = WifiConfig { country, ssid, psk };
        Ok(config)
    }

    async fn prompt_ap_config(&self) -> Result<ApConfig> {
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

    async fn confirm_installation(&self, config: &InstallConfig) -> Result<()> {
        println!("\n        autoAP Configuration");
        println!(" Access Point SSID:     {}", config.access_point.ssid);
        println!(" Access Point password: {}", config.access_point.psk);
        println!(" Access Point IP addr:  {}", config.access_point.ip_address);
        println!(" Your WiFi country:     {}", config.wifi.country);
        println!(" Your WiFi SSID:        {}", config.wifi.ssid);
        println!(" Your WiFi password:    {}", config.wifi.psk);
        println!();

        if !Confirm::new()
            .with_prompt("Are you ready to proceed?")
            .interact()?
        {
            return Err(anyhow::anyhow!("Installation cancelled by user"));
        }

        Ok(())
    }

    async fn perform_installation(&self, config: &InstallConfig) -> Result<()> {
        // Backup and create wpa_supplicant config
        self.setup_wpa_supplicant(&config.wifi, &config.access_point).await?;

        // Create systemd network files
        self.setup_systemd_network(&config.access_point).await?;

        // Create systemd service files
        self.setup_systemd_services().await?;

        // Create local script
        self.setup_local_script().await?;

        // Save autoAP configuration
        config.autoap.save().await?;

        // Configure services
        self.configure_services().await?;

        Ok(())
    }

    async fn setup_wpa_supplicant(&self, wifi: &WifiConfig, ap: &ApConfig) -> Result<()> {
        info!("Setting up wpa_supplicant configuration...");

        // Find existing wpa_supplicant config
        let original_config = if std::path::Path::new("/etc/wpa_supplicant/wpa_supplicant.conf").exists() {
            "/etc/wpa_supplicant/wpa_supplicant.conf"
        } else {
            "/etc/wpa_supplicant/wpa_supplicant-wlan0.conf"
        };

        // Backup original config
        if std::path::Path::new(original_config).exists() {
            let backup_path = format!("{}-orig", original_config);
            backup_file(original_config).await?;
            
            // Move original to -orig
            tokio::fs::rename(original_config, &backup_path).await
                .context("Failed to backup original wpa_supplicant config")?;
            
            info!("Renamed {} to {}", original_config, backup_path);
        }

        // Create new wpa_supplicant-wlan0.conf
        let wpa_config = format!(
            r#"country={}
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
update_config=1
ap_scan=1

network={{
    priority=10
    ssid="{}"
    psk="{}"
}}

### autoAP access point ###
network={{
    ssid="{}"
    mode=2
    key_mgmt=WPA-PSK
    psk="{}"
    frequency=2462
}}
"#,
            wifi.country, 
            wifi.ssid.replace('"', ""), 
            wifi.psk.replace('"', ""), 
            ap.ssid.replace('"', ""), 
            ap.psk.replace('"', "")
        );

        write_file("/etc/wpa_supplicant/wpa_supplicant-wlan0.conf", &wpa_config).await?;
        info!("Created /etc/wpa_supplicant/wpa_supplicant-wlan0.conf");

        Ok(())
    }

    async fn setup_systemd_network(&self, ap: &ApConfig) -> Result<()> {
        info!("Creating WiFi network files in /etc/systemd/network...");

        // Backup existing files
        backup_file("/etc/systemd/network/11-wlan0.network").await?;
        backup_file("/etc/systemd/network/12-wlan0AP.network").await?;

        // Remove any existing backup file that autoAP creates
        if std::path::Path::new("/etc/systemd/network/11-wlan0.network~").exists() {
            tokio::fs::remove_file("/etc/systemd/network/11-wlan0.network~").await?;
        }

        // Create client network config
        let client_config = r#"[Match]
Name=wlan0

[Network]
DHCP=ipv4

[DHCP]
RouteMetric=20
UseDomains=yes

"#;
        write_file("/etc/systemd/network/11-wlan0.network", client_config).await?;

        // Create AP network config
        let ap_config_content = format!(
            r#"[Match]
Name=wlan0

[Network]
DHCPServer=yes
Address={}/24

"#,
            ap.ip_address
        );
        write_file("/etc/systemd/network/12-wlan0AP.network", &ap_config_content).await?;

        info!("Created systemd network configuration files");
        Ok(())
    }

    async fn setup_systemd_services(&self) -> Result<()> {
        info!("Creating systemd service files...");

        // Backup existing service files
        backup_file("/etc/systemd/system/wpa-autoap@wlan0.service").await?;
        backup_file("/etc/systemd/system/wpa-autoap-restore.service").await?;

        // Create wpa-autoap@wlan0.service
        let autoap_service = r#"[Unit]
Description=autoAP Automatic Access Point When No WiFi Connection (wpa-autoap@wlan0.service)
#After=network.target network-online.target wpa_supplicant@%i.service sys-subsystem-net-devices-%i.device
Before=wpa_supplicant@%i.service
BindsTo=wpa_supplicant@%i.service

[Service]
Type=simple
ExecStart=/usr/local/bin/autoap start %I
Restart=on-failure
TimeoutSec=1

[Install]
WantedBy=multi-user.target

"#;
        write_file("/etc/systemd/system/wpa-autoap@wlan0.service", autoap_service).await?;

        // Create wpa-autoap-restore.service
        let restore_service = r#"[Unit]
Description=Restore wpa-autoap configuration (wpa-autoap-restore.service)
DefaultDependencies=no
After=local-fs-pre.target

[Service]
Type=oneshot
ExecStart=/bin/bash -c '[ -x /usr/local/bin/autoap ] && /usr/local/bin/autoap reset'

[Install]
WantedBy=multi-user.target

"#;
        write_file("/etc/systemd/system/wpa-autoap-restore.service", restore_service).await?;

        info!("Created systemd service files");
        Ok(())
    }

    async fn setup_local_script(&self) -> Result<()> {
        info!("Creating /usr/local/bin/autoAP-local.sh...");

        backup_file("/usr/local/bin/autoAP-local.sh").await?;

        let local_script = r#"#!/bin/bash
# $1 has either "Client" or "AccessPoint"

logmsg () {
    [ $debug -eq 0 ] && logger --id=$$ "$1"
}

[ -f /usr/local/bin/autoAP.conf ] && source /usr/local/bin/autoAP.conf || debug=0

case "$1" in
    Client)
          logmsg "/usr/local/bin/autoAP-local: Client"
	  ## Add your code here that runs when the Client WiFi is enabled
	  ;;
    AccessPoint)
          logmsg "/usr/local/bin/autoAP-local: Access Point"
	  ## Add your code here that runs when the Access Point is enabled
	  ;;
esac
"#;

        write_file("/usr/local/bin/autoAP-local.sh", local_script).await?;
        make_executable("/usr/local/bin/autoAP-local.sh").await?;

        info!("Created local script");
        Ok(())
    }

    async fn configure_services(&self) -> Result<()> {
        info!("Configuring systemd services...");

        // Reload systemd daemon
        systemctl_command(&["daemon-reload"]).await?;

        // Enable wpa_supplicant@wlan0
        info!("Enabling wpa_supplicant@wlan0...");
        systemctl_command(&["enable", "wpa_supplicant@wlan0"]).await?;

        // Disable vanilla wpa_supplicant
        info!("Disabling (vanilla) wpa_supplicant...");
        systemctl_command(&["disable", "wpa_supplicant"]).await
            .unwrap_or_else(|e| {
                warn!("Failed to disable wpa_supplicant (may not be enabled): {}", e);
            });

        // Enable wpa-autoap@wlan0
        info!("Enabling wpa-autoap@wlan0 service...");
        systemctl_command(&["enable", "wpa-autoap@wlan0"]).await?;

        // Enable wpa-autoap-restore
        info!("Enabling wpa-autoap-restore service...");
        systemctl_command(&["enable", "wpa-autoap-restore"]).await?;

        info!("Service configuration completed");
        Ok(())
    }
}