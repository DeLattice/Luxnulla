use serde_json::from_str;

use crate::{
    http::models::xray_config::{
        ExtraOutboundClientConfig, XrayOutboundClientConfig, XrayOutboundClientConfigModel,
    },
    services::repository::config::ConfigModel,
};

pub fn config_model_to_xray_outbound(
    config: ConfigModel,
) -> Result<XrayOutboundClientConfigModel, serde_json::Error> {
    let xray_config: XrayOutboundClientConfig = from_str(&config.data)?;

    let extra: Option<ExtraOutboundClientConfig> = if config.extra.is_empty() {
        None
    } else {
        Some(from_str(&config.extra)?)
    };

    Ok(XrayOutboundClientConfigModel {
        id: config.id,
        extra,
        config: xray_config,
    })
}

pub fn config_models_to_xray_outbounds(
    configs: Vec<ConfigModel>,
) -> Result<Vec<XrayOutboundClientConfigModel>, serde_json::Error> {
    configs
        .into_iter()
        .map(config_model_to_xray_outbound)
        .collect()
}

pub fn xray_outbound_to_config_model(
    xray: &XrayOutboundClientConfigModel,
    group_id: i32,
) -> Result<ConfigModel, serde_json::Error> {
    let data = serde_json::to_string(&xray.config)?;
    let extra = if let Some(extra_config) = &xray.extra {
        serde_json::to_string(extra_config)?
    } else {
        String::new()
    };

    Ok(ConfigModel {
        id: xray.id,
        group_id,
        data,
        extra,
    })
}

pub fn xray_outbounds_to_config_models(
    configs: &[XrayOutboundClientConfigModel],
    group_id: i32,
) -> Result<Vec<ConfigModel>, serde_json::Error> {
    configs
        .into_iter()
        .map(|config| xray_outbound_to_config_model(config, group_id))
        .collect()
}
