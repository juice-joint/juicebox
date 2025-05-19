use derive_more::Display;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;
use thiserror::Error;
use which::which;

use crate::utils::{architecture::Architecture, platform::Platform};

use super::{FetcherError, Release, ReleaseFetcher};

const YTDLP_ASSET_NAME: &'static str = "yt-dlp";

#[derive(Debug, Deserialize, Display)]
#[display("Release: tag={}, assets={};", tag_name, assets.len())]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize, Display)]
#[display("Asset: name={}, url={};", name, download_url)]
pub struct GithubAsset {
    pub name: String,
    #[serde(rename = "browser_download_url")]
    pub download_url: String,
}

pub struct YtdlpFetcher {}

impl YtdlpFetcher {
    pub fn new() -> YtdlpFetcher {
        YtdlpFetcher {}
    }
}

impl ReleaseFetcher for YtdlpFetcher {
    async fn get_release(
        &self,
        platform: &Platform,
        architecture: &Architecture,
    ) -> Result<Release, FetcherError> {
        let owner = "yt-dlp";
        let repo = "yt-dlp";
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );

        let json_response = fetch_json(&url, None)
            .await
            .map_err(|e| FetcherError::NetworkError(format!("Failed to fetch release: {}", e)))?;

        let github_release: GitHubRelease = serde_json::from_value(json_response).map_err(|e| {
            FetcherError::ParseError(format!("Failed to parse GitHub release: {}", e))
        })?;

        let asset = github_release
            .assets
            .iter()
            .find(|asset| {
                let name = &asset.name;
                match (platform, architecture) {
                    (Platform::Windows, Architecture::X64) => {
                        name.contains(&format!("{}.exe", YTDLP_ASSET_NAME))
                    }
                    (Platform::Windows, Architecture::X86) => {
                        name.contains(&format!("{}_x86.exe", YTDLP_ASSET_NAME))
                    }
                    (Platform::Linux, Architecture::X64) => {
                        name.contains(&format!("{}_linux", YTDLP_ASSET_NAME))
                    }
                    (Platform::Linux, Architecture::Armv7l) => {
                        name.contains(&format!("{}_linux_armv7l", YTDLP_ASSET_NAME))
                    }
                    (Platform::Linux, Architecture::Aarch64) => {
                        name.contains(&format!("{}_linux_aarch64", YTDLP_ASSET_NAME))
                    }
                    (Platform::Mac, _) => name.contains(&format!("{}*macos", YTDLP_ASSET_NAME)),
                    _ => false,
                }
            })
            .ok_or_else(|| {
                FetcherError::AssetNotFound(format!(
                    "Could not find asset for platform: {:?}, architecture: {:?}",
                    platform, architecture
                ))
            })?;

        Ok(Release {
            url: asset.download_url.to_owned(),
            binary_name: asset.name.to_owned(),
        })
    }
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Invalid header value: {0}")]
    InvalidHeader(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub async fn fetch_json(url: &str, auth_token: Option<String>) -> Result<Value, ApiError> {
    #[cfg(feature = "tracing")]
    tracing::debug!("Fetching JSON from {}", self.url);

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("rust-reqwest"));

    if let Some(auth_token) = auth_token {
        let value = HeaderValue::from_str(&format!("Bearer {}", auth_token))
            .map_err(|e| ApiError::InvalidHeader(e.to_string()))?;

        headers.insert(reqwest::header::AUTHORIZATION, value);
    }

    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await?
        .error_for_status()?;

    let json = response.json().await?;
    Ok(json)
}
