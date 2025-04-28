use core::fmt;
use std::path::{Path, PathBuf};

use thiserror::Error;
use tracing::debug;
use zip::result::ZipError;

use crate::utils::{architecture::Architecture, platform::Platform};

pub mod ffmpeg;
pub mod ytdlp;

#[derive(Error, Debug)]
pub enum DownloadError {
    #[error("Failed to make HTTP request: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Failed to write file: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct Release {
    pub url: String,
    pub binary_name: String,
}

impl fmt::Display for Release {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Release: {}", self.url)
    }
}

// Error type for the ReleaseFetcher trait
#[derive(Debug, thiserror::Error)]
pub enum FetcherError {
    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Asset not found: {0}")]
    AssetNotFound(String),

    #[error("Other error: {0}")]
    Other(String),
}

pub trait ReleaseFetcher {
    async fn get_release(
        &self,
        platform: &Platform,
        architecture: &Architecture,
    ) -> Result<Release, FetcherError>;
}