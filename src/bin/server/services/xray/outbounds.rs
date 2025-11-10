use crate::{
    http::services::model::xray_config::XrayOutboundClientConfig,
    services::storage::StorageService, services::xray::file::XrayFileCore,
};
use luxnulla::XRAY_CONFIG_FILE;
use std::io::{Error, ErrorKind};

pub fn update_outbounds(config_ids: &Vec<i32>) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    let configs = StorageService::new()
        .get_configs_by_ids(config_ids)
        .unwrap();

    let xray_config = XrayFileCore::new(XRAY_CONFIG_FILE);

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

pub fn get_outbounds() -> Result<Vec<XrayOutboundClientConfig>, Box<dyn std::error::Error>> {
    Ok(XrayFileCore::new(XRAY_CONFIG_FILE).read_xray_outbounds()?)
}
