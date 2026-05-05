use std::sync::Mutex;

use serde::{Deserialize, Serialize};

use crate::models::AppSettings;

use super::storage;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SettingsFile {
    settings: AppSettings,
}

static SETTINGS_LOCK: Mutex<()> = Mutex::new(());

fn default_settings() -> AppSettings {
    AppSettings {
        download_path: String::new(),
        generic_mode: false,
        language: "zh".to_string(),
        proxy_url: None,
        geo_proxy_url: None,
        filename_format: "%(title)s_%(resolution)s.%(ext)s".to_string(),
    }
}

fn load_settings_file() -> AppSettings {
    let path = storage::config_path();
    match storage::read_json_file::<SettingsFile>(&path) {
        Ok(file) => file.settings,
        Err(_) => default_settings(),
    }
}

fn save_settings_file(settings: &AppSettings) {
    let payload = SettingsFile {
        settings: settings.clone(),
    };
    let _ = storage::write_json_file(&storage::config_path(), &payload);
}

pub fn get_settings() -> AppSettings {
    let _guard = SETTINGS_LOCK.lock().expect("settings lock poisoned");
    load_settings_file()
}

pub fn save_settings(settings: AppSettings) -> AppSettings {
    let _guard = SETTINGS_LOCK.lock().expect("settings lock poisoned");
    save_settings_file(&settings);
    settings
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use super::storage;
    use uuid::Uuid;

    fn set_temp_data_dir() {
        let dir = env::temp_dir().join(format!("ytsage-rust-settings-{}", Uuid::new_v4()));
        storage::set_data_dir_override(dir);
    }

    #[test]
    fn save_settings_persists_download_path_and_language() {
        set_temp_data_dir();

        let settings = AppSettings {
            download_path: "C:/Downloads".to_string(),
            generic_mode: true,
            language: "en".to_string(),
            proxy_url: Some("http://localhost:8080".to_string()),
            geo_proxy_url: None,
            filename_format: "%(title)s.%(ext)s".to_string(),
        };

        save_settings(settings.clone());
        let loaded = get_settings();

        assert_eq!(loaded.download_path, "C:/Downloads");
        assert_eq!(loaded.language, "en");
    }
}
