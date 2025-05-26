use std::env::current_dir;
// Import everything from the binary_sidecar crate
use binary_sidecar::{
    deps::{ffmpeg::FfmpegFetcher, ytdlp::YtdlpFetcher, ReleaseFetcher},
    download_and_extract_binary,
    utils::{architecture::Architecture, platform::Platform},
};
// Skip importing the local utils that have conflicting types
// use utils::{architecture::Architecture, platform::Platform};

#[tokio::main]
async fn main() {
    // Use Platform and Architecture from binary_sidecar library
    let platform = Platform::detect();
    let architecture = Architecture::detect();
    let destination_dir = current_dir().unwrap();

    let ffmpeg_fetcher = FfmpegFetcher::new("ffmpeg".to_string());
    let ffmpeg_path =
        download_and_extract_binary(&ffmpeg_fetcher, &destination_dir, &platform, &architecture)
            .await
            .unwrap();
    println!("{:?}", ffmpeg_path);

    let ffmpeg_release = ffmpeg_fetcher
        .get_release(&platform, &architecture)
        .await
        .unwrap();
    let ffmpeg_binary = download_and_extract_binary(ffmpeg_release, &destination_dir);

    let ytdlp_fetcher = YtdlpFetcher::new();
    let ytdlp_path =
        download_and_extract_binary(&ytdlp_fetcher, &destination_dir, &platform, &architecture)
            .await
            .unwrap();
    println!("{:?}", ytdlp_path);
}
