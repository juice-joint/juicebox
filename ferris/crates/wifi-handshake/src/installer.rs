use anyhow::{Context, Result};
use dialoguer::{Confirm, Input};
use tracing::{info, warn};

use crate::config::{ApConfig, AutoApConfig, InstallConfig, WifiConfig};
use crate::utils::{backup_file, is_systemd_networkd_active, is_systemd_resolved_active, systemctl_command, write_file};

pub struct Installer;

impl Installer {
    pub fn new() -> Self {
        Self
    }

    pub async fn install(&self) -> Result<()> {
        info!("Starting autoAP installation...");

        // Check required systemd services
        self.check_required_systemd_services().await?;

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

    async fn check_required_systemd_services(&self) -> Result<()> {
        info!("Checking required systemd services...");
        
        // Check for NetworkManager conflict first
        self.check_network_manager_conflict().await?;
        
        // Check systemd-networkd
        self.check_and_enable_service("systemd-networkd", "manage network configurations").await?;
        
        // Check systemd-resolved
        self.check_and_enable_service("systemd-resolved", "provide DNS resolution").await?;

        Ok(())
    }

    async fn check_network_manager_conflict(&self) -> Result<()> {
        info!("Checking for NetworkManager conflicts...");
        
        // Check if NetworkManager is active
        let output = std::process::Command::new("systemctl")
            .args(["is-active", "NetworkManager"])
            .output()
            .context("Failed to check NetworkManager status")?;
        
        if output.status.success() {
            warn!("NetworkManager is active and will conflict with autoAP");
            warn!("NetworkManager and wpa_supplicant@wlan0 cannot both manage the same interface");
            
            if Confirm::new()
                .with_prompt("Would you like autoAP to disable NetworkManager? (Recommended)")
                .interact()?
            {
                info!("Stopping and disabling NetworkManager...");
                
                // Stop NetworkManager
                std::process::Command::new("systemctl")
                    .args(["stop", "NetworkManager"])
                    .output()
                    .context("Failed to stop NetworkManager")?;
                
                // Disable NetworkManager
                std::process::Command::new("systemctl")
                    .args(["disable", "NetworkManager"])
                    .output()
                    .context("Failed to disable NetworkManager")?;
                
                // Mask NetworkManager to prevent accidental re-enabling
                std::process::Command::new("systemctl")
                    .args(["mask", "NetworkManager"])
                    .output()
                    .context("Failed to mask NetworkManager")?;
                
                info!("NetworkManager has been disabled");
            } else if Confirm::new()
                .with_prompt("Configure NetworkManager to ignore wlan0 instead?")
                .interact()?
            {
                self.configure_network_manager_ignore().await?;
            } else {
                return Err(anyhow::anyhow!(
                    "Installation cancelled: NetworkManager conflicts with autoAP must be resolved"
                ));
            }
        } else {
            info!("NetworkManager is not active ✓");
        }
        
        Ok(())
    }

    async fn configure_network_manager_ignore(&self) -> Result<()> {
        info!("Configuring NetworkManager to ignore wlan0...");
        
        // Create NetworkManager config directory if it doesn't exist
        tokio::fs::create_dir_all("/etc/NetworkManager/conf.d").await
            .context("Failed to create NetworkManager config directory")?;
        
        let config_content = r#"[keyfile]
unmanaged-devices=interface-name:wlan0
"#;
        
        write_file("/etc/NetworkManager/conf.d/99-unmanaged-devices.conf", config_content).await?;
        
        // Restart NetworkManager to apply the configuration
        std::process::Command::new("systemctl")
            .args(["restart", "NetworkManager"])
            .output()
            .context("Failed to restart NetworkManager")?;
        
        info!("NetworkManager configured to ignore wlan0");
        Ok(())
    }

    async fn check_and_enable_service(&self, service_name: &str, description: &str) -> Result<()> {
        info!("Checking {} configuration...", service_name);
        
        // First check if the service exists
        if !self.service_exists(service_name).await? {
            warn!("{} service does not exist on this system", service_name);
            
            if service_name == "systemd-resolved" {
                if Confirm::new()
                    .with_prompt("systemd-resolved is not installed. Would you like autoAP to install it?")
                    .interact()?
                {
                    self.install_systemd_resolved().await?;
                } else {
                    return Err(anyhow::anyhow!(
                        "Installation cancelled: systemd-resolved is required for autoAP to function properly"
                    ));
                }
            } else {
                return Err(anyhow::anyhow!(
                    "{} service does not exist and cannot be automatically installed", 
                    service_name
                ));
            }
        }
        
        let is_active = match service_name {
            "systemd-networkd" => is_systemd_networkd_active().await?,
            "systemd-resolved" => is_systemd_resolved_active().await?,
            _ => return Err(anyhow::anyhow!("Unknown service: {}", service_name)),
        };

        if !is_active {
            warn!("{} is not active", service_name);
            warn!("autoAP requires {} to {}", service_name, description);
            
            if Confirm::new()
                .with_prompt(format!("Would you like autoAP to enable and start {}?", service_name))
                .interact()?
            {
                info!("Enabling and starting {}...", service_name);
                systemctl_command(&["enable", service_name]).await
                    .context(format!("Failed to enable {}", service_name))?;
                systemctl_command(&["start", service_name]).await
                    .context(format!("Failed to start {}", service_name))?;
                
                // Verify it's now running
                let is_now_active = match service_name {
                    "systemd-networkd" => is_systemd_networkd_active().await?,
                    "systemd-resolved" => is_systemd_resolved_active().await?,
                    _ => return Err(anyhow::anyhow!("Unknown service: {}", service_name)),
                };

                if !is_now_active {
                    return Err(anyhow::anyhow!("Failed to start {}", service_name));
                }
                info!("{} is now active", service_name);
            } else {
                return Err(anyhow::anyhow!(
                    "Installation cancelled: {} is required for autoAP to function properly", 
                    service_name
                ));
            }
        } else {
            info!("{} is active ✓", service_name);
        }

        Ok(())
    }

    async fn service_exists(&self, service_name: &str) -> Result<bool> {
        let output = std::process::Command::new("systemctl")
            .args(["cat", service_name])
            .output()
            .context("Failed to check if service exists")?;
        
        Ok(output.status.success())
    }

    async fn install_systemd_resolved(&self) -> Result<()> {
        info!("Installing systemd-resolved...");
        
        // Detect package manager and install systemd-resolved
        if self.command_exists("apt").await? {
            info!("Using apt to install systemd-resolved...");
            let output = std::process::Command::new("apt")
                .args(["update"])
                .output()
                .context("Failed to update package list")?;
            
            if !output.status.success() {
                warn!("apt update failed, continuing with installation attempt...");
            }
            
            let output = std::process::Command::new("apt")
                .args(["install", "-y", "systemd-resolved"])
                .output()
                .context("Failed to install systemd-resolved with apt")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
            }
        } else if self.command_exists("dnf").await? {
            info!("Using dnf to install systemd-resolved...");
            let output = std::process::Command::new("dnf")
                .args(["install", "-y", "systemd-resolved"])
                .output()
                .context("Failed to install systemd-resolved with dnf")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
            }
        } else if self.command_exists("yum").await? {
            info!("Using yum to install systemd-resolved...");
            let output = std::process::Command::new("yum")
                .args(["install", "-y", "systemd-resolved"])
                .output()
                .context("Failed to install systemd-resolved with yum")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
            }
        } else if self.command_exists("pacman").await? {
            info!("Using pacman to install systemd-resolved...");
            let output = std::process::Command::new("pacman")
                .args(["-S", "--noconfirm", "systemd-resolvconf"])
                .output()
                .context("Failed to install systemd-resolved with pacman")?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
            }
        } else {
            return Err(anyhow::anyhow!(
                "No supported package manager found (apt, dnf, yum, pacman). Please install systemd-resolved manually."
            ));
        }
        
        info!("systemd-resolved installed successfully");
        
        // Reload systemd to pick up the new service
        systemctl_command(&["daemon-reload"]).await
            .context("Failed to reload systemd after installing systemd-resolved")?;
        
        Ok(())
    }

    async fn command_exists(&self, command: &str) -> Result<bool> {
        let output = std::process::Command::new("which")
            .arg(command)
            .output()
            .context("Failed to check if command exists")?;
        
        Ok(output.status.success())
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

        // Save autoAP configuration
        config.autoap.save().await?;

        // Configure services
        self.configure_services().await?;

        // Verify installation (both services should now be active)
        self.verify_installation().await?;

        Ok(())
    }

    async fn verify_installation(&self) -> Result<()> {
        info!("Verifying installation...");

        // Check that systemd-networkd is still running
        if !is_systemd_networkd_active().await? {
            return Err(anyhow::anyhow!("systemd-networkd is not active after installation"));
        }
        info!("systemd-networkd is active ✓");

        // Check that systemd-resolved is running (now mandatory)
        if !is_systemd_resolved_active().await? {
            return Err(anyhow::anyhow!("systemd-resolved is not active after installation"));
        }
        info!("systemd-resolved is active ✓");

        // Check that required services are enabled
        let services_to_check = [
            "wpa_supplicant@wlan0",
            "wpa-autoap@wlan0", 
            "wpa-autoap-restore",
            "systemd-networkd",
            "systemd-resolved"  // Added to verification list
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

        // Test that our binary is accessible and working
        let output = std::process::Command::new("/usr/local/bin/autoap")
            .args(["--help"])
            .output()
            .context("Failed to test autoap binary")?;
            
        if !output.status.success() {
            return Err(anyhow::anyhow!("autoap binary is not working properly"));
        }

        info!("Installation verification completed ✓");
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
        info!("Creating network files in /etc/systemd/network...");

        // Backup existing files
        backup_file("/etc/systemd/network/10-eth0.network").await?;
        backup_file("/etc/systemd/network/11-wlan0.network").await?;
        backup_file("/etc/systemd/network/12-wlan0AP.network").await?;

        // Remove any existing backup file that autoAP creates
        if std::path::Path::new("/etc/systemd/network/11-wlan0.network~").exists() {
            tokio::fs::remove_file("/etc/systemd/network/11-wlan0.network~").await?;
        }

        // Create ethernet network config
        let ethernet_config = r#"[Match]
Name=eth0

[Network]
DHCP=ipv4

[DHCP]
RouteMetric=10
UseDomains=yes
UseDNS=yes

"#;
        write_file("/etc/systemd/network/10-eth0.network", ethernet_config).await?;

        // Create WiFi client network config
        let client_config = r#"[Match]
Name=wlan0

[Network]
DHCP=ipv4

[DHCP]
RouteMetric=20
UseDomains=yes
UseDNS=yes

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

        info!("Created systemd network configuration files (eth0, wlan0 client, wlan0 AP)");
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