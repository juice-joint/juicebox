use binary_sidecar::{
    deps::{ffmpeg::FfmpegFetcher, ytdlp::YtdlpFetcher, ReleaseFetcher},
    download_and_extract_binary_path,
    utils::{architecture::Architecture, platform::Platform},
};
use desktop::window::{AppEvent, WindowEventHandle};
use server::globals::{init_config_dir, set_binary_path};
use std::{net::SocketAddr, path::PathBuf, thread, time::Duration};
use tao::event_loop::EventLoopBuilder;
use tokio::{sync::oneshot, task::JoinHandle};

mod desktop;
mod server;

const DOWNLOAD_FFMPEG: bool = true;
const DOWNLOAD_YTDLP: bool = true;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    let config_dir = PathBuf::from("./config");

    // Start the server
    let server_handle = start_server(addr).await;

    // Create window event loop and handle
    let (event_loop, window_event_handle) = create_window_components();

    // Initialize binaries
    initialize_binaries(config_dir.clone(), window_event_handle.clone()).await;

    // Run the desktop window
    match run_desktop_window(event_loop).await {
        Ok(_) => tracing::info!("Desktop app closed successfully"),
        Err(e) => tracing::error!("Desktop app error: {}", e),
    }

    // Cleanup
    server_handle.abort();
    tracing::info!("Application shutting down");
}

async fn start_server(addr: SocketAddr) -> JoinHandle<()> {
    tracing::info!("Starting server on {}", addr);

    let (tx, rx) = oneshot::channel();
    let server_handle = tokio::spawn(async move {
        server::run_server(addr, tx).await;
    });

    rx.await.expect("Failed to receive server ready signal");
    tracing::info!("Server is ready");

    server_handle
}

async fn initialize_binaries(config_dir: PathBuf, window_event_handle: WindowEventHandle) {
    tokio::spawn(async move {
        tracing::info!("Starting binary initialization");

        let platform = Platform::detect();
        let architecture = Architecture::detect();

        if DOWNLOAD_FFMPEG {
            let ffmpeg_fetcher = FfmpegFetcher::new("ffmpeg".to_string());
            let ffmpeg_path = match download_and_extract_binary_path(
                ffmpeg_fetcher
                    .get_release(&platform, &architecture)
                    .await
                    .unwrap(),
                &config_dir,
            )
            .await
            {
                Ok(path) => {
                    tracing::info!(
                        "ffmpeg binary downloaded and extracted at {}",
                        path.display()
                    );
                    path
                }
                Err(e) => {
                    tracing::error!("Failed to initialize ffmpeg: {}", e);
                    return;
                }
            };
            set_binary_path("ffmpeg", ffmpeg_path);
        }

        if DOWNLOAD_YTDLP {
            let ytdlp_fetcher = YtdlpFetcher::new();
            let ytdlp_path = match download_and_extract_binary_path(
                ytdlp_fetcher
                    .get_release(&platform, &architecture)
                    .await
                    .unwrap(),
                &config_dir,
            )
            .await
            {
                Ok(path) => {
                    tracing::info!(
                        "yt-dlp binary downloaded and extracted at: {}",
                        path.display()
                    );
                    path
                }
                Err(e) => {
                    tracing::error!("Failed to initialize yt-dlp: {}", e);
                    return;
                }
            };
            set_binary_path("yt-dlp", ytdlp_path);
        }

        tracing::info!("Binary initialization complete, redirecting to /goldie");
        window_event_handle.load_url("http://localhost:8000/goldie?view=home".to_string());
        window_event_handle.hide_window();
        thread::sleep(Duration::from_millis(100));
        window_event_handle.show_window();
        init_config_dir(config_dir);
    });
}

fn create_window_components() -> (tao::event_loop::EventLoop<AppEvent>, WindowEventHandle) {
    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let event_loop_proxy = event_loop.create_proxy();
    let window_event_handle = WindowEventHandle::new(event_loop_proxy);

    (event_loop, window_event_handle)
}

async fn run_desktop_window(
    event_loop: tao::event_loop::EventLoop<AppEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    desktop::window::create_desktop_webview("http://localhost:8000/goldie?view=loading", event_loop)
        .map(|_| ())
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}
