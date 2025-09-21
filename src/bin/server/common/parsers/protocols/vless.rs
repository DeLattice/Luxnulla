use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    common::parsers::outbound::{ClientConfigCommon, ParseError, Parser},
    http::services::model::xray_config::{GRPCSettings, RealitySettings, TlsSettings, User},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Vless {
    user: User,
    address: String,
    port: u16,
    transport: String,
    name_client: Option<String>,
    security: Option<String>,
    path: Option<String>,
    host: Option<String>,
    reality: Option<RealitySettings>,
    grpc: Option<GRPCSettings>,
    tls: Option<TlsSettings>,
}

impl ClientConfigCommon for Vless {
    fn address(&self) -> &str {
        &self.address
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn protocol(&self) -> &'static str {
        "vless"
    }
}

pub trait VlessClientConfigAccessor {
    fn user(&self) -> Option<&User>;
    fn name(&self) -> Option<&str>;
    fn security(&self) -> Option<&str>;
    fn transport(&self) -> Option<&str>;
    fn reality_settings(&self) -> Option<&RealitySettings>;
    fn grpc_settings(&self) -> Option<&GRPCSettings>;
    fn tls_settings(&self) -> Option<&TlsSettings>;
}

impl VlessClientConfigAccessor for Vless {
    fn user(&self) -> Option<&User> {
        Some(&self.user)
    }

    fn name(&self) -> Option<&str> {
        self.name_client.as_deref()
    }

    fn security(&self) -> Option<&str> {
        self.security.as_deref()
    }

    fn transport(&self) -> Option<&str> {
        Some(&self.transport)
    }

    fn reality_settings(&self) -> Option<&RealitySettings> {
        self.reality.as_ref()
    }

    fn grpc_settings(&self) -> Option<&GRPCSettings> {
        self.grpc.as_ref()
    }

    fn tls_settings(&self) -> Option<&TlsSettings> {
        self.tls.as_ref()
    }
}

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

        let transport = query
            .get("type")
            .ok_or(ParseError::FieldMissing("type".to_string()))?
            .to_string();

        let name_client = url.fragment().map(|s| s.to_string());

        let flow = query.get("flow").map(|s| s.to_string());

        let encryption = query
            .get("encryption")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_string());

        let grpc_settings = match query.get("mode") {
            Some(multi_mode) => {
                let multi_mode = multi_mode == "multi";

                Some(GRPCSettings {
                    service_name: query
                        .get("serviceName")
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string()),
                    multi_mode: multi_mode,
                    idle_timeout: Some(60),
                    health_check_timeout: Some(20),
                    permit_without_stream: Some(true),
                    initial_windows_size: Some(35536),
                })
            }
            None => None,
        };

        let fingerprint = query
            .get("fp")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "chrome".to_string());

        let spx = query.get("spx").map(|s| s.to_string());

        let reality_settings = if let (Some(pbk), Some(sni), Some(sid)) =
            (query.get("pbk"), query.get("sni"), query.get("sid"))
        {
            Some(RealitySettings {
                fingerprint: Some(fingerprint.clone()),
                spider_x: spx,
                public_key: pbk.to_string(),
                server_name: sni.to_string(),
                short_id: sid.to_string(),
            })
        } else {
            None
        };

        let tls = if let Some(sni) = query.get("sni") {
            Some(TlsSettings {
                fingerprint: Some(fingerprint.clone()),
                server_name: Some(sni.to_string()),
                verify_peer_cert_in_names: None,
                reject_unknown_sni: None,
                allow_insecure: None,
                alpn: None,
                min_version: None,
                max_version: None,
                cipher_suites: None,
                certificates: None,
                disable_system_root: None,
                enable_session_resumption: None,
                pinned_peer_certificate_chain_sha256: None,
                curve_preferences: None,
                master_key_log: None,
                ech_config_list: None,
                ech_server_keys: None,
                ech_force_query: None,
            })
        } else {
            None
        };

        let config = Vless {
            user: User {
                id: user_id,
                flow,
                encryption,
            },
            address,
            port,
            transport,
            name_client,
            security: query.get("security").cloned(),
            path: query.get("path").cloned(),
            host: query.get("host").cloned(),
            reality: reality_settings,
            grpc: grpc_settings,
            tls,
        };

        Ok(config)
    }
}
