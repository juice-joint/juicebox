use std::{path::{Path, PathBuf}, process::{Command, Output}};

use deps::{FetcherError, Release, ReleaseFetcher};
use thiserror::Error;
use tracing::debug;
use utils::{architecture::Architecture, platform::Platform};
use zip::result::ZipError;

pub mod deps;
pub mod utils;

#[derive(Error, Debug)]
pub enum ExtractError {
    #[error("Zip extraction failed for {0} with error: {1}")]
    ZipExtractionError(String, #[source] ZipError),

    #[error("IO error during extraction: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TarXz extraction failed: {0}")]
    TarXzExtractionError(String),

    #[error("TarGz extraction failed: {0}")]
    TarGzExtractionError(String),

    #[error("Binary not found: {0}")]
    BinaryNotFound(String),

    #[error("Task execution error: {0}")]
    TaskError(#[from] tokio::task::JoinError),

    #[error("Unsupported archive format: {0}")]
    UnsupportedFormat(String),

    #[error("Failed to fetch release: {0}")]
    FetchError(#[from] FetcherError),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Failed to execute binary: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Binary execution failed with exit code: {0}")]
    NonZeroExit(i32),
    
    #[error("Binary execution terminated by signal")]
    TerminatedBySignal,
}

#[derive(Debug, Clone)]
pub struct Binary {
    /// Path to the binary executable
    path: PathBuf,
}

impl Binary {
    /// Create a new Binary instance
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    
    /// Get the path to the binary
    pub fn path(&self) -> &Path {
        &self.path
    }
    
    /// Execute the binary with the given arguments
    pub fn execute(&self, args: &[&str]) -> Result<Output, ExecutionError> {
        debug!("Executing binary at {:?} with args: {:?}", self.path, args);
        
        let output = Command::new(&self.path)
            .args(args)
            .output()?;
            
        if !output.status.success() {
            match output.status.code() {
                Some(code) => return Err(ExecutionError::NonZeroExit(code)),
                None => return Err(ExecutionError::TerminatedBySignal),
            }
        }
        
        Ok(output)
    }
}

pub async fn download_and_extract_binary(
    release: Release,
    destination_dir: impl AsRef<Path>
) -> Result<Binary, ExtractError> {
    let binary_path = download_and_extract_binary_path(release, destination_dir).await?;
    Ok(Binary::new(binary_path))
}

pub async fn download_and_extract_binary_path(
    release: Release,
    destination_dir: impl AsRef<Path>
) -> Result<PathBuf, ExtractError> {
    // let release = release_fetcher.get_release(platform, architecture).await
    //     .map_err(|err| ExtractError::FetchError(err))?;
    
    let destination_dir = destination_dir.as_ref();
    tokio::fs::create_dir_all(destination_dir).await?;
    
    // Download the archive
    debug!(
        "Downloading binary from {} to {:?}",
        release.url, destination_dir
    );
    
    let response = reqwest::get(&release.url)
        .await
        .map_err(|e| {
            ExtractError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Download failed: {}", e),
            ))
        })?
        .error_for_status()
        .map_err(|e| {
            ExtractError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP error: {}", e),
            ))
        })?;
    
    let bytes = response.bytes().await.map_err(|e| {
        ExtractError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read response bytes: {}", e),
        ))
    })?;
    
    // Determine the file type
    let is_zip = release.url.ends_with(".zip");
    let is_tar_xz = release.url.ends_with(".tar.xz");
    let is_tar_gz = release.url.ends_with(".tar.gz") || release.url.ends_with(".tgz");
    
    // Create a temporary directory for archive extraction if needed
    let temp_dir = tempfile::tempdir().map_err(|e| {
        ExtractError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create temp dir: {}", e),
        ))
    })?;
    
    let binary_path = if is_zip || is_tar_xz || is_tar_gz {
        // Handle archive formats
        let archive_path = temp_dir.path().join("downloaded_archive");
        tokio::fs::write(&archive_path, &bytes).await?;
        
        if is_zip {
            extract_binary_from_zip(&archive_path, destination_dir, &release.binary_name).await?
        } else if is_tar_xz {
            extract_binary_from_tarxz(&archive_path, destination_dir, &release.binary_name).await?
        } else {
            extract_binary_from_targz(&archive_path, destination_dir, &release.binary_name).await?
        }
    } else {
        // Not an archive, just write the binary directly
        let binary_path = destination_dir.join(&release.binary_name);
        tokio::fs::write(&binary_path, &bytes).await?;
        binary_path
    };
    
    // Make the binary executable (on Unix platforms)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = tokio::fs::metadata(&binary_path).await?.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        tokio::fs::set_permissions(&binary_path, perms).await?;
    }
    
    Ok(binary_path)
}

