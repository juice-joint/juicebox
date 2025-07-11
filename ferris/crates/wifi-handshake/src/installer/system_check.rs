use anyhow::{Context, Result};
use dialoguer::Confirm;
use std::fs;
use tracing::{info, warn};

use crate::{installer::InstallerStep, utils::{is_systemd_networkd_active, is_systemd_resolved_active, systemctl_command, write_file}};

pub struct SystemCheckStep;

impl InstallerStep for SystemCheckStep {
    fn execute(&self) -> Result<()> {
        info!("Step 1: Checking system requirements...");
        
        // Check for NetworkManager conflict first
        self.check_network_manager_conflict()?;
        
        // Check systemd-networkd
        self.check_and_enable_service("systemd-networkd", "manage network configurations")?;
        
        // Check systemd-resolved
        self.check_and_enable_service("systemd-resolved", "provide DNS resolution")?;

        info!("✓ System check completed");
        Ok(())
    }
}

impl SystemCheckStep {
    pub fn new() -> Self {
        Self
    }

    fn check_network_manager_conflict(&self) -> Result<()> {
        info!("Checking for NetworkManager conflicts...");
        
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
                self.disable_network_manager()?;
            } else if Confirm::new()
                .with_prompt("Configure NetworkManager to ignore wlan0 instead?")
                .interact()?
            {
                self.configure_network_manager_ignore()?;
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

    fn disable_network_manager(&self) -> Result<()> {
        info!("Stopping and disabling NetworkManager...");
        
        std::process::Command::new("systemctl")
            .args(["stop", "NetworkManager"])
            .output()
            .context("Failed to stop NetworkManager")?;
        
        std::process::Command::new("systemctl")
            .args(["disable", "NetworkManager"])
            .output()
            .context("Failed to disable NetworkManager")?;
        
        std::process::Command::new("systemctl")
            .args(["mask", "NetworkManager"])
            .output()
            .context("Failed to mask NetworkManager")?;
        
        info!("NetworkManager has been disabled");
        Ok(())
    }

    fn configure_network_manager_ignore(&self) -> Result<()> {
        info!("Configuring NetworkManager to ignore wlan0...");
        
        fs::create_dir_all("/etc/NetworkManager/conf.d")
            .context("Failed to create NetworkManager config directory")?;
        
        let config_content = r#"[keyfile]
unmanaged-devices=interface-name:wlan0
"#;
        
        write_file("/etc/NetworkManager/conf.d/99-unmanaged-devices.conf", config_content)?;
        
        std::process::Command::new("systemctl")
            .args(["restart", "NetworkManager"])
            .output()
            .context("Failed to restart NetworkManager")?;
        
        info!("NetworkManager configured to ignore wlan0");
        Ok(())
    }

    fn check_and_enable_service(&self, service_name: &str, description: &str) -> Result<()> {
        info!("Checking {} configuration...", service_name);
        
        if !self.service_exists(service_name)? {
            warn!("{} service does not exist on this system", service_name);
            
            if service_name == "systemd-resolved" {
                if Confirm::new()
                    .with_prompt("systemd-resolved is not installed. Would you like autoAP to install it?")
                    .interact()?
                {
                    self.install_systemd_resolved()?;
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
            "systemd-networkd" => is_systemd_networkd_active()?,
            "systemd-resolved" => is_systemd_resolved_active()?,
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
                systemctl_command(&["enable", service_name])
                    .context(format!("Failed to enable {}", service_name))?;
                systemctl_command(&["start", service_name])
                    .context(format!("Failed to start {}", service_name))?;
                
                let is_now_active = match service_name {
                    "systemd-networkd" => is_systemd_networkd_active()?,
                    "systemd-resolved" => is_systemd_resolved_active()?,
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

    fn service_exists(&self, service_name: &str) -> Result<bool> {
        let output = std::process::Command::new("systemctl")
            .args(["cat", service_name])
            .output()
            .context("Failed to check if service exists")?;
        
        Ok(output.status.success())
    }

    fn install_systemd_resolved(&self) -> Result<()> {
        info!("Installing systemd-resolved...");
        
        if self.command_exists("apt")? {
            self.install_with_apt()?;
        } else if self.command_exists("dnf")? {
            self.install_with_dnf()?;
        } else if self.command_exists("yum")? {
            self.install_with_yum()?;
        } else if self.command_exists("pacman")? {
            self.install_with_pacman()?;
        } else {
            return Err(anyhow::anyhow!(
                "No supported package manager found (apt, dnf, yum, pacman). Please install systemd-resolved manually."
            ));
        }
        
        info!("systemd-resolved installed successfully");
        systemctl_command(&["daemon-reload"])
            .context("Failed to reload systemd after installing systemd-resolved")?;
        
        Ok(())
    }

    fn install_with_apt(&self) -> Result<()> {
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
        Ok(())
    }

    fn install_with_dnf(&self) -> Result<()> {
        info!("Using dnf to install systemd-resolved...");
        let output = std::process::Command::new("dnf")
            .args(["install", "-y", "systemd-resolved"])
            .output()
            .context("Failed to install systemd-resolved with dnf")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
        }
        Ok(())
    }

    fn install_with_yum(&self) -> Result<()> {
        info!("Using yum to install systemd-resolved...");
        let output = std::process::Command::new("yum")
            .args(["install", "-y", "systemd-resolved"])
            .output()
            .context("Failed to install systemd-resolved with yum")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
        }
        Ok(())
    }

    fn install_with_pacman(&self) -> Result<()> {
        info!("Using pacman to install systemd-resolved...");
        let output = std::process::Command::new("pacman")
            .args(["-S", "--noconfirm", "systemd-resolvconf"])
            .output()
            .context("Failed to install systemd-resolved with pacman")?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to install systemd-resolved: {}", stderr));
        }
        Ok(())
    }

    fn command_exists(&self, command: &str) -> Result<bool> {
        let output = std::process::Command::new("which")
            .arg(command)
            .output()
            .context("Failed to check if command exists")?;
        
        Ok(output.status.success())
    }
}

