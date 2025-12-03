use serde_json::{Value, json};
use std::{fs::File, io::Write, path::PathBuf};

pub struct XrayFileCore {
    pub xray_config_path: PathBuf,
}

impl XrayFileCore {
    pub fn new(xray_config_path: PathBuf) -> Self {
        XrayFileCore { xray_config_path }
    }

    fn load_json(&self) -> Value {
        File::open(&self.xray_config_path)
            .ok()
            .and_then(|f| serde_json::from_reader(f).ok())
            .unwrap_or_else(|| json!({}))
    }

    pub fn read_with_json<F, R>(&self, reader: F) -> R
    where
        F: FnOnce(&Value) -> R,
    {
        let root = self.load_json();
        reader(&root)
    }

    fn save_json(&self, data: &Value) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(data)?;
        let mut file = File::create(&self.xray_config_path)?;

        file.write_all(content.as_bytes())?;
        file.sync_all()?;

        Ok(())
    }

    pub fn modify_json<F>(&self, modifier: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut Value),
    {
        let mut root = self.load_json();
        modifier(&mut root);
        self.save_json(&root)
    }

    pub fn write_full_config(&self, config: &Value) -> Result<(), Box<dyn std::error::Error>> {
        self.save_json(config)
    }
}
