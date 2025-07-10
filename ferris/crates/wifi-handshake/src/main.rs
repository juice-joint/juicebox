use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};

mod config;
mod installer;
mod runtime;
mod utils;

use installer::Installer;
use runtime::AutoAp;

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
    /// Force installation
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
    /// Show current status
    Status,
    /// Reset autoAP state
    Reset,
    /// Start monitoring mode
    Start {
        /// Interface name (e.g., wlan0)
        interface: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("autoap={}", log_level))
        .with_target(false)
        .with_thread_ids(true)
        .init();

    // Check if autoAP is already installed
    let is_installed = utils::is_autoap_installed().await;
    
    match cli.command {
        Some(Commands::Install) => {
            info!("Starting autoAP installation...");
            let installer = Installer::new();
            installer.install().await?;
        }
        None if cli.force_install || !is_installed => {
            info!("Starting autoAP installation...");
            let installer = Installer::new();
            installer.install().await?;
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
                std::process::exit(1);
            }
            
            let autoap = AutoAp::new().await?;
            autoap.run(args).await?;
        }
        Some(Commands::Status) => {
            if is_installed {
                println!("autoAP is installed and configured");
                // TODO: Add more detailed status
            } else {
                println!("autoAP is not installed");
            }
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
        Some(Commands::Run { .. }) => {
            // autoAP is not installed but user tried to run
            error!("autoAP is not installed. Run without arguments to install, or use 'autoap install'");
            std::process::exit(1);
        }
        None => {
            // No command and no installation - this case should be handled above
            unreachable!("This case should be handled by the installation logic above");
        }
    }

    Ok(())
}