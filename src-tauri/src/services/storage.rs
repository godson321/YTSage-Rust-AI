use std::env;
use std::fs;
use std::io;
use std::cell::RefCell;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

thread_local! {
    static DATA_DIR_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

pub fn data_dir() -> PathBuf {
    if let Some(override_dir) = DATA_DIR_OVERRIDE.with(|cell| cell.borrow().clone()) {
        return override_dir;
    }

    if let Some(custom_dir) = env::var_os("YTSAGE_RUST_DATA_DIR") {
        return PathBuf::from(custom_dir);
    }

    if let Some(appdata) = env::var_os("APPDATA") {
        return PathBuf::from(appdata).join("ytsage-rust");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home).join(".config").join("ytsage-rust");
    }

    env::temp_dir().join("ytsage-rust")
}

pub fn set_data_dir_override(path: PathBuf) {
    DATA_DIR_OVERRIDE.with(|cell| {
        *cell.borrow_mut() = Some(path);
    });
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

pub fn history_path() -> PathBuf {
    data_dir().join("history.json")
}

pub fn log_path() -> PathBuf {
    data_dir().join("logs").join("app.log")
}

pub fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

pub fn read_json_file<T: DeserializeOwned>(path: &Path) -> io::Result<T> {
    let content = fs::read_to_string(path)?;
    serde_json::from_str(&content).map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

pub fn write_json_file<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    ensure_parent_dir(path)?;
    let content = serde_json::to_string_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    fs::write(path, content)
}
