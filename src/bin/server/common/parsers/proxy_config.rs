use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

use crate::http::services::model::xray_config::{RealitySettings, User};

#[derive(Debug)]
enum ParseError {
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

trait Parser
where
    Self: Sized,
{
    fn parse(url: &Url) -> Result<Self, ParseError>;
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ProxyConfig {
    Vmess(Vmess),
    Vless(Vless),
    Shadowsocks(Shadowsocks),
    Trojan(Trojan),
}

impl ProxyConfig {
    pub fn user(&self) -> Option<&User> {
        match self {
            ProxyConfig::Vless(value) => Some(&value.user),
            _ => None,
        }
    }

    pub fn address(&self) -> &str {
        match self {
            ProxyConfig::Vless(value) => &value.address,
            ProxyConfig::Vmess(value) => &value.address,
            ProxyConfig::Trojan(value) => &value.address,
            ProxyConfig::Shadowsocks(value) => &value.address,
        }
    }

    pub fn port(&self) -> u16 {
        match self {
            ProxyConfig::Vless(value) => value.port,
            ProxyConfig::Vmess(value) => value.port,
            ProxyConfig::Trojan(value) => value.port,
            ProxyConfig::Shadowsocks(value) => value.port,
        }
    }

    pub fn protocol(&self) -> &'static str {
        match self {
            ProxyConfig::Vless(_) => "vless",
            ProxyConfig::Vmess(_) => "vmess",
            ProxyConfig::Trojan(_) => "trojan",
            ProxyConfig::Shadowsocks(_) => "ss",
        }
    }

    pub fn name(&self) -> Option<&str> {
        match self {
            ProxyConfig::Vless(value) => value.name_client.as_deref(),
            ProxyConfig::Vmess(value) => value.name.as_deref(),
            ProxyConfig::Trojan(value) => value.name.as_deref(),
            ProxyConfig::Shadowsocks(value) => value.name.as_deref(),
        }
    }

    pub fn security(&self) -> Option<&str> {
        match self {
            ProxyConfig::Vless(value) => value.security.as_deref(),
            _ => None,
        }
    }

    pub fn network(&self) -> Option<&str> {
        match self {
            ProxyConfig::Vless(value) => Some(&value.network),
            ProxyConfig::Vmess(value) => Some(&value.network),
            _ => None,
        }
    }

    pub fn path(&self) -> Option<&str> {
        match self {
            ProxyConfig::Vless(value) => value.path.as_deref(),
            ProxyConfig::Vmess(value) => value.path.as_deref(),
            _ => None,
        }
    }

    pub fn host(&self) -> Option<&str> {
        match self {
            ProxyConfig::Vless(value) => value.host.as_deref(),
            ProxyConfig::Vmess(value) => value.host.as_deref(),
            ProxyConfig::Trojan(value) => value.host.as_deref(),
            _ => None,
        }
    }

    pub fn reality_settings(&self) -> Option<&RealitySettings> {
        match self {
            ProxyConfig::Vless(value) => value.reality.as_ref(),
            _ => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Vless {
    user: User,
    address: String,
    port: u16,
    network: String,
    name_client: Option<String>,
    security: Option<String>,
    path: Option<String>,
    host: Option<String>,
    reality: Option<RealitySettings>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Vmess {
    user_id: String,
    address: String,
    port: u16,
    aid: u32,
    network: String,
    type_field: Option<String>,
    host: Option<String>,
    path: Option<String>,
    name: Option<String>,
    name_client: Option<String>,
    // raw parameters store
    extras: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Shadowsocks {
    method: String,
    password: String,
    address: String,
    port: u16,
    name: Option<String>,
    extras: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Trojan {
    user_id: String,
    password: String,
    address: String,
    port: u16,
    sni: Option<String>,
    ws_path: Option<String>,
    host: Option<String>,
    allow_insecure: bool,
    name: Option<String>,
    extras: HashMap<String, String>,
}

// example vless_grpc config
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

impl Parser for Vless {
    fn parse(url: &Url) -> Result<Self, ParseError> {
        let query: HashMap<_, _> = url.query_pairs().into_owned().collect();

        let user_id = url.username().to_string();
        if user_id.is_empty() {
            return Err(ParseError::FieldMissing("user_id".to_string()));
        }

        let address = url
            .host_str()
            .ok_or(ParseError::FieldMissing("address".to_string()))?
            .to_string();

        let port = url
            .port()
            .ok_or(ParseError::FieldMissing("port".to_string()))?;

        let network = query
            .get("type")
            .ok_or(ParseError::FieldMissing("type".to_string()))?
            .to_string();

        let name_client = url.fragment().map(|s| s.to_string());

        let encryption= query.get("encryption")
            .map(|s| s.to_string())
            .or_else(|| Some("none".to_string()));

        let reality_settings = if let (Some(pbk), Some(sni), Some(sid)) =
            (query.get("pbk"), query.get("sni"), query.get("sid"))
        {
            Some(RealitySettings {
                fingerprint: Some(
                    query
                        .get("fp")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "chrome".to_string()),
                ),
                public_key: pbk.to_string(),
                server_name: sni.to_string(),
                short_id: sid.to_string(),
            })
        } else {
            None
        };

        let config = Vless {
            user: User {
                id: Some(user_id),
                encryption: encryption,
            },
            address,
            port,
            network,
            name_client,
            security: query.get("security").cloned(),
            path: query.get("path").cloned(),
            host: query.get("host").cloned(),
            reality: reality_settings,
        };

        Ok(config)
    }
}

pub fn decode_config_from_base64(payload: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
    return line.starts_with("vless") || line.starts_with("vmess") || line.starts_with("ss") || line.starts_with("trojan");
}

fn parse_line(url: Url) -> Result<ProxyConfig, String> {
    match url.scheme() {
        "vless" => Vless::parse(&url)
            .map(ProxyConfig::Vless)
            .map_err(|err| format!("{}", err)),
        other => Err(format!("unknown url scheme: \"{other}\"")),
    }
}

pub fn work(payload: &str) -> Result<Vec<ProxyConfig>, ()> {
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
