use core::panic;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("assets");

    let goldie_dir = Path::new(&manifest_dir).join("..").join("goldie");
    let phippy_dir = Path::new(&manifest_dir).join("..").join("phippy");

    // little hack to rerun the builds everytime on cargo build/run
    if goldie_dir.exists() {
        println!("cargo:rerun-if-changed={}", goldie_dir.display());
    }

    if phippy_dir.exists() {
        println!("cargo:rerun-if-changed={}", phippy_dir.display());
    }

    fs::create_dir_all(&out_dir).unwrap();

    build_goldie(&goldie_dir, &out_dir);
    build_phippy(&phippy_dir, &out_dir);
}

fn build_goldie(goldie_dir: &PathBuf, out_dir: &PathBuf) {
    println!("Building Goldie React app...");
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
}

fn build_phippy(phippy_dir: &PathBuf, out_dir: &PathBuf) {
    println!("Building Phippy React app...");
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

// Recursively copy directories
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
