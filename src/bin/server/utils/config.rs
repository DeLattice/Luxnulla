use dirs::{self};
use luxnulla::{CONFIG_DIR, XRAY_CONFIG_FILE, XRAY_LOG_FILE};
use std::{fs, path::PathBuf};

//$HOME/.config/luxnulla/
pub fn app_config_dir() -> PathBuf {
    let app_dir = dirs::config_dir()
        .expect("Failed to find config directory.")
        .join(CONFIG_DIR);

    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)
            .unwrap_or_else(|e| panic!("Failed to create app config directory: {}", e));
    };

    app_dir
}

pub fn xray_config_file() -> PathBuf {
    app_config_dir().join(XRAY_CONFIG_FILE)
}

pub fn xray_log_file() -> PathBuf {
    app_config_dir().join("xray").join(XRAY_LOG_FILE)
}
