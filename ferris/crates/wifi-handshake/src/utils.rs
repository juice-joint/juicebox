use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use tokio::fs;
use tracing::{debug, warn};

/// Check if autoAP is already installed by looking for key files
pub async fn is_autoap_installed() -> bool {
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
pub async fn is_systemd_networkd_active() -> Result<bool> {
    let output = Command::new("systemctl")
        .args(["is-active", "systemd-networkd"])
        .output()
        .context("Failed to check systemd-networkd status")?;

    Ok(output.status.success() && 
       String::from_utf8_lossy(&output.stdout).trim() == "active")
}

/// Run a systemctl command
pub async fn systemctl_command(args: &[&str]) -> Result<()> {
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
pub async fn backup_file(original: &str) -> Result<()> {
    if !Path::new(original).exists() {
        return Ok(());
    }

    let backup_path = format!("{}.bak", original);
    
    // If backup already exists, create a .bak.old version
    if Path::new(&backup_path).exists() {
        let old_backup = format!("{}.old", backup_path);
        if Path::new(&old_backup).exists() {
            fs::remove_file(&old_backup).await
                .context("Failed to remove old backup")?;
        }
        fs::rename(&backup_path, &old_backup).await
            .context("Failed to move existing backup")?;
    }

    fs::copy(original, &backup_path).await
        .context("Failed to create backup")?;

    debug!("Created backup: {} -> {}", original, backup_path);
    Ok(())
}

/// Write content to a file, creating parent directories if needed
pub async fn write_file(path: &str, content: &str) -> Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).await
            .context("Failed to create parent directories")?;
    }

    fs::write(path, content).await
        .context(format!("Failed to write file: {}", path))?;

    debug!("Wrote file: {}", path);
    Ok(())
}

/// Check if a file is executable
pub async fn is_executable(path: &str) -> bool {
    match fs::metadata(path).await {
        Ok(metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(not(unix))]
            {
                true // Assume executable on non-Unix systems
            }
        }
        Err(_) => false,
    }
}

/// Make a file executable
pub async fn make_executable(path: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path).await
            .context("Failed to get file metadata")?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(permissions.mode() | 0o755);
        fs::set_permissions(path, permissions).await
            .context("Failed to set file permissions")?;
    }
    
    debug!("Made file executable: {}", path);
    Ok(())
}

/// Run wpa_cli command and return output
pub async fn wpa_cli_command(interface: &str, args: &[&str]) -> Result<String> {
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
pub async fn has_connected_stations(interface: &str) -> Result<bool> {
    match wpa_cli_command(interface, &["all_sta"]).await {
        Ok(output) => Ok(!output.trim().is_empty()),
        Err(e) => {
            warn!("Failed to check connected stations: {}", e);
            Ok(false)
        }
    }
}

/// Check if interface is in station mode
pub async fn is_station_mode(interface: &str) -> Result<bool> {
    match wpa_cli_command(interface, &["status"]).await {
        Ok(output) => Ok(output.contains("mode=station")),
        Err(e) => {
            warn!("Failed to check station mode: {}", e);
            Ok(false)
        }
    }
}