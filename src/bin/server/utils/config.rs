use dirs;
use elux::{CONFIG_DIR, XRAY_CONFIG_FILE, XRAY_LOG_FILE};
use std::{fs, path::PathBuf, sync::OnceLock};

use crate::utils::templates;

pub struct AppPaths {
    pub config_dir: PathBuf,
    pub xray_config: PathBuf,
    pub xray_log: PathBuf,
}

static INSTANCE: OnceLock<AppPaths> = OnceLock::new();

impl AppPaths {
    pub fn init() {
        let config_dir = dirs::config_dir()
            .expect("Failed to find config directory")
            .join(CONFIG_DIR);

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).expect("Failed to create app config directory");
        }

        let xray_log_dir = config_dir.join("xray");
        if !xray_log_dir.exists() {
            fs::create_dir_all(&xray_log_dir).expect("Failed to create xray log directory");
        }

        let xray_log = xray_log_dir.join(XRAY_LOG_FILE);
        if !xray_log.exists() {
            fs::File::create(&xray_log).expect("Failed to create xray log file");
        }

        let xray_config = config_dir.join(XRAY_CONFIG_FILE);
        if !xray_config.exists() {
            let config_content = templates::get_init_xray_config(&xray_log);

            std::fs::write(&xray_config, config_content).expect("Failed to create xray log file");
        }

        let paths = AppPaths {
            config_dir,
            xray_config,
            xray_log,
        };

        INSTANCE.set(paths).ok();
    }

    pub fn get() -> &'static AppPaths {
        INSTANCE.get().expect("AppPaths is not initialized")
    }
}
