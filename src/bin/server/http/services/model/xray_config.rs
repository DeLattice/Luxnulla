use serde::{Deserialize, Serialize};

use crate::common::parsers::{
    outbound::{ClientConfigCommon, ExtraOutboundClientConfig, OutboundClientConfig},
    protocols::{ss::ShadowsocksClientConfigAccessor, vless::VlessClientConfigAccessor},
};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TlsSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub verify_peer_cert_in_names: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reject_unknown_sni: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_insecure: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpn: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cipher_suites: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificates: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_system_root: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_session_resumption: Option<bool>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned_peer_certificate_chain_sha256: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub curve_preferences: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub master_key_log: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ech_config_list: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ech_server_keys: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ech_force_query: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RealitySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,

    pub public_key: String,
    pub server_name: String,
    pub short_id: String,

    #[serde(skip_serializing_if = "Option::is_none")]
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShadowsocksServer {
    address: String,
    port: u16,
    method: String,
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vnext: Option<Vec<VNext>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub servers: Option<Vec<ShadowsocksServer>>,
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

    #[serde(
        skip_serializing_if = "Option::is_none",
        rename(serialize = "tlsSettings", deserialize = "tlsSettings")
    )]
    pub tls: Option<TlsSettings>,
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

    #[serde(skip)]
    pub extra: Option<ExtraOutboundClientConfig>,
}

impl XrayOutboundClientConfig {
    pub fn new(config: &OutboundClientConfig) -> Self {
        XrayOutboundClientConfig {
            extra: Some(config.extra().clone()),
            tag: None,
            mux: None,
            protocol: config.protocol().to_string(),
            settings: Settings {
                servers: match config {
                    OutboundClientConfig::Shadowsocks(ss_config) => Some(vec![ShadowsocksServer {
                        address: ss_config.address().to_string(),
                        port: ss_config.port(),
                        method: ss_config.method().to_string(),
                        password: ss_config.password().to_string(),
                    }]),
                    _ => None,
                },
                vnext: match config {
                    OutboundClientConfig::Vless(vless_config) => Some(vec![VNext {
                        address: vless_config.address().to_string(),
                        port: vless_config.port(),
                        users: vec![vless_config.user().unwrap().clone()],
                    }]),
                    _ => None,
                },
            },
            stream_settings: StreamSettings {
                grpc: match config {
                    OutboundClientConfig::Vless(vless_config) => {
                        vless_config.grpc_settings().cloned()
                    }
                    _ => None,
                },
                reality: match config {
                    OutboundClientConfig::Vless(vless_config) => {
                        vless_config.reality_settings().cloned()
                    }
                    _ => None,
                },
                tls: match config {
                    OutboundClientConfig::Vless(vless_config) => {
                        vless_config.tls_settings().cloned()
                    }
                    _ => None,
                },
                network: match config {
                    OutboundClientConfig::Vless(vless_config) => {
                        vless_config.network().map(|e| e.to_string())
                    }
                    OutboundClientConfig::Shadowsocks(ss_config) => {
                        ss_config.network().map(|e| e.to_string())
                    }
                    _ => None,
                },
                security: match config {
                    OutboundClientConfig::Vless(vless_config) => {
                        vless_config.security().map(|e| e.to_string())
                    }
                    _ => None,
                },
            },
        }
    }

    pub fn extra(&self) -> Option<ExtraOutboundClientConfig> {
        self.extra.clone()
    }
}
