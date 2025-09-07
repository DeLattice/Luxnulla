use crate::{
    common::parsers::proxy_config::ProxyConfig,
    http::services::model::xray_config::{
        Settings, StreamSettings, XrayClientOutboundConfig,
    },
};

impl XrayClientOutboundConfig {
    pub fn new(config: &ProxyConfig) -> Self {
        XrayClientOutboundConfig {
            tag: None,
            mux: None,
            protocol: config.protocol().to_string(),
            // name_client: config.name().map(|name| name.to_string()),
            settings: Settings {
                address: config.address().to_string(),
                port: config.port(),
                users: vec![config.user().unwrap().clone()],
            },
            stream_settings: StreamSettings {
                reality: config.reality_settings().cloned(),
                network: config.network().map(|network| network.to_string()),
                security: config.security().map(|security| security.to_string()),
            },
        }
    }
}