async fn extract_binary_from_zip(
    archive_path: &Path,
    destination_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, ExtractError> {
    let archive_path = archive_path.to_path_buf();
    let extract_dir = tempfile::tempdir()?;
    let extract_dir_path = extract_dir.path().to_path_buf();
    let archive_path_str = archive_path.to_string_lossy().to_string();

    tokio::task::spawn_blocking(move || -> Result<(), ExtractError> {
        let file = std::fs::File::open(&archive_path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| ExtractError::ZipExtractionError(archive_path_str.clone(), e))?;

        for i in 0..archive.len() {
            let mut file = archive.by_index(i).map_err(|e| {
                ExtractError::ZipExtractionError(format!("{}[{}]", archive_path_str, i), e)
            })?;

            let outpath = extract_dir_path.join(file.name());

            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                let mut outfile = std::fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }

        Ok(())
    })
    .await??;

    let binary = find_binary(&extract_dir.path(), binary_name).await?;

    let filename = binary.file_name().ok_or_else(|| {
        ExtractError::BinaryNotFound(format!("Invalid filename for binary: {}", binary_name))
    })?;

    let destination = destination_dir.join(filename);
    tokio::fs::copy(&binary, &destination).await?;

    Ok(destination)
}

async fn extract_binary_from_tarxz(
    archive_path: &Path,
    destination_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, ExtractError> {
    let archive_path = archive_path.to_path_buf();
    let extract_dir = tempfile::tempdir()?;
    let extract_dir_path = extract_dir.path().to_path_buf();
    let archive_path_str = archive_path.to_string_lossy().to_string();

    // Extract in a blocking task
    tokio::task::spawn_blocking(move || -> Result<(), ExtractError> {
        let file = std::fs::File::open(&archive_path)?;
        let xz_decoder = xz2::read::XzDecoder::new(file);
        let mut archive = tar::Archive::new(xz_decoder);

        archive.unpack(&extract_dir_path).map_err(|e| {
            ExtractError::TarXzExtractionError(format!(
                "Failed to extract {}: {}",
                archive_path_str, e
            ))
        })?;

        Ok(())
    })
    .await??;

    // Find the binary
    let binary = find_binary(&extract_dir.path(), binary_name).await?;

    // Copy to destination
    let filename = binary.file_name().ok_or_else(|| {
        ExtractError::BinaryNotFound(format!("Invalid filename for binary: {}", binary_name))
    })?;
    let destination = destination_dir.join(filename);
    tokio::fs::copy(&binary, &destination).await?;

    Ok(destination)
}

async fn extract_binary_from_targz(
    archive_path: &Path,
    destination_dir: &Path,
    binary_name: &str,
) -> Result<PathBuf, ExtractError> {
    let archive_path = archive_path.to_path_buf();
    let extract_dir = tempfile::tempdir()?;
    let extract_dir_path = extract_dir.path().to_path_buf();
    let archive_path_str = archive_path.to_string_lossy().to_string();

    // Extract in a blocking task
    tokio::task::spawn_blocking(move || -> Result<(), ExtractError> {
        let file = std::fs::File::open(&archive_path)?;
        let gz_decoder = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz_decoder);

        archive.unpack(&extract_dir_path).map_err(|e| {
            ExtractError::TarGzExtractionError(format!(
                "Failed to extract {}: {}",
                archive_path_str, e
            ))
        })?;

        Ok(())
    })
    .await??;

    // Find the binary
    let binary = find_binary(&extract_dir.path(), binary_name).await?;

    // Copy to destination
    let filename = binary.file_name().ok_or_else(|| {
        ExtractError::BinaryNotFound(format!("Invalid filename for binary: {}", binary_name))
    })?;
    let destination = destination_dir.join(filename);
    tokio::fs::copy(&binary, &destination).await?;

    Ok(destination)
}

async fn find_binary(dir: &Path, binary_name: &str) -> Result<PathBuf, ExtractError> {
    // On Windows, we might look for binary_name.exe
    let windows_binary_name = format!("{}.exe", binary_name);
    let binary_name_clone = binary_name.to_string();

    // Use tokio::task::spawn_blocking for the directory traversal since WalkDir isn't async
    let dir = dir.to_path_buf();
    let result = tokio::task::spawn_blocking(move || -> Result<PathBuf, ExtractError> {
        for entry in walkdir::WalkDir::new(&dir) {
            let entry = entry.map_err(|e| {
                ExtractError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("WalkDir error: {}", e),
                ))
            })?;

            let path = entry.path();

            // Check if this is the binary
            if path.is_file() {
                let filename = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");

                // Look for binary or binary.exe
                if filename == binary_name_clone || filename == windows_binary_name {
                    return Ok(path.to_path_buf());
                }
            }
        }

        Err(ExtractError::BinaryNotFound(format!(
            "Binary '{}' not found in extracted files",
            binary_name_clone
        )))
    })
    .await??;

    Ok(result)
}
