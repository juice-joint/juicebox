use anyhow::Result;
use tracing::info;

use crate::{installer::InstallerStep, utils::{backup_file, write_file}};

pub struct SystemdServicesStep;

impl InstallerStep for SystemdServicesStep {
    fn execute(&self) -> Result<()> {
        info!("Step 6: Creating systemd service files...");

        // Backup existing service files
        backup_file("/etc/systemd/system/wpa-autoap@wlan0.service")?;
        backup_file("/etc/systemd/system/wpa-autoap-restore.service")?;

        self.create_autoap_service()?;
        self.create_restore_service()?;

        info!("✓ systemd service files created");
        Ok(())
    }
}

impl SystemdServicesStep {
    pub fn new() -> Self {
        Self
    }

    fn create_autoap_service(&self) -> Result<()> {
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
        write_file("/etc/systemd/system/wpa-autoap@wlan0.service", autoap_service)?;
        info!("Created wpa-autoap@wlan0.service");
        Ok(())
    }

    fn create_restore_service(&self) -> Result<()> {
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
        write_file("/etc/systemd/system/wpa-autoap-restore.service", restore_service)?;
        info!("Created wpa-autoap-restore.service");
        Ok(())
    }
}