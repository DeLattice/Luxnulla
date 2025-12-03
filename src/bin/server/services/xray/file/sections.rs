use serde::{Serialize, de::DeserializeOwned};

use crate::services::xray::file::core::XrayFileCore;

pub trait SectionOps {
    fn get_section<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>>;
    fn set_section<T: Serialize>(
        &self,
        key: &str,
        data: &Vec<T>,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

impl SectionOps for XrayFileCore {
    fn get_section<T: DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Vec<T>, Box<dyn std::error::Error>> {
        self.read_with_json(|root| {
            let section = root.get(key).cloned().unwrap_or(serde_json::json!([]));
            Ok(serde_json::from_value(section).unwrap_or_default())
        })
    }

    fn set_section<T: Serialize>(
        &self,
        key: &str,
        data: &Vec<T>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.modify_json(|root| {
            root[key] = serde_json::to_value(data).unwrap_or(serde_json::json!([]));
        })
    }
}
