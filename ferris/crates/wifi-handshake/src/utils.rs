use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{debug, warn};

/// Check if autoAP is already installed by looking for key files
pub fn is_autoap_installed() -> bool {
    let required_files = [
        "/usr/local/bin/autoAP.conf",
        "/etc/systemd/system/wpa-autoap@wlan0.service",
        "/etc/systemd/system/wpa-autoap-restore.service",
        "/etc/wpa_supplicant/wpa_supplicant-wlan0.conf",
        "/etc/systemd/network/12-wlan0AP.network",
    ];

    // Check required files
    for file in &required_files {
        if !Path::new(file).exists() {
            debug!("Missing required file for autoAP: {}", file);
            return false;
        }
    }

    // For the client network file, check both locations since it moves between them
    let client_network_file = "/etc/systemd/network/11-wlan0.network";
    let client_network_backup = "/etc/systemd/network/11-wlan0.network~";
    
    if !Path::new(client_network_file).exists() && !Path::new(client_network_backup).exists() {
        debug!("Missing client network file (checked both {} and {})", client_network_file, client_network_backup);
        return false;
    }

    debug!("All required autoAP files found - installation detected");
    true
}

/// Check if systemd-networkd is active
pub fn is_systemd_networkd_active() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["is-active", "systemd-networkd"])
        .output()
        .context("Failed to check systemd-networkd status")?;

    Ok(output.status.success() && 
       String::from_utf8_lossy(&output.stdout).trim() == "active")
}

/// Check if systemd-resolved is active
pub fn is_systemd_resolved_active() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["is-active", "systemd-resolved"])
        .output()
        .context("Failed to check systemd-resolved status")?;

    Ok(output.status.success() && 
       String::from_utf8_lossy(&output.stdout).trim() == "active")
}

/// Run a systemctl command
pub fn systemctl_command(args: &[&str]) -> Result<()> {
    let output = Command::new("systemctl")
        .args(args)
        .output()
        .context(format!("Failed to run systemctl {:?}", args))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "systemctl command failed: {:?}\nError: {}", 
            args, 
            stderr
        ));
    }

    Ok(())
}

/// Create a backup of a file if it exists
pub fn backup_file(original: &str) -> Result<()> {
    if !Path::new(original).exists() {
        return Ok(());
    }

    let backup_path = format!("{}.bak", original);
    
    // If backup already exists, create a .bak.old version
    if Path::new(&backup_path).exists() {
        let old_backup = format!("{}.old", backup_path);
        if Path::new(&old_backup).exists() {
            fs::remove_file(&old_backup)
                .context("Failed to remove old backup")?;
        }
        fs::rename(&backup_path, &old_backup)
            .context("Failed to move existing backup")?;
    }

    fs::copy(original, &backup_path)
        .context("Failed to create backup")?;

    debug!("Created backup: {} -> {}", original, backup_path);
    Ok(())
}

/// Write content to a file, creating parent directories if needed
pub fn write_file(path: &str, content: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)
            .context("Failed to create parent directories")?;
    }

    fs::write(path, content)
        .context(format!("Failed to write file: {}", path))?;

    debug!("Wrote file: {}", path);
    Ok(())
}

/// Run wpa_cli command and return output
pub fn wpa_cli_command(interface: &str, args: &[&str]) -> Result<String> {
    let mut cmd_args = vec!["-i", interface];
    cmd_args.extend(args);

    let output = Command::new("wpa_cli")
        .args(&cmd_args)
        .output()
        .context("Failed to run wpa_cli command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!(
            "wpa_cli command failed: {:?}\nError: {}", 
            cmd_args, 
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Check if any stations are connected to the AP
pub fn has_connected_stations(interface: &str) -> Result<bool> {
    match wpa_cli_command(interface, &["all_sta"]) {
        Ok(output) => Ok(!output.trim().is_empty()),
        Err(e) => {
            warn!("Failed to check connected stations: {}", e);
            Ok(false)
        }
    }
}