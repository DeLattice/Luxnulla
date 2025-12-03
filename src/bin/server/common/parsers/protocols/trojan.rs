use serde::{Deserialize, Serialize};
use url::Url;

use crate::{
    common::parsers::outbound::{ClientConfigCommon, ParseError, Parser},
    http::models::xray_config::{ExtraOutboundClientConfig, TlsSettings},
};

#[derive(Debug, Deserialize, Serialize)]
pub struct Trojan {
    address: String,
    port: u16,
    password: String,
    flow: Option<String>,

    network: Option<String>,
    security: Option<String>,
    path: Option<String>,
    host: Option<String>,
    tls: Option<TlsSettings>,
    extra: ExtraOutboundClientConfig,
}

impl ClientConfigCommon for Trojan {
    fn address(&self) -> &str {
        &self.address
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn protocol(&self) -> &'static str {
        "trojan"
    }

    fn extra(&self) -> &ExtraOutboundClientConfig {
        &self.extra
    }
}

pub trait TrojanClientConfigAccessor {
    fn flow(&self) -> Option<&str>;
    fn password(&self) -> &str;
    fn network(&self) -> Option<&str>;
    fn security(&self) -> Option<&str>;
    fn path(&self) -> Option<&str>;
    fn host(&self) -> Option<&str>;
    fn tls(&self) -> Option<&TlsSettings>;
}

impl TrojanClientConfigAccessor for Trojan {
    fn flow(&self) -> Option<&str> {
        self.flow.as_deref()
    }

    fn password(&self) -> &str {
        &self.password
    }

    fn network(&self) -> Option<&str> {
        self.network.as_deref()
    }

    fn security(&self) -> Option<&str> {
        self.security.as_deref()
    }

    fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }

    fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    fn tls(&self) -> Option<&TlsSettings> {
        self.tls.as_ref()
    }
}

// trojan://e5e79b65-c5fd-4b06-8f56-6bf8c44a61d9@193.42.112.66:2053
// ?path=%2FFree-VPN-CF-Geo-Project%2F193.42.112.66%3D2053
// &security=tls
// &host=joss.gpj4.web.id
// &fp=randomized
// &type=ws&sni=joss.gpj4.web.id#%40v2FreeHub%20%F0%9F%87%AA%F0%9F%87%AA

impl Parser for Trojan {
    fn parse(url: &Url) -> Result<Self, ParseError> {
        let address = url
            .host_str()
            .ok_or_else(|| "address".to_string())?
            .to_string();
        let port = url.port().ok_or_else(|| "port".to_string())?;
        let password = url.username().to_string();
        let extra = ExtraOutboundClientConfig {
            client_name: url.fragment().map(|s| s.to_string()),
        };

        let mut flow = None;
        let mut security: Option<String> = None;
        let mut path: Option<String> = None;
        let mut host: Option<String> = None;
        let mut sni: Option<String> = None;
        let mut fingerprint: Option<String> = None;
        let mut network = Some("tcp".to_string());

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "flow" => flow = Some(value.to_string()),
                "type" => network = Some(value.to_string()),
                "security" => security = Some(value.to_string()),
                "path" => path = Some(value.to_string()),
                "host" => host = Some(value.to_string()),
                "sni" => sni = Some(value.to_string()),
                "fp" => fingerprint = Some(value.to_string()),
                _ => {}
            }
        }

        let tls = if security.as_deref() == Some("tls") {
            Some(TlsSettings {
                server_name: sni,
                fingerprint,
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

        Ok(Trojan {
            address,
            port,
            password,
            flow,
            network,
            security,
            path,
            host,
            tls,
            extra,
        })
    }
}
