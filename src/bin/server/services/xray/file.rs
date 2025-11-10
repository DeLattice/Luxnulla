use luxnulla::CONFIG_DIR;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::http::services::model::xray_config::XrayOutboundClientConfig;

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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PartialXraySettingsFile {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inbounds: Option<Vec<XrayInboundClientConfig>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outbounds: Option<Vec<XrayOutboundClientConfig>>,
}

#[derive(Debug, Deserialize)]
pub struct XrayFileCore {
    pub xray_config_path: PathBuf,
}

impl XrayFileCore {
    pub fn new(xray_config_file: &str) -> Self {
        let config_dir_path = dirs::config_dir().unwrap().join(CONFIG_DIR);
        if !config_dir_path.exists() {
            std::fs::create_dir_all(&config_dir_path).unwrap();
        }
        let xray_config_path = config_dir_path.join(xray_config_file);

        XrayFileCore { xray_config_path }
    }

    fn write_xray_section<T: Serialize>(
        &self,
        key: &str,
        data: &Vec<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.xray_config_path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut json_root: Value = if content.is_empty() {
            json!({})
        } else {
            serde_json::from_str(&content).unwrap_or(json!({}))
        };

        let new_value: Value = serde_json::to_value(data)?;

        if let Some(obj) = json_root.as_object_mut() {
            obj.insert(key.to_string(), new_value);
        } else {
            return Err("JSON root is not a valid object".into());
        }

        let new_content = serde_json::to_string_pretty(&json_root)?;

        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(new_content.as_bytes())?;

        Ok(())
    }

    fn delete_xray_section(&self, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.xray_config_path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut json_root: Value = if content.is_empty() {
            json!({})
        } else {
            serde_json::from_str(&content).unwrap_or(json!({}))
        };

        if let Some(obj) = json_root.as_object_mut() {
            obj.remove(key);
        } else {
            return Err("JSON root is not a valid object".into());
        }

        let new_content = serde_json::to_string_pretty(&json_root)?;

        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(new_content.as_bytes())?;

        Ok(())
    }

    pub fn delete_xray_outbound_by_id(
        &self,
        outbound_id: &i32,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&self.xray_config_path)?;

        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut json_root: Value = if content.is_empty() {
            json!({})
        } else {
            serde_json::from_str(&content).unwrap_or(json!({}))
        };

        if let Some(outbounds_value) = json_root.get_mut("outbounds") {
            if let Some(outbounds_array) = outbounds_value.as_array_mut() {
                outbounds_array.retain(|outbound| {
                    if let Some(tag) = outbound.get("tag").and_then(|v| v.as_str()) {
                        if let Ok(id) = tag.parse::<i32>() {
                            id != *outbound_id
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                });
            }
        }

        let new_content = serde_json::to_string_pretty(&json_root)?;

        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(new_content.as_bytes())?;

        Ok(())
    }

    pub fn write_xray_inbounds(
        &self,
        inbounds: Vec<XrayInboundClientConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.write_xray_section("inbounds", &inbounds)
    }

    pub fn delete_xray_inbounds(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.delete_xray_section("inbounds")
    }

    pub fn write_xray_outbounds(
        &self,
        outbounds: Vec<XrayOutboundClientConfig>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.write_xray_section("outbounds", &outbounds)
    }

    pub fn delete_xray_outbounds(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.delete_xray_section("outbounds")
    }

    pub fn read_xray_inbounds(
        &self,
    ) -> Result<Vec<XrayInboundClientConfig>, Box<dyn std::error::Error>> {
        if !self.xray_config_path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.xray_config_path)?;
        let xray_settings_file: PartialXraySettingsFile = serde_json::from_reader(file)?;
        Ok(xray_settings_file.inbounds.unwrap_or_default())
    }

    pub fn read_xray_outbounds(
        &self,
    ) -> Result<Vec<XrayOutboundClientConfig>, Box<dyn std::error::Error>> {
        if !self.xray_config_path.exists() {
            return Ok(Vec::new());
        }
        let file = File::open(&self.xray_config_path)?;
        let xray_settings_file: PartialXraySettingsFile = serde_json::from_reader(file)?;
        Ok(xray_settings_file.outbounds.unwrap_or_default())
    }

    pub fn read_xray_file(&self) -> Result<Value, Box<dyn std::error::Error>> {
        if !self.xray_config_path.exists() {
            return Ok(json!({}));
        }
        let file = File::open(&self.xray_config_path)?;
        let xray_settings_file: Value = serde_json::from_reader(file)?;
        Ok(xray_settings_file)
    }
}
