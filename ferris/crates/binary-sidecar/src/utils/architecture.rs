//! Architecture detection.
//! From: https://github.com/boul2gom/yt-dlp/blob/develop/src/utils/platform.rs

use derive_more::Display;

/// Represents the architecture of the CPU where the program is running.
#[derive(Clone, Debug, Display)]
pub enum Architecture {
    /// The x64 architecture.
    #[display("x64")]
    X64,
    /// The x86_64 architecture.
    #[display("x86")]
    X86,
    /// The ARMv7l architecture.
    #[display("armv7l")]
    Armv7l,
    /// The Aarch64 (Arm64) architecture.
    #[display("aarch64")]
    Aarch64,

    /// An unknown architecture.
    #[display("Unknown: {}", _0)]
    Unknown(String),
}

impl Architecture {
    /// Detects the current architecture of the CPU where the program is running.
    pub fn detect() -> Self {
        let arch = std::env::consts::ARCH;

        tracing::debug!("Detected architecture: {}", arch);

        match arch {
            "x86_64" => Architecture::X64,
            "x86" => Architecture::X86,
            "armv7l" => Architecture::Armv7l,
            "aarch64" => Architecture::Aarch64,
            _ => Architecture::Unknown(arch.to_string()),
        }
    }
}