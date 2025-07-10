use binary_sidecar::{
    deps::{ffmpeg::FfmpegFetcher, ytdlp::YtdlpFetcher, ReleaseFetcher},
    download_and_extract_binary_path,
    utils::{architecture::Architecture, platform::Platform},
};
use crate::server::globals::{init_config_dir, set_binary_path};
use crate::ui_state_controller::UIStateController;
use std::{path::PathBuf, sync::atomic::{AtomicBool, Ordering}};
use tracing::{error, info};

const DOWNLOAD_FFMPEG: bool = true;
const DOWNLOAD_YTDLP: bool = true;

// Global flag to track if binaries have been initialized
static BINARIES_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Manages binary initialization for the application
pub struct BinaryInitializer;

impl BinaryInitializer {
    /// Check if binaries have already been initialized
    pub fn are_binaries_initialized() -> bool {
        BINARIES_INITIALIZED.load(Ordering::Relaxed)
    }

    /// Initialize all required binaries (ffmpeg, yt-dlp)
    pub async fn initialize(config_dir: PathBuf, ui_controller: UIStateController) {
        // Check if already initialized
        if Self::are_binaries_initialized() {
            info!("Binaries already initialized, skipping download");
            ui_controller.handle_initialization_complete();
            return;
        }

        tokio::spawn(async move {
            info!("Starting binary initialization");

            let platform = Platform::detect();
            let architecture = Architecture::detect();

            // Initialize binaries in parallel
            let mut tasks = Vec::new();

            // Add ffmpeg download task
            if DOWNLOAD_FFMPEG {
                let platform_clone = platform.clone();
                let architecture_clone = architecture.clone();
                let config_dir_clone = config_dir.clone();
                tasks.push(tokio::spawn(async move {
                    Self::download_ffmpeg(&platform_clone, &architecture_clone, &config_dir_clone).await
                }));
            }

            // Add yt-dlp download task
            if DOWNLOAD_YTDLP {
                let platform_clone = platform.clone();
                let architecture_clone = architecture.clone();
                let config_dir_clone = config_dir.clone();
                tasks.push(tokio::spawn(async move {
                    Self::download_ytdlp(&platform_clone, &architecture_clone, &config_dir_clone).await
                }));
            }

            // Wait for all tasks to complete
            for task in tasks {
                match task.await {
                    Ok(result) => {
                        if let Err(e) = result {
                            error!("Failed to initialize binary: {}", e);
                            return;
                        }
                    }
                    Err(e) => {
                        error!("Task failed to execute: {}", e);
                        return;
                    }
                }
            }

            // Mark binaries as initialized
            BINARIES_INITIALIZED.store(true, Ordering::Relaxed);
            info!("All binaries initialized successfully");

            // Signal completion to UI
            ui_controller.handle_initialization_complete();
            
            // Initialize config directory
            init_config_dir(config_dir);
        });
    }

    /// Download and configure ffmpeg binary
    async fn download_ffmpeg(
        platform: &Platform,
        architecture: &Architecture,
        config_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Downloading ffmpeg binary");
        
        let ffmpeg_fetcher = FfmpegFetcher::new("ffmpeg".to_string());
        let release = ffmpeg_fetcher
            .get_release(platform, architecture)
            .await?;
            
        let ffmpeg_path = download_and_extract_binary_path(release, config_dir).await?;
        
        info!("ffmpeg binary downloaded and extracted at {}", ffmpeg_path.display());
        set_binary_path("ffmpeg", ffmpeg_path);
        
        Ok(())
    }

    /// Download and configure yt-dlp binary
    async fn download_ytdlp(
        platform: &Platform,
        architecture: &Architecture,
        config_dir: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Downloading yt-dlp binary");
        
        let ytdlp_fetcher = YtdlpFetcher::new();
        let release = ytdlp_fetcher
            .get_release(platform, architecture)
            .await?;
            
        let ytdlp_path = download_and_extract_binary_path(release, config_dir).await?;
        
        info!("yt-dlp binary downloaded and extracted at: {}", ytdlp_path.display());
        set_binary_path("yt-dlp", ytdlp_path);
        
        Ok(())
    }

}