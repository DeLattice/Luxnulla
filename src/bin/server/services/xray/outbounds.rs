use luxnulla::XRAY_CONFIG_FILE;
use std::io::{Error, ErrorKind};
use crate::{
    services::storage::StorageService,
    http::services::model::xray_config::XrayOutboundClientConfig,
    services::xray::file::XrayFileCore,
};

pub fn update_outbounds(
    config_ids: &Vec<i32>,
) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    // check_tags_exist(payload).map_err(|err_msg| Error::new(ErrorKind::InvalidInput, err_msg))?;

    let configs = StorageService::new().get_configs_by_ids(config_ids).unwrap();

    let xray_config = XrayFileCore::new(XRAY_CONFIG_FILE);

    match xray_config.write_xray_outbounds(configs.clone()) {
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

pub fn get_outbounds() -> Result<Vec<XrayOutboundClientConfig>, Box<dyn std::error::Error>> {
    Ok(XrayFileCore::new(XRAY_CONFIG_FILE).read_xray_outbounds()?)
}

fn check_tags_exist(configs: &Vec<XrayOutboundClientConfig>) -> Result<(), &'static str> {
    if configs.iter().any(|config| {
        config.tag.is_none() || (config.tag.is_some() && config.tag.as_ref().unwrap().is_empty())
    }) {
        Err("One or more elements in the payload are missing a 'tag' or have an empty 'tag'.")
    } else {
        Ok(())
    }
}
