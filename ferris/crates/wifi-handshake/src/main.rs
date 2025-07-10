use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::{error, info};

mod config;
mod installer;
mod runtime;
mod utils;

use config::AutoApConfig;
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
            tracing_subscriber::fmt()
                .with_env_filter("autoap=info")
                .with_target(false)
                .with_thread_ids(true)
                .init();
            
            // Check if autoAP is installed
            if !utils::is_autoap_installed().await {
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