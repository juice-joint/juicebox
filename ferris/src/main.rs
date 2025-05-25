use binary_sidecar::{
    deps::{ffmpeg::FfmpegFetcher, ytdlp::YtdlpFetcher, ReleaseFetcher}, download_and_extract_binary, download_and_extract_binary_path, utils::{architecture::Architecture, platform::Platform}
};
use desktop::{webview, window::{AppEvent, WindowEventHandle}};
use server::globals::{self, init_config_dir, set_binary_path};
use tao::{event::{Event, WindowEvent}, event_loop::{ControlFlow, EventLoopBuilder}};
use std::{net::SocketAddr, path::PathBuf, sync::{atomic::Ordering, Arc}, thread, time::Duration};
use tokio::sync::oneshot;

mod desktop;
mod server;

const DOWNLOAD_FFMPEG: bool = true;
const DOWNLOAD_YTDLP: bool = true;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let platform = Platform::detect();
    let architecture = Architecture::detect();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Starting server on {}", addr);

    let (tx, rx) = oneshot::channel();

    let server_handle = tokio::spawn(async move {
        server::run_server(addr, tx).await;
    });

    rx.await.expect("Failed to receive server ready signal");
    tracing::info!("Server is ready");

    let event_loop = EventLoopBuilder::<AppEvent>::with_user_event().build();
    let event_loop_proxy = event_loop.create_proxy();
    let window_event_handle = WindowEventHandle::new(event_loop_proxy);
    let window_event_handle_clone = window_event_handle.clone();

    let config_dir = PathBuf::from("./config");
    tokio::spawn(async move {
        tracing::info!("Starting binary initialization");

        if (DOWNLOAD_FFMPEG) {
            let ffmpeg_fetcher = FfmpegFetcher::new("ffmpeg".to_string());
            let ffmpeg_path = match download_and_extract_binary_path(
                ffmpeg_fetcher.get_release(&platform, &architecture).await.unwrap(),
                &config_dir,
            )
            .await
            {
                Ok(path) => {
                    println!("FFmpeg initialized: {:?}", path);
                    path
                }
                Err(e) => {
                    tracing::error!("Failed to initialize FFmpeg: {}", e);
                    return;
                }
            };
            set_binary_path("ffmpeg", ffmpeg_path);
        }

        if (DOWNLOAD_YTDLP) { 
            let ytdlp_fetcher = YtdlpFetcher::new();
            let ytdlp_path = match download_and_extract_binary_path(
                ytdlp_fetcher.get_release(&platform, &architecture).await.unwrap(),
                &config_dir,
            )
            .await
            {
                Ok(path) => {
                    println!("yt-dlp initialized: {:?}", path);
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
        window_event_handle_clone.load_url("http://localhost:8000/goldie".to_string());


        window_event_handle_clone.hide_window();
        thread::sleep(Duration::from_millis(1000));
        window_event_handle_clone.show_window();


        init_config_dir(config_dir);
    });

    match desktop::window::create_desktop_webview("http://localhost:8000/", event_loop) {
        Ok(_) => tracing::info!("Desktop app closed successfully"),
        Err(e) => tracing::error!("Desktop app error: {}", e),
    }


    let _ = server_handle.abort();
    tracing::info!("Application shutting down");
}
