use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RealitySettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    pub public_key: String,
    pub server_name: String,
    pub short_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GRPCSettings {
    pub service_name: String,
    pub multi_mode: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub address: String,
    pub port: u16,
    pub users: Vec<User>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encryption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StreamSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    // #[serde(skip_serializing_if = "Option::is_none")]
    // grpc_settings: Option<GRPCSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reality: Option<RealitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MuxSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub concurrency: Option<RealitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xudp_concurrency: Option<RealitySettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xudp_proxy_udp443: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XrayClientOutboundConfig {
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
