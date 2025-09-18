use anyhow::Context;
use dirs::config_dir;
use luxnulla::{CONFIG_DIR, XRAY_CHECKER_CONFIG_FILE};
use std::sync::Mutex;
use tokio::process::{Child, Command};

use crate::{
    http::services::model::xray_config::XrayOutboundClientConfig,
    services::xray::file::{XrayFileCore, XrayInboundClientConfig, XrayInboundSettings},
};

static XRAY_CHILD: Mutex<Option<Child>> = Mutex::new(None);

pub fn ping(payload: Vec<XrayOutboundClientConfig>) {
    // spawn_xray();

    craft_xray_checker_file(payload);
}

fn craft_xray_checker_file(mut configs: Vec<XrayOutboundClientConfig>) {
    let xray_config = XrayFileCore::new(XRAY_CHECKER_CONFIG_FILE);

    configs.iter_mut().enumerate().for_each(|(idx, el)| {
        el.tag = Some(format!("test-outbound-{}", idx.to_string()));
    });

    let outbounds = configs.clone();

    xray_config
        .write_xray_outbounds(outbounds)
        .expect("Failed to write Xray checker outbounds config");

    let inbound = XrayInboundClientConfig {
        tag: "inbound".to_string(),
        listen: "127.0.0.1".to_string(),
        protocol: "socks".to_string(),
        port: 2050,
        settings: XrayInboundSettings {
            auth: "noauth".to_string(),
            udp: true,
        },
    };

    xray_config
        .write_xray_inbounds(vec![inbound])
        .expect("Failed to write Xray checker inbounds config");
}

fn spawn_xray() -> Result<(), anyhow::Error> {
    let config_path = config_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get config directory"))?
        .join(CONFIG_DIR)
        .join(XRAY_CHECKER_CONFIG_FILE);

    let child = Command::new("xray")
        .args(["run", "-c", config_path.to_str().context("Invalid path")?])
        .spawn()
        .context("Failed to spawn Xray command")?;

    *XRAY_CHILD.lock().unwrap() = Some(child);
    Ok(())
}

fn check_tags_exist(payload: &Vec<XrayOutboundClientConfig>) -> Result<(), &'static str> {
    if payload.iter().any(|config| {
        config.tag.is_none() || (config.tag.is_some() && config.tag.as_ref().unwrap().is_empty())
    }) {
        Err("One or more elements in the payload are missing a 'tag' or have an empty 'tag'.")
    } else {
        Ok(())
    }
}
