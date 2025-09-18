
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{common::parsers::outbound::{ClientConfigAccessor, ParseError, Parser}, http::services::model::xray_config::{GRPCSettings, RealitySettings, User}};

#[derive(Debug, Deserialize, Serialize)]
pub struct Vless {
    user: User,
    address: String,
    port: u16,
    network: String,
    name_client: Option<String>,
    security: Option<String>,
    path: Option<String>,
    host: Option<String>,
    reality: Option<RealitySettings>,
    grpc: Option<GRPCSettings>,
}

impl ClientConfigAccessor for Vless {
    fn user(&self) -> Option<&User> {
        Some(&self.user)
    }

    fn address(&self) -> &str {
        &self.address
    }

    fn port(&self) -> u16 {
        self.port
    }

    fn protocol(&self) -> &'static str {
        "vless"
    }

    fn name(&self) -> Option<&str> {
        self.name_client.as_deref()
    }

    fn security(&self) -> Option<&str> {
        self.security.as_deref()
    }

    fn network(&self) -> Option<&str> {
        Some(&self.network)
    }

    fn reality_settings(&self) -> Option<&RealitySettings> {
        self.reality.as_ref()
    }

    fn grpc_settings(&self) -> Option<&GRPCSettings> {
        self.grpc.as_ref()
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

        let network = query
            .get("type")
            .ok_or(ParseError::FieldMissing("type".to_string()))?
            .to_string();

        let name_client = url.fragment().map(|s| s.to_string());

        let flow = query.get("flow").map(|s| s.to_string());

        let encryption = query
            .get("encryption")
            .map(|s| s.to_string())
            .unwrap_or_else(|| "none".to_string());

        let multi_mode = query.get("mode").map_or(true, |s| s.to_string() != "gun");

        //dev
        let grpc_settings = Some(GRPCSettings {
            service_name: query
                .get("serviceName")
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string()),
            multi_mode: multi_mode,
            idle_timeout: Some(60),
            health_check_timeout: Some(20),
            permit_without_stream: Some(true),
            initial_windows_size: Some(35536),
        });

        let reality_settings = if let (Some(pbk), Some(sni), Some(sid)) =
            (query.get("pbk"), query.get("sni"), query.get("sid"))
        {
            let fingerprint = query
                .get("fp")
                .map(|s| s.to_string())
                .unwrap_or_else(|| "chrome".to_string());

            let spx = query.get("spx").map(|s| s.to_string());

            Some(RealitySettings {
                fingerprint: Some(fingerprint),
                spider_x: spx,
                public_key: pbk.to_string(),
                server_name: sni.to_string(),
                short_id: sid.to_string(),
            })
        } else {
            None
        };

        let config = Vless {
            user: User {
                id: user_id,
                flow: flow,
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
            grpc: grpc_settings,
        };

        Ok(config)
    }
}
