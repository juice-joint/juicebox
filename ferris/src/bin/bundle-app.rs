use std::{collections::HashMap, path::{Path, PathBuf}};
use serde::{Deserialize, Serialize};
use tauri_bundler::{
    bundle_project, AppImageSettings, BundleBinary, BundleSettings, DebianSettings, DmgSettings, 
    IosSettings, MacOsSettings, PackageSettings, PackageType, Position, RpmSettings, Settings, 
    SettingsBuilder, Size, WindowsSettings, AppCategory
};
use tauri_utils::{config::WebviewInstallMode, platform::Target};
use tracing::Level;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let package_settings = PackageSettings {
        product_name: "juicebox".to_string(),
        version: "1.0".to_string(),
        description: "loool".to_string(),
        homepage: None,
        authors: Some(vec!["jpiece".to_string(), "bigbitchtrev".to_string()]),
        default_run: Some("juicebox".to_string()) // Add default_run to specify the main binary
    };

    let package_types = vec![
        PackageType::from_short_name("deb").unwrap(),
        PackageType::from_short_name("dmg").unwrap()
    ];

    let bundle_settings = BundleSettings {
        identifier: Some("com.example.app".to_string()),
        publisher: Some("Example Company".to_string()),
        homepage: Some("https://example.com".to_string()),
        icon: Some(vec!["/home/jaredp/icon.ico".to_string()]),
        resources: None,
        resources_map: None,
        copyright: Some("Copyright © 2024 Example Company".to_string()),
        license: Some("MIT".to_string()),
        license_file: None,
        category: Some(AppCategory::Utility), // Set a proper category
        file_associations: Some(vec![]),
        short_description: Some("A short description of the app".to_string()),
        long_description: Some("A longer, more detailed description of the application".to_string()),
        bin: None,
        external_bin: None,
        deep_link_protocols: None,
        deb: DebianSettings {
            depends: None,
            recommends: None,
            provides: None,
            conflicts: None,
            replaces: None,
            files: HashMap::new(),
            desktop_template: None,
            section: None,
            priority: None,
            changelog: None,
            pre_install_script: None,
            post_install_script: None,
            pre_remove_script: None,
            post_remove_script: None
        },
        appimage: AppImageSettings {
            files: HashMap::new(),
            bundle_media_framework: true,
            bundle_xdg_open: true
        },
        rpm: RpmSettings {
            depends: None,
            recommends: None,
            provides: None,
            conflicts: None,
            obsoletes: None,
            release: "1.0".to_string(),
            epoch: 1,
            files: HashMap::new(),
            desktop_template: None,
            pre_install_script: None,
            post_install_script: None,
            pre_remove_script: None,
            post_remove_script: None,
            compression: None
        },
        dmg: DmgSettings {
            background: None,
            window_position: None,
            window_size: Size { width: 200, height: 100 },
            app_position: Position { x: 0, y: 0 },
            application_folder_position: Position { x: 100, y: 0 }
        },
        ios: IosSettings {
            bundle_version: None
        },
        macos: MacOsSettings {
            frameworks: None,
            files: HashMap::new(),
            bundle_version: None,
            minimum_system_version: None,
            exception_domain: None,
            signing_identity: None,
            hardened_runtime: false,
            provider_short_name: None,
            entitlements: None,
            info_plist_path: None
        },
        updater: None,
        windows: WindowsSettings {
            digest_algorithm: None,
            certificate_thumbprint: None,
            timestamp_url: None,
            tsp: true,
            wix: None,
            nsis: None,
            icon_path: PathBuf::new(), // Use a proper path
            webview_install_mode: WebviewInstallMode::EmbedBootstrapper { silent: true },
            allow_downgrades: false,
            sign_command: None
        }
    };

    let settings = SettingsBuilder::new()
        .package_settings(package_settings)
        .package_types(package_types)
        .project_out_directory("/home/jaredp/Desktop/juicebox/ferris/target/debug/")
        .bundle_settings(bundle_settings)
        .binaries(vec![BundleBinary::new("juicebox".to_string(), true)])
        .build();

    println!("{:?}", bundle_project(&settings.unwrap()));

    Ok(())
}

// To use this, add to your Cargo.toml:
/*
[dependencies]
tauri-bundler = "2.0.0"  # or whatever version you need
tauri-utils = "2.0.0"   # if you need platform utilities
serde = { version = "1.0", features = ["derive"] }
tracing = "0.1"
thiserror = "1.0"  # Only if you want custom error types
*/