use once_cell::sync::OnceCell;
use std::path::PathBuf;
use std::collections::HashMap;
use std::sync::RwLock;

static CONFIG_DIR: OnceCell<PathBuf> = OnceCell::new();
static BINARY_PATHS: OnceCell<RwLock<HashMap<String, PathBuf>>> = OnceCell::new();

fn get_binary_paths() -> &'static RwLock<HashMap<String, PathBuf>> {
    BINARY_PATHS.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn init_config_dir(path: PathBuf) {
    CONFIG_DIR.set(path).expect("Config dir already set");
}

pub fn set_binary_path(binary_name: &str, path: PathBuf) {
    if let Ok(mut paths) = get_binary_paths().write() {
        paths.insert(binary_name.to_string(), path);
    } else {
        panic!("Failed to acquire write lock for binary paths");
    }
}

pub fn get_binary_path(name: &str) -> PathBuf {
    // First check if we have a custom path set
    if let Ok(paths) = get_binary_paths().read() {
        if let Some(path) = paths.get(name) {
            return path.clone();
        }
    }
    
    // Fall back to the default path in CONFIG_DIR
    CONFIG_DIR
        .get()
        .expect("Config dir not initialized")
        .join(if cfg!(windows) {
            format!("{}.exe", name)
        } else {
            name.to_string()
        })
}