use crate::{
    http::models::xray_config::XrayOutboundClientConfig,
    services::{
        common::convertors::config_models_to_xray_outbounds,
        repository::config::ConfigModel,
        xray::file::{core::XrayFileCore, sections::SectionOps},
    },
};
use std::io::{Error, ErrorKind};

fn to_io_err<E: ToString>(e: E) -> Error {
    Error::new(ErrorKind::Other, e.to_string())
}

pub fn get_outbounds(
    core: &XrayFileCore,
) -> Result<Vec<XrayOutboundClientConfig>, Box<dyn std::error::Error>> {
    core.get_section("outbounds")
}

pub fn update_outbounds(
    core: &XrayFileCore,
    models: &[ConfigModel],
) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    let configs: Vec<_> = config_models_to_xray_outbounds(models.to_vec())
        .map_err(to_io_err)?
        .into_iter()
        .map(|mut m| {
            m.config.tag = Some(m.id.to_string());
            m.config
        })
        .collect();

    core.set_section("outbounds", &configs).map_err(to_io_err)?;
    core.get_section("outbounds").map_err(to_io_err)
}

pub fn delete_outbounds(
    core: &XrayFileCore,
    ids: &[i32],
) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    core.modify_json(|root| {
        if let Some(outbounds) = root["outbounds"].as_array_mut() {
            outbounds.retain(|item| {
                let tag_id = item["tag"].as_str().and_then(|s| s.parse::<i32>().ok());

                tag_id.map_or(true, |id| !ids.contains(&id))
            });
        }
    })
    .map_err(to_io_err)?;

    core.get_section("outbounds").map_err(to_io_err)
}

pub fn delete_outbound(
    core: &XrayFileCore,
    id: i32,
) -> Result<Vec<XrayOutboundClientConfig>, Error> {
    delete_outbounds(core, &[id])
}
