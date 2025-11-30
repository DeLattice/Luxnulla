use luxnulla::CONFIG_DIR;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use crate::http::models::xray_config::XrayOutboundClientConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct XrayInboundSettings {
    pub auth: String,
    pub udp: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct XrayInboundClientConfig {
    pub tag: String,
    pub port: u16,
    pub listen: String,
    pub protocol: String,
    pub settings: XrayInboundSettings,
}

#[derive(Debug, Deserialize)]
pub struct XrayFileCore {
    pub xray_config_path: PathBuf,
}

impl XrayFileCore {
    pub fn new(xray_config_file: &str) -> Self {
        let config_dir = dirs::config_dir().unwrap().join(CONFIG_DIR);
        if !config_dir.exists() {
            let _ = fs::create_dir_all(&config_dir);
        }
        XrayFileCore {
            xray_config_path: config_dir.join(xray_config_file),
        }
    }

    pub fn get_path_string(&self) -> String {
        self.xray_config_path.to_string_lossy().to_string()
    }

    fn load_json(&self) -> Value {
        File::open(&self.xray_config_path)
            .ok()
            .and_then(|f| serde_json::from_reader(f).ok())
            .unwrap_or_else(|| json!({}))
    }

    fn save_json(&self, data: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(data)?;

        let mut file = File::create(&self.xray_config_path)?;
        file.write_all(content.as_bytes())?;

        file.sync_all()?;

        Ok(())
    }

    pub fn write_full_config(&self, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
        self.save_json(config)
    }

    pub fn read_xray_file(&self) -> Result<Value, Box<dyn std::error::Error>> {
        Ok(self.load_json())
    }

    pub fn get_section<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        let root = self.load_json();
        let section = root.get(key).cloned().unwrap_or(json!([]));
        Ok(serde_json::from_value(section).unwrap_or_default())
    }

    pub fn set_section<T: Serialize>(
        &self,
        key: &str,
        data: &Vec<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut root = self.load_json();
        root[key] = serde_json::to_value(data)?;
        self.save_json(&root)
    }

    pub fn delete_section(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut root = self.load_json();
        if let Some(obj) = root.as_object_mut() {
            obj.remove(key);
        }
        self.save_json(&root)
    }

    pub fn delete_xray_outbound_by_id(&self, id: &i32) -> Result<(), Box<dyn std::error::Error>> {
        let mut root = self.load_json();
        if let Some(outbounds) = root.get_mut("outbounds").and_then(|v| v.as_array_mut()) {
            outbounds.retain(|item| {
                item.get("tag")
                    .and_then(|t| t.as_str())
                    .and_then(|s| s.parse::<i32>().ok())
                    .map_or(true, |tag_id| tag_id != *id)
            });
        }
        self.save_json(&root)
    }

    pub fn read_xray_inbounds(
        &self,
    ) -> Result<Vec<XrayInboundClientConfig>, Box<dyn std::error::Error>> {
        self.get_section("inbounds")
    }

    pub fn write_xray_inbounds(
        &self,
        data: Vec<XrayInboundClientConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.set_section("inbounds", &data)
    }

    pub fn read_xray_outbounds(
        &self,
    ) -> Result<Vec<XrayOutboundClientConfig>, Box<dyn std::error::Error>> {
        self.get_section("outbounds")
    }

    pub fn write_xray_outbounds(
        &self,
        data: Vec<XrayOutboundClientConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.set_section("outbounds", &data)
    }
}
