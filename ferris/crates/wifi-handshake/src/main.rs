use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};

mod config;
mod installer;
mod runtime;
mod utils;
mod web_server;
mod wpa_manager;

use config::AutoApConfig;
use installer::Installer;

use crate::runtime::AutoAp;

#[derive(Parser)]
#[command(name = "autoap")]
#[command(about = "Automatic Access Point - Install or run WiFi AP fallback")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// Force installation even if already installed
    #[arg(long)]
    force_install: bool,
    
    /// Verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Install autoAP (AP-only mode, WiFi client management added later)
    Install,
    /// Run the autoAP service (auto-detected if already installed)
    Run {
        /// Interface name (e.g., wlan0)
        interface: Option<String>,
        /// WiFi state change
        state: Option<String>,
        /// MAC address for station events
        mac: Option<String>,
    },
    /// Show current status and configuration
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,
    },
    /// Reset autoAP state
    Reset,
    /// Start monitoring mode
    Start {
        /// Interface name (e.g., wlan0)
        interface: String,
    },
    /// Start web server for WiFi configuration
    WebServer {
        /// Port to run web server on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Uninstall autoAP
    Uninstall {
        /// Force uninstall without confirmation
        #[arg(long)]
        force: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let raw_args: Vec<String> = std::env::args().collect();
    
    // Handle wpa_cli calls BEFORE clap parsing
    // wpa_cli calls us like: autoap wlan0 AP-ENABLED [mac]
    // or: autoap reset, autoap start wlan0
    if raw_args.len() >= 2 {
        let second_arg = &raw_args[1];
        
        // Check if this looks like a wpa_cli call
        if second_arg == "reset" || 
           second_arg == "start" || 
           (second_arg.starts_with("wlan") && raw_args.len() >= 3) {
            
            // Initialize basic tracing for wpa_cli calls
            init_tracing(false);
            
            // Check if autoAP is installed
            if !utils::is_autoap_installed() {
                debug_missing_files().await;
                error!("autoAP is not installed but being called by wpa_cli");
                std::process::exit(1);
            }
            
            info!("Called by wpa_cli with args: {:?}", &raw_args[1..]);
            let autoap = AutoAp::new().await?;
            autoap.run(raw_args).await?;
            return Ok(());
        }
    }
    
    // If we get here, it's a normal CLI call - parse with clap
    let cli = Cli::parse();
    
    // Initialize tracing with user's verbosity preference
    init_tracing(cli.verbose);

    // Check if autoAP is already installed
    let is_installed = utils::is_autoap_installed();

    match cli.command {
        Some(Commands::Install) => {
            if is_installed && !cli.force_install {
                error!("autoAP is already installed. Use --force-install to reinstall.");
                std::process::exit(1);
            }
            
            info!("Starting autoAP installation (AP-only mode)...");
            let installer = Installer::new();

            match installer.install().await {
                Ok(()) => {
                    info!("🎉 autoAP installation completed successfully!");
                    info!("📋 Next steps:");
                    info!("   • Reboot the system: sudo reboot");
                    info!("   • Your Access Point will be available after reboot");
                    info!("   • Use 'autoap status' to check configuration");
                    info!("   • WiFi client networks can be added later");
                }
                Err(e) => {
                    error!("Installation failed: {}", e);
                    info!("💡 Tips:");
                    info!("   • Run with --verbose for more details");
                    info!("   • Check system requirements");
                    info!("   • Ensure you have sudo/root privileges");
                    std::process::exit(1);
                }
            }
        }
        None if cli.force_install || !is_installed => {
            info!("Starting autoAP installation (AP-only mode)...");
            let installer = Installer::new();
            
            match installer.install().await {
                Ok(()) => {
                    info!("🎉 autoAP installation completed successfully!");
                    info!("Please reboot the system for changes to take effect");
                }
                Err(e) => {
                    error!("Installation failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Run { interface, state, mac }) if is_installed => {
            // Build args vector for compatibility with original script interface
            let mut args = vec!["autoap".to_string()];
            
            if let Some(iface) = interface {
                args.push(iface);
                if let Some(s) = state {
                    args.push(s);
                    if let Some(m) = mac {
                        args.push(m);
                    }
                }
            }
            
            let autoap = AutoAp::new().await?;
            autoap.run(args).await?;
        }
        None if is_installed => {
            // No arguments provided but autoAP is installed
            // This means we're being called by wpa_cli with arguments
            let args = std::env::args().collect::<Vec<_>>();
            
            if args.len() == 1 {
                error!("autoAP is installed but no operation specified");
                info!("Available commands:");
                info!("  autoap status    - Show current status");
                info!("  autoap reset     - Reset autoAP state");
                info!("  autoap start <interface> - Start monitoring");
                std::process::exit(1);
            }
            
            let autoap = AutoAp::new().await?;
            autoap.run(args).await?;
        }
        Some(Commands::Status { detailed }) => {
            show_status(is_installed, detailed).await?;
        }
        Some(Commands::Reset) => {
            if is_installed {
                let autoap = AutoAp::new().await?;
                autoap.reset().await?;
                info!("autoAP state reset");
            } else {
                error!("autoAP is not installed");
                std::process::exit(1);
            }
        }
        Some(Commands::Start { interface }) => {
            if is_installed {
                let autoap = AutoAp::new().await?;
                autoap.start(&interface).await?;
            } else {
                error!("autoAP is not installed");
                std::process::exit(1);
            }
        }
        Some(Commands::WebServer { port }) => {
            info!("Starting WiFi configuration web server on port {}", port);
            let server = web_server::WebServer::new();
            if let Err(e) = server.start(port).await {
                error!("Failed to start web server: {}", e);
                std::process::exit(1);
            }
        }
        Some(Commands::Uninstall { force }) => {
            if !is_installed {
                error!("autoAP is not installed");
                std::process::exit(1);
            }
            
            // TODO: Implement uninstaller using reverse of installation steps
            info!("Uninstall functionality coming soon...");
            if !force {
                info!("For now, manually remove:");
                info!("  • /usr/local/bin/autoap");
                info!("  • /usr/local/bin/autoAP.conf");
                info!("  • /etc/systemd/system/wpa-autoap*.service");
                info!("  • Restore backed up network configs");
            }
        }
        Some(Commands::Run { .. }) => {
            // autoAP is not installed but user tried to run
            error!("autoAP is not installed. Run 'autoap install' to install");
            std::process::exit(1);
        }
        None => {
            // No command and no installation - this case should be handled above
            unreachable!("This case should be handled by the installation logic above");
        }
    }

    Ok(())
}

fn init_tracing(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("autoap={}", log_level))
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .init();
}

async fn debug_missing_files() {
    eprintln!("autoAP installation check failed. Checking files...");
    let required_files = [
        "/usr/local/bin/autoAP.conf",
        "/etc/systemd/system/wpa-autoap@wlan0.service", 
        "/etc/systemd/system/wpa-autoap-restore.service",
        "/etc/wpa_supplicant/wpa_supplicant-wlan0.conf",
        "/etc/systemd/network/12-wlan0AP.network",
    ];
    
    for file in &required_files {
        if std::path::Path::new(file).exists() {
            eprintln!("✓ {}", file);
        } else {
            eprintln!("✗ {} (MISSING)", file);
        }
    }
    
    // Check client network file (can be in either location)
    let client_file = "/etc/systemd/network/11-wlan0.network";
    let client_backup = "/etc/systemd/network/11-wlan0.network~";
    if std::path::Path::new(client_file).exists() {
        eprintln!("✓ {} (client mode)", client_file);
    } else if std::path::Path::new(client_backup).exists() {
        eprintln!("✓ {} (AP mode - client config backed up)", client_backup);
    } else {
        eprintln!("✗ Client network config missing (checked both {} and {})", client_file, client_backup);
    }
}

async fn show_status(is_installed: bool, detailed: bool) -> Result<()> {
    if !is_installed {
        println!("❌ autoAP is not installed");
        println!("Run 'autoap install' to install");
        return Ok(());
    }

    println!("✅ autoAP is installed and configured");
    
    if detailed {
        println!("\n📋 Configuration Details:");
        
        // Show AP configuration if available
        if let Ok(_config) = AutoApConfig::load() {
            println!("   • Configuration file: /usr/local/bin/autoAP.conf");
            // Add more config details here
        }
        
        // Check service status
        println!("\n🔧 Service Status:");
        let services = [
            "systemd-networkd",
            "systemd-resolved", 
            "wpa_supplicant@wlan0",
            "wpa-autoap@wlan0",
            "wpa-autoap-restore"
        ];
        
        for service in &services {
            let output = std::process::Command::new("systemctl")
                .args(["is-active", service])
                .output();
                
            match output {
                Ok(result) if result.status.success() => {
                    println!("   ✅ {} (active)", service);
                }
                Ok(_) => {
                    println!("   ❌ {} (inactive)", service);
                }
                Err(_) => {
                    println!("   ❓ {} (unknown)", service);
                }
            }
        }
        
        // Show network configuration
        println!("\n🌐 Network Configuration:");
        if std::path::Path::new("/etc/wpa_supplicant/wpa_supplicant-wlan0.conf").exists() {
            println!("   ✅ wpa_supplicant configuration");
        }
        if std::path::Path::new("/etc/systemd/network/12-wlan0AP.network").exists() {
            println!("   ✅ Access Point network configuration");
        }
        if std::path::Path::new("/etc/systemd/network/11-wlan0.network").exists() {
            println!("   ✅ WiFi client network configuration");
        }
    } else {
        println!("Use 'autoap status --detailed' for more information");
    }

    Ok(())
}