use serde::{Deserialize, Serialize};

use crate::common::parsers::outbound::{OutboundClientConfig, ClientConfigAccessor};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RealitySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    #[serde(rename(serialize = "publicKey", deserialize = "publicKey"))]
    pub public_key: String,

    #[serde(rename(serialize = "serverName", deserialize = "serverName"))]
    pub server_name: String,

    #[serde(rename(serialize = "shortId", deserialize = "shortId"))]
    pub short_id: String,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "spiderX", deserialize = "spiderX")
    )]
    pub spider_x: Option<String>,
}

//Fucking bullshit. I can't understand a damn thing. It works even without serviceName using sing-box.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GRPCSettings {
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "serviceName", deserialize = "serviceName")
    )]
    pub service_name: Option<String>,

    #[serde(rename(serialize = "multiMode", deserialize = "multiMode"))]
    pub multi_mode: bool,

    pub idle_timeout: Option<u64>,
    pub health_check_timeout: Option<u64>,
    pub permit_without_stream: Option<bool>,
    pub initial_windows_size: Option<u64>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VNext {
    pub address: String,
    pub port: u16,
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub vnext: Vec<VNext>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: String,
    pub encryption: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub flow: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "grpcSettings", deserialize = "grpcSettings")
    )]
    pub grpc: Option<GRPCSettings>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "realitySettings", deserialize = "realitySettings")
    )]
    pub reality: Option<RealitySettings>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MuxSettings {
    pub enable: String,
    pub concurrency: Option<u32>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "xudpConcurrency", deserialize = "xudpConcurrency")
    )]
    pub xudp_concurrency: Option<u32>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "xudpProxyUDP443", deserialize = "xudpProxyUDP443")
    )]
    pub xudp_proxy_udp443: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XrayOutboundClientConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    pub protocol: String,
    pub settings: Settings,

    #[serde(rename(serialize = "streamSettings", deserialize = "streamSettings"))]
    pub stream_settings: StreamSettings,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub mux: Option<MuxSettings>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // pub name_client: Option<String>,
}

impl XrayOutboundClientConfig {
    pub fn new(config: &OutboundClientConfig) -> Self {
        XrayOutboundClientConfig {
            tag: None,
            mux: None,
            protocol: config.protocol().to_string(),
            settings: Settings {
                vnext: vec![VNext {
                    address: config.address().to_string(),
                    port: config.port(),
                    users: vec![config.user().unwrap().clone()],
                }],
            },
            stream_settings: StreamSettings {
                grpc: config.grpc_settings().cloned(),
                reality: config.reality_settings().cloned(),
                network: config.network().map(|network| network.to_string()),
                security: config.security().map(|security| security.to_string()),
            },
        }
    }
}
