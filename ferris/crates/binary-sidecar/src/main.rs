use std::env::current_dir;
// Import everything from the binary_sidecar crate
use binary_sidecar::{
    download_and_extract_binary,
    // Use reexported types from the library instead of local types
    utils::platform::Platform,
    utils::architecture::Architecture,
    deps::{
        ffmpeg::FfmpegFetcher, ytdlp::YtdlpFetcher
    }
};
// Skip importing the local utils that have conflicting types
// use utils::{architecture::Architecture, platform::Platform};

#[tokio::main]
async fn main() {
    println!("Hello, world!");
    // Use Platform and Architecture from binary_sidecar library
    let platform = Platform::detect();
    let architecture = Architecture::detect();
    let destination_dir = current_dir().unwrap();
    
    let ffmpeg_fetcher = FfmpegFetcher::new();
    let ffmpeg_path = download_and_extract_binary(
        &ffmpeg_fetcher,
        &destination_dir,
        &platform,
        &architecture
    )
    .await
    .unwrap();
    println!("{:?}", ffmpeg_path);
    
    let ytdlp_fetcher = YtdlpFetcher::new();
    let ytdlp_path = download_and_extract_binary(
        &ytdlp_fetcher,
        &destination_dir,
        &platform,
        &architecture
    )
        .await
        .unwrap();
    println!("{:?}", ytdlp_path);
}