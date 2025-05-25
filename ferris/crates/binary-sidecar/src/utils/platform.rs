//! Platform detection.
//! From: https://github.com/boul2gom/yt-dlp/blob/develop/src/utils/platform.rs

use derive_more::Display;

/// Represents the operating system where the program is running.
#[derive(Clone, Debug, Display)]
pub enum Platform {
    /// The Windows operating system.
    #[display("Windows")]
    Windows,
    /// The Linux operating system.
    #[display("Linux")]
    Linux,
    /// The macOS operating system.
    #[display("MacOS")]
    Mac,

    /// An unknown operating system.
    #[display("Unknown: {}", _0)]
    Unknown(String),
}

impl Platform {
    /// Detects the current platform where the program is running.
    pub fn detect() -> Self {
        let os = std::env::consts::OS;

        tracing::debug!("Detected platform: {}", os);

        match os {
            "windows" => Platform::Windows,
            "linux" => Platform::Linux,
            "macos" => Platform::Mac,
            _ => Platform::Unknown(os.to_string()),
        }
    }
}