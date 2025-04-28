use crate::utils::{architecture::Architecture, platform::Platform};

use super::{FetcherError, Release, ReleaseFetcher};

const WINDOWS_FFMPEG_URL: &'static str =
    "https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip";
const MACOS_INTEL_FFMPEG_URL: &'static str = "https://www.osxexperts.net/ffmpeg71intel.zip";
const MACOS_ARM_FFMPEG_URL: &'static str = "https://www.osxexperts.net/ffmpeg71arm.zip";
const LINUX_FFMPEG_URL_TEMPLATE: &'static str =
    "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-{}-static.tar.xz";

pub struct FfmpegFetcher {}

impl FfmpegFetcher {
    pub fn new() -> FfmpegFetcher {
        FfmpegFetcher {}
    }
}

impl ReleaseFetcher for FfmpegFetcher {
    async fn get_release(
        &self,
        platform: &Platform,
        architecture: &Architecture,
    ) -> Result<Release, FetcherError> {
        let url = match platform {
            Platform::Windows => WINDOWS_FFMPEG_URL.to_string(),
            Platform::Mac => match architecture {
                Architecture::X64 => MACOS_INTEL_FFMPEG_URL.to_string(),
                Architecture::Aarch64 => MACOS_ARM_FFMPEG_URL.to_string(),
                _ => MACOS_INTEL_FFMPEG_URL.to_string(),
            },
            Platform::Linux => {
                let arch_str = match architecture {
                    Architecture::X64 => "amd64",
                    Architecture::X86 => "i686",
                    Architecture::Aarch64 => "arm64",
                    Architecture::Armv7l => "armv7",
                    Architecture::Unknown(_) => "amd64",
                };

                LINUX_FFMPEG_URL_TEMPLATE.replace("{}", arch_str)
            }
            Platform::Unknown(_) => {
                // Default to Linux AMD64 for unknown platforms
                LINUX_FFMPEG_URL_TEMPLATE.replace("{}", "amd64")
            }
        };

        Ok(Release {
            url,
            binary_name: String::from("ffmpeg"),
        })
    }
}
