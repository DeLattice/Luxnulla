use std::string::FromUtf8Error;

use base64::{Engine, prelude::BASE64_STANDARD};
use url::Url;

use crate::{
    common::parsers::protocols::{ss::Shadowsocks, trojan::Trojan, vless::Vless},
    http::models::xray_config::ExtraOutboundClientConfig,
};

#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    FieldMissing(String),
    Base64DecodeError(base64::DecodeError),
    Utf8Error(std::string::FromUtf8Error),
    UnknownFieldType { current: String, expected: String },
}
impl From<base64::DecodeError> for ParseError {
    fn from(err: base64::DecodeError) -> Self {
        ParseError::Base64DecodeError(err)
    }
}
impl From<String> for ParseError {
    fn from(err: String) -> Self {
        ParseError::FieldMissing(err)
    }
}
impl From<FromUtf8Error> for ParseError {
    fn from(err: FromUtf8Error) -> Self {
        ParseError::Utf8Error(err)
    }
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
            ParseError::Base64DecodeError(err) => write!(f, "Failed to decode base64: {}", err),
            ParseError::Utf8Error(err) => write!(f, "Failed to decode UTF-8: {}", err),
            ParseError::InvalidFormat(err) => write!(f, "Invalid format: {}", err),
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

pub enum OutboundClientConfig {
    Vless(Vless),
    Shadowsocks(Shadowsocks),
    Trojan(Trojan),
}

pub trait ClientConfigCommon {
    fn address(&self) -> &str;
    fn port(&self) -> u16;
    fn protocol(&self) -> &'static str;
    fn extra(&self) -> &ExtraOutboundClientConfig;
}

impl ClientConfigCommon for OutboundClientConfig {
    fn address(&self) -> &str {
        match self {
            OutboundClientConfig::Vless(vless_config) => vless_config.address(),
            OutboundClientConfig::Shadowsocks(shadowsocks_config) => shadowsocks_config.address(),
            OutboundClientConfig::Trojan(trojan_config) => trojan_config.address(),
        }
    }

    fn port(&self) -> u16 {
        match self {
            OutboundClientConfig::Vless(vless_config) => vless_config.port(),
            OutboundClientConfig::Shadowsocks(shadowsocks_config) => shadowsocks_config.port(),
            OutboundClientConfig::Trojan(trojan_config) => trojan_config.port(),
        }
    }

    fn protocol(&self) -> &'static str {
        match self {
            OutboundClientConfig::Vless(vless) => vless.protocol(),
            OutboundClientConfig::Shadowsocks(ss) => ss.protocol(),
            OutboundClientConfig::Trojan(trojan) => trojan.protocol(),
        }
    }

    fn extra(&self) -> &ExtraOutboundClientConfig {
        match self {
            OutboundClientConfig::Vless(vless) => vless.extra(),
            OutboundClientConfig::Shadowsocks(ss) => ss.extra(),
            OutboundClientConfig::Trojan(trojan) => trojan.extra(),
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
        Ok(decoded_bytes) => match String::from_utf8(decoded_bytes) {
            Ok(decoded_string) => decoded_string,
            Err(e) => {
                eprintln!("Failed to decode UTF-8 after Base64 decode: {}", e);
                return Err(Box::new(e));
            }
        },
        Err(e) => {
            eprintln!("INFO: Failed to decode Base64: {}", e);
            body.to_string()
        }
    };

    Ok(content)
}

pub fn is_supported_scheme(line: &str) -> bool {
    return line.starts_with("vless")
        || line.starts_with("vmess")
        || line.starts_with("ss")
        || line.starts_with("hysteria")
        || line.starts_with("trojan");
}

fn parse_line(url: Url) -> Result<OutboundClientConfig, String> {
    match url.scheme() {
        "vless" => Vless::parse(&url)
            .map(OutboundClientConfig::Vless)
            .map_err(|err| format!("{}", err)),
        "ss" => Shadowsocks::parse(&url)
            .map(OutboundClientConfig::Shadowsocks)
            .map_err(|err| format!("{}", err)),
        "trojan" => Trojan::parse(&url)
            .map(OutboundClientConfig::Trojan)
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
