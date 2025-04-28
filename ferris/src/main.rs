use binary_sidecar::{
    deps::{ffmpeg::FfmpegFetcher, ytdlp::YtdlpFetcher},
    download_and_extract_binary,
    utils::{architecture::Architecture, platform::Platform},
};
use server::globals::{init_config_dir, set_binary_path};
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::oneshot;

mod desktop;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let platform = Platform::detect();
    let architecture = Architecture::detect();

    let config_dir = PathBuf::from("./config");

    let ffmpeg_fetcher = FfmpegFetcher::new();
    let ffmpeg_path =
        download_and_extract_binary(&ffmpeg_fetcher, &config_dir, &platform, &architecture)
            .await
            .unwrap();
    println!("{:?}", ffmpeg_path);
    set_binary_path("ffmpeg", ffmpeg_path);
    
    let ytdlp_fetcher = YtdlpFetcher::new();
    let ytdlp_path = download_and_extract_binary(
        &ytdlp_fetcher,
        &config_dir,
        &platform,
        &architecture
    )
        .await
        .unwrap();
    println!("{:?}", ytdlp_path);
    set_binary_path("yt-dlp", ytdlp_path);

    init_config_dir(config_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    tracing::info!("Starting server on {}", addr);

    let (tx, rx) = oneshot::channel();

    let server_handle = tokio::spawn(async move {
        server::run_server(addr, tx).await;
    });

    rx.await.expect("Failed to receive server ready signal");
    tracing::info!("Server is ready");

    match desktop::window::run_desktop_app("http://localhost:8000/goldie") {
        Ok(_) => tracing::info!("Desktop app closed successfully"),
        Err(e) => tracing::error!("Desktop app error: {}", e),
    }

    let _ = server_handle.abort();
    tracing::info!("Application shutting down");
}
