use crate::{
    http::models::xray_config::XrayOutboundClientConfig,
    services::{
        common::convertors::config_models_to_xray_outbounds, repository::config::ConfigModel,
        xray::file::XrayFileCore,
    },
};
use elux::XRAY_CONFIG_FILE;
use std::io::{Error, ErrorKind};

pub fn get_outbounds() -> Result<Vec<XrayOutboundClientConfig>, Box<dyn std::error::Error>> {
    Ok(XrayFileCore::new(XRAY_CONFIG_FILE).read_xray_outbounds()?)
}

pub fn update_outbounds(
    configs_models: &[ConfigModel],
) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    let xray_config = XrayFileCore::new(XRAY_CONFIG_FILE);

    let configs = config_models_to_xray_outbounds(configs_models.to_vec())
        .unwrap()
        .iter()
        .map(|xray_config_model| {
            let mut config = xray_config_model.clone().config;

            config.tag = Some(xray_config_model.id.to_string());

            config
        })
        .collect::<Vec<_>>();

    match xray_config.write_xray_outbounds(configs) {
        Ok(_) => match xray_config.read_xray_outbounds() {
            Ok(outbound) => Ok(outbound),
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("Failed to read Xray outbounds: {}", e),
            )),
        },
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Failed to write Xray outbounds: {}", e),
        )),
    }
}
pub fn delete_outbounds(config_ids: &Vec<i32>) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    let xray_config = XrayFileCore::new(XRAY_CONFIG_FILE);

    for id in config_ids {
        match xray_config.delete_xray_outbound_by_id(&id) {
            Ok(_) => {}
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("Failed to delete Xray outbound with ID {}: {}", id, e),
                ));
            }
        }
    }

    match xray_config.read_xray_outbounds() {
        Ok(outbound) => Ok(outbound),
        Err(e) => Err(Error::new(
            ErrorKind::Other,
            format!("Failed to read Xray outbounds after deletion: {}", e),
        )),
    }
}
