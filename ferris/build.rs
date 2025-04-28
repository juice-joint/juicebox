use core::panic;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("huhehuahwefawef");

    let out_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");

    // Create assets directory if it doesn't exist
    fs::create_dir_all(&out_dir).unwrap();

    // Build Goldie React app
    println!("Building Goldie React app...");
    let goldie_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("goldie");

    if goldie_dir.exists() {
        // First install dependencies with bun
        println!("Installing Goldie dependencies...");
        let install_status = Command::new("bun")
            .current_dir(&goldie_dir)
            .arg("install")
            .status()
            .expect("Failed to install Goldie dependencies");

        if !install_status.success() {
            panic!("Failed to install Goldie dependencies");
        }

        // Then build with bun
        println!("Building Goldie app...");
        let build_status = Command::new("bun")
            .current_dir(&goldie_dir)
            .args(&["run", "build"])
            .status()
            .expect("Failed to build Goldie React app");

        if !build_status.success() {
            panic!("Failed to build Goldie React app");
        }

        // Copy the build output to our assets directory
        let goldie_build_dir = goldie_dir.join("dist");
        let goldie_asset_dir = out_dir.join("goldie");

        if goldie_build_dir.exists() {
            fs::create_dir_all(&goldie_asset_dir).unwrap();
            copy_dir_all(&goldie_build_dir, &goldie_asset_dir).unwrap();
            println!("Copied Goldie build to assets");
        } else {
            panic!("Goldie build directory not found");
        }
    } else {
        println!("Goldie directory not found, skipping build");
    }

    // Build Phippy React app
    println!("Building Phippy React app...");
    let phippy_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("..")
        .join("phippy");

    if phippy_dir.exists() {
        // First install dependencies with bun
        println!("Installing Phippy dependencies...");
        let install_status = Command::new("bun")
            .current_dir(&phippy_dir)
            .arg("install")
            .status()
            .expect("Failed to install Phippy dependencies");

        if !install_status.success() {
            panic!("Failed to install Phippy dependencies");
        }

        // Then build with bun
        println!("Building Phippy app...");
        let build_status = Command::new("bun")
            .current_dir(&phippy_dir)
            .args(&["run", "build"])
            .status()
            .expect("Failed to build Phippy React app");

        if !build_status.success() {
            panic!("Failed to build Phippy React app");
        }

        // Copy the build output to our assets directory
        let phippy_build_dir = phippy_dir.join("dist");
        let phippy_asset_dir = out_dir.join("phippy");

        if phippy_build_dir.exists() {
            fs::create_dir_all(&phippy_asset_dir).unwrap();
            copy_dir_all(&phippy_build_dir, &phippy_asset_dir).unwrap();
            println!("Copied Phippy build to assets");
        } else {
            panic!("Phippy build directory not found");
        }
    } else {
        println!("Phippy directory not found, skipping build");
    }
}

// Helper function to recursively copy directories
fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
