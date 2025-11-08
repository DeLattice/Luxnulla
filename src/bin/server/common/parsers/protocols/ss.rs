use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::common::parsers::outbound::{
    ClientConfigCommon, ExtraOutboundClientConfig, ParseError, Parser,
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Shadowsocks {
    method: String,
    password: String,
    address: String,
    port: u16,
    name: Option<String>,
    extra: ExtraOutboundClientConfig,
}

pub trait ShadowsocksClientConfigAccessor {
    fn method(&self) -> &str;
    fn password(&self) -> &str;
    fn transport(&self) -> Option<&str>;
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

    fn transport(&self) -> Option<&str> {
        Some("tcp")
    }
}
// ss://YWVzLTEyOC1nY206Z3g1S25pMmY2YVhZakJmQ0VnU0tuUQ==@37.27.184.130:1080#%F0%9F%9A%80%20Marz%20%28igni_desktop_grpc_reality_flow%29%20%5BShadowsocks%20-%20tcp%5D
impl Parser for Shadowsocks {
    fn parse(url: &Url) -> Result<Self, ParseError> {
        let address = url
            .host_str()
            .ok_or(ParseError::FieldMissing("address".to_string()))?
            .to_string();

        let port = url
            .port()
            .ok_or(ParseError::FieldMissing("port".to_string()))?;

        println!("username: {}", url.username());

        let encoded_data = url.username();
        let decoded_bytes = BASE64_STANDARD.decode(encoded_data)?;
        let decoded_data = String::from_utf8(decoded_bytes)?;

        let (method, password) = parse_shadowsocks_creds(&decoded_data)
            .ok_or(ParseError::FieldMissing("port".to_string()))?;

        let client_name = url.fragment().map(|s| s.to_string());

        let extra = ExtraOutboundClientConfig { client_name };

        let config = Shadowsocks {
            method: method.to_string(),
            password: password.to_string(),
            address,
            port,
            name: Some("My Shadowsocks Server".to_string()),
            extra,
        };

        Ok(config)
    }
}

fn parse_shadowsocks_creds(decoded_string: &str) -> Option<(&str, &str)> {
    let parts: Vec<&str> = decoded_string.splitn(2, ':').collect();
    if parts.len() == 2 {
        Some((parts[0], parts[1]))
    } else {
        None
    }
}
