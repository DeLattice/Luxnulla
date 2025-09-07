use luxnulla::{CONFIG_DIR, XRAY_CONFIG_FILE};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs::{File, OpenOptions},
    io::{Error, ErrorKind, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use crate::http::services::model::xray_config::XrayClientOutboundConfig;

pub fn update_outbounds(
    payload: &Vec<XrayClientOutboundConfig>,
) -> Result<Vec<XrayClientOutboundConfig>, Error> {
    check_tags_exist(payload).map_err(|err_msg| Error::new(ErrorKind::InvalidInput, err_msg))?;

    let xray_config = XrayFileCore::new();

    match xray_config.write_xray_outbounds(payload.clone()) {
        Ok(outbounds) => Ok(outbounds),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Failed to read Xray outbounds: {}", e),
        )),
    }
}

pub fn get_outbounds() -> Result<Vec<XrayClientOutboundConfig>, Box<dyn std::error::Error>>  {
    Ok(XrayFileCore::new().read_xray_outbounds()?)
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PartialXraySettingsFile {
    pub outbounds: Vec<XrayClientOutboundConfig>,
}

#[derive(Debug, Deserialize)]
struct XrayFileCore {
    // inbounds: todo!(),
    pub xray_config_path: PathBuf,
}

impl XrayFileCore {
    pub fn new() -> Self {
        let config_dir_path = dirs::config_dir().unwrap().join(CONFIG_DIR);

        let xray_config_path = config_dir_path.join(XRAY_CONFIG_FILE);

        XrayFileCore {
            xray_config_path: xray_config_path,
        }
    }

    pub fn write_xray_outbounds(
        &self,
        outbounds: Vec<XrayClientOutboundConfig>,
    ) -> Result<Vec<XrayClientOutboundConfig>, Box<dyn std::error::Error>> {
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
            serde_json::from_str(&content)?
        };

        let new_outbounds_value: Value = serde_json::to_value(&outbounds)?;

        if let Some(obj) = json_root.as_object_mut() {
            obj.insert("outbounds".to_string(), new_outbounds_value);
        } else {
            return Err("JSON root is not a valid object".into());
        }

        let new_content = serde_json::to_string_pretty(&json_root)?;

        file.set_len(0)?;
        file.seek(SeekFrom::Start(0))?;
        file.write_all(new_content.as_bytes())?;

        Ok(outbounds)
    }

    pub fn read_xray_outbounds(
        &self,
    ) -> Result<Vec<XrayClientOutboundConfig>, Box<dyn std::error::Error>> {
        let file = File::open(&self.xray_config_path)?;

        let xray_settings_file: PartialXraySettingsFile = serde_json::from_reader(file)?;

        Ok(xray_settings_file.outbounds)
    }
}

fn check_tags_exist(payload: &Vec<XrayClientOutboundConfig>) -> Result<(), &'static str> {
    if payload.iter().any(|config| {
        config.tag.is_none() || (config.tag.is_some() && config.tag.as_ref().unwrap().is_empty())
    }) {
        Err("One or more elements in the payload are missing a 'tag' or have an empty 'tag'.")
    } else {
        Ok(())
    }
}
