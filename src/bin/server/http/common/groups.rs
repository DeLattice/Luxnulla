use base64::{Engine, prelude::BASE64_STANDARD};
use url::Url;

use crate::{common::parsers::outbound, http::services::model::xray_config::XrayOutboundClientConfig, services::xray::fetcher::get_configs};

pub enum ConfigType {
    RAW,
    BASE64,
    URL,
}

pub fn determine_config_type(config: &str) -> Result<ConfigType, std::io::Error> {
    if outbound::is_supported_scheme(config) {
        Ok(ConfigType::RAW)
    } else if config.trim().starts_with("http") || config.trim().starts_with("https") {
        Ok(ConfigType::URL)
    } else if BASE64_STANDARD.decode(config.trim()).is_ok() {
        Ok(ConfigType::BASE64)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid config",
        ))
    }
}

pub async fn process_config(payload: &str) -> Result<Vec<XrayOutboundClientConfig>, std::io::Error> {
    match determine_config_type(payload)? {
        ConfigType::RAW => {
            if let Ok(_) = Url::parse(&payload) {
                match outbound::work(&payload) {
                    Ok(configs) => {
                        let configs = configs
                            .iter()
                            .map(|config| XrayOutboundClientConfig::new(config))
                            .collect::<Vec<_>>();

                        Ok(configs)
                    }
                    Err(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to perform work with config",
                    )),
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid URL",
                ))
            }
        }
        ConfigType::BASE64 => {
            let raw_config = outbound::decode_config_from_base64(payload);

            if let Ok(config) = raw_config {
                if let Ok(_) = Url::parse(&config) {
                    match outbound::work(&config) {
                        Ok(configs) => {
                            let configs = configs
                                .iter()
                                .map(|config| XrayOutboundClientConfig::new(config))
                                .collect::<Vec<_>>();

                            Ok(configs)
                        }
                        Err(_) => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to perform work with config",
                        )),
                    }
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid URL",
                    ))
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid config",
                ))
            }
        }
        ConfigType::URL => match get_configs(payload).await {
            Ok(configs) => {
                let configs = configs
                    .iter()
                    .map(|config| XrayOutboundClientConfig::new(config))
                    .collect::<Vec<_>>();

                Ok(configs)
            }
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch configs",
            )),
        },
    }
}
