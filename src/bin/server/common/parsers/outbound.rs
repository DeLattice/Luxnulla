use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::{common::parsers::protocols::vless::Vless, http::services::model::xray_config::{GRPCSettings, RealitySettings, User}};

#[derive(Debug)]
pub enum ParseError {
    FieldMissing(String),
    UnknownFieldType { current: String, expected: String },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::FieldMissing(field) => write!(f, "Missing field: {}", field),
            ParseError::UnknownFieldType { current, expected } => write!(
                f,
                "Unknown field type: {} (expected: {})",
                current, expected
            ),
        }
    }
}

impl std::error::Error for ParseError {}

pub trait Parser
where
    Self: Sized,
{
    fn parse(url: &Url) -> Result<Self, ParseError>;
}

pub trait ClientConfigAccessor {
    fn user(&self) -> Option<&User>;
    fn address(&self) -> &str;
    fn port(&self) -> u16;
    fn protocol(&self) -> &'static str;
    fn name(&self) -> Option<&str>;
    fn security(&self) -> Option<&str>;
    fn network(&self) -> Option<&str>;
    fn reality_settings(&self) -> Option<&RealitySettings>;
    fn grpc_settings(&self) -> Option<&GRPCSettings>;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum OutboundClientConfig {
    Vless(Vless),
}

impl ClientConfigAccessor for OutboundClientConfig {
    fn user(&self) -> Option<&User> {
        match self {
            OutboundClientConfig::Vless(vless) => vless.user(),
        }
    }

    fn address(&self) -> &str {
        match self {
            OutboundClientConfig::Vless(vless) => vless.address(),
        }
    }

    fn port(&self) -> u16 {
        match self {
            OutboundClientConfig::Vless(vless) => vless.port(),
        }
    }

    fn protocol(&self) -> &'static str {
        match self {
            OutboundClientConfig::Vless(vless) => vless.protocol(),
        }
    }

    fn name(&self) -> Option<&str> {
        match self {
            OutboundClientConfig::Vless(vless) => vless.name(),
        }
    }

    fn security(&self) -> Option<&str> {
        match self {
            OutboundClientConfig::Vless(vless) => vless.security(),
        }
    }

    fn network(&self) -> Option<&str> {
        match self {
            OutboundClientConfig::Vless(vless) => vless.network(),
        }
    }

    fn reality_settings(&self) -> Option<&RealitySettings> {
        match self {
            OutboundClientConfig::Vless(vless) => vless.reality_settings(),
        }
    }

    fn grpc_settings(&self) -> Option<&GRPCSettings> {
        match self {
            OutboundClientConfig::Vless(vless) => vless.grpc_settings(),
        }
    }
}


// example (reality vless grpc) config
// vless://
// d8737518-5251-4e25-a653-8c625ef18b8f
// @24.120.32.42:2040
// ?security=reality
// &type=grpc
// &sni=unpkg.com
// &sid=e0969a6f81b52865
// &pbk=FPIcpZmVrQcqkF1vR_aBnLw_Uu4CNhuuKkrRtKpzRHg
//
// <=== extra ===>
// &headerType=
// &serviceName=
// &authority=
// &mode=gun
// &fp=chrome
// #%F0%9F%9A%80%20Marz%20%28igni_laptop_grpc_reality_flow%29%20%5BVLESS%20-%20grpc%5D


pub fn decode_config_from_base64(
    payload: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let body = payload.trim();

    let content = match BASE64_STANDARD.decode(body) {
        Ok(decoded_bytes) => {
            println!("INFO: Content detected as Base64. Decoding...");
            String::from_utf8(decoded_bytes)?
        }
        Err(_) => {
            println!("INFO: Content detected as plain text.");
            body.to_string()
        }
    };

    Ok(content)
}

pub fn is_supported_scheme(line: &str) -> bool {
    return line.starts_with("vless")
        || line.starts_with("vmess")
        || line.starts_with("ss")
        || line.starts_with("trojan");
}

fn parse_line(url: Url) -> Result<OutboundClientConfig, String> {
    match url.scheme() {
        "vless" => Vless::parse(&url)
            .map(OutboundClientConfig::Vless)
            .map_err(|err| format!("{}", err)),
        other => Err(format!("unknown url scheme: \"{other}\"")),
    }
}

pub fn work(payload: &str) -> Result<Vec<OutboundClientConfig>, ()> {
    let mut configs = Vec::new();

    for line in payload.lines() {
        let Ok(url) = Url::parse(line) else {
            eprintln!("Is not valid url {}", line);

            continue;
        };

        match parse_line(url) {
            Ok(url) => configs.push(url),
            Err(err) => eprintln!("failed to parse line: {}", err),
        }
    }

    Ok(configs)
}
