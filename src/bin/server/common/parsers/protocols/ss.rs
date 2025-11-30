use base64::{Engine, prelude::BASE64_STANDARD};
use percent_encoding;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    common::parsers::outbound::{ClientConfigCommon, ParseError, Parser},
    http::models::xray_config::ExtraOutboundClientConfig,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Shadowsocks {
    method: String,
    password: String,
    address: String,
    port: u16,
    extra: ExtraOutboundClientConfig,
}

pub trait ShadowsocksClientConfigAccessor {
    fn method(&self) -> &str;
    fn password(&self) -> &str;
    fn network(&self) -> Option<&str>;
}

impl ClientConfigCommon for Shadowsocks {
    fn address(&self) -> &str {
        &self.address
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn protocol(&self) -> &'static str {
        "shadowsocks"
    }

    fn extra(&self) -> &ExtraOutboundClientConfig {
        &self.extra
    }
}

impl ShadowsocksClientConfigAccessor for Shadowsocks {
    fn method(&self) -> &str {
        &self.method.as_ref()
    }

    fn password(&self) -> &str {
        &self.password.as_ref()
    }

    fn network(&self) -> Option<&str> {
        Some("tcp")
    }
}

impl Parser for Shadowsocks {
    fn parse(url: &Url) -> Result<Self, ParseError> {
        let address = url
            .host_str()
            .ok_or(ParseError::FieldMissing("address".to_string()))?
            .to_string();

        let port = url
            .port()
            .ok_or(ParseError::FieldMissing("port".to_string()))?;

        let creds = {
            let percent_decoded =
                percent_encoding::percent_decode_str(url.username()).collect::<Vec<u8>>();
            let base_decoded = BASE64_STANDARD.decode(percent_decoded)?;

            String::from_utf8(base_decoded)?
        };

        let (method, password) =
            parse_shadowsocks_creds(&creds).ok_or(ParseError::FieldMissing("port".to_string()))?;

        let client_name = url.fragment().map(|s| s.to_string());

        let extra = ExtraOutboundClientConfig { client_name };

        let config = Shadowsocks {
            method: method.to_string(),
            password: password.to_string(),
            address,
            port,
            extra,
        };

        Ok(config)
    }
}

// aes-128-gcm:gx5Kni2f6aXYjBfCEgSKnQ
// ==>
// method: aes-128-gcm | password: gx5Kni2f6aXYjBfCEgSKnQ
fn parse_shadowsocks_creds(decoded_string: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = decoded_string.splitn(2, ':').collect();

    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}
