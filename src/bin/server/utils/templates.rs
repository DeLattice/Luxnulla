use rust_embed::RustEmbed;
use serde_json::{json, Value};
use std::{path::PathBuf,};

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Assets;

pub fn get_init_xray_config(log_path: &PathBuf) -> String {
    let file = Assets::get("xray.json").expect("xray.json missing");
    let content = std::str::from_utf8(file.data.as_ref()).expect("Invalid UTF-8");

    let mut config: Value = serde_json::from_str(content).expect("Invalid JSON");

    if let Some(log) = config.get_mut("log") {
        log["access"] = json!(log_path);
        log["error"] = json!(log_path);
    }

    serde_json::to_string_pretty(&config).expect("Serialization failed")
}

pub fn get_nft_config() -> String {
    let file = Assets::get("proxy.conf").expect("nftables.conf missing");
    let content = std::str::from_utf8(file.data.as_ref()).expect("Invalid UTF-8");

    content.replace("PORT_PLACEHOLDER", "1080")
}
