use crate::{
    common::parsers::proxy_config::{self, ProxyConfig}, http::services::model::xray_config::XrayClientOutboundConfig, services::{xray::fetcher::get_configs, Group, StorageService}
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use base64::{Engine, prelude::BASE64_STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::sync::Arc;
use url::Url;

pub enum ConfigType {
    RAW,
    BASE64,
    URL,
}

fn determine_config_type(config: &str) -> Result<ConfigType, std::io::Error> {
    if proxy_config::is_supported_scheme(config) {
        Ok(ConfigType::RAW)
    } else if config.trim().starts_with("http") || config.trim().starts_with("https") {
        Ok(ConfigType::URL)
    } else if BASE64_STANDARD.decode(config.trim()).is_ok() {
        Ok(ConfigType::BASE64)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid config",
        ))
    }
}

async fn process_config(payload: &str) -> Result<Vec<XrayClientOutboundConfig>, std::io::Error> {
    match determine_config_type(payload)? {
        ConfigType::RAW => {
            if let Ok(_) = Url::parse(&payload) {
                match proxy_config::work(&payload) {
                    Ok(configs) => {
                        let configs = configs
                            .iter()
                            .map(|config| XrayClientOutboundConfig::new(config))
                            .collect::<Vec<_>>();

                        Ok(configs)
                    },
                    Err(_) => Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to perform work with config",
                    )),
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid URL",
                ))
            }
        }
        ConfigType::BASE64 => {
            let raw_config = proxy_config::decode_config_from_base64(payload);

            if let Ok(config) = raw_config {
                if let Ok(_) = Url::parse(&config) {
                    match proxy_config::work(&config) {
                        Ok(configs) => {
                            let configs = configs
                                .iter()
                                .map(|config| XrayClientOutboundConfig::new(config))
                                .collect::<Vec<_>>();

                            Ok(configs)
                        },
                        Err(_) => Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to perform work with config",
                        )),
                    }
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid URL",
                    ))
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid config",
                ))
            }
        }
        ConfigType::URL => match get_configs(payload).await {
            Ok(configs) => {
                let configs = configs
                    .iter()
                    .map(|config| XrayClientOutboundConfig::new(config))
                    .collect::<Vec<_>>();

                Ok(configs)
            },
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to fetch configs",
            )),
        },
    }
}

#[derive(Deserialize)]
pub struct CreateGroup {
    name: String,
    payload: String,
}

#[derive(Deserialize, Serialize)]
pub struct CreateGroupResponse {
    name: String,
    configs: Value,
}

//todo add if group is exist
#[axum::debug_handler]
pub async fn create_group(
    State(storage): State<Arc<StorageService>>,
    Json(req): Json<CreateGroup>,
) -> impl IntoResponse {
    let decoded_config = match process_config(&req.payload).await {
        Ok(config) => config,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "Invalid config format",
                    "details": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let group = Group::new(req.name.clone(), json!(decoded_config));

    match storage.store_group(group) {
        Ok(()) => (
            StatusCode::CREATED,
            Json(CreateGroupResponse {
                name: req.name,
                configs: json!(decoded_config),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to save group",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn get_groups(State(storage): State<Arc<StorageService>>) -> impl IntoResponse {
    match storage.get_all_groups() {
        Ok(groups) => (
            StatusCode::OK,
            Json(json!({
                "groups": groups,
                "count": groups.len()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve groups",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateGroup {
    name: String,
    payload: Value,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateGroupResponse {
    name: String,
    configs: Value,
}

#[axum::debug_handler]
pub async fn update_group(
    State(storage): State<Arc<StorageService>>,
    Json(req): Json<UpdateGroup>,
) -> impl IntoResponse {
    let new_configs = serde_json::to_value(req.payload).unwrap();

    let group = Group::new(req.name.clone(), new_configs.clone());

    match storage.update_group_config(group) {
        Ok(_) => (
            StatusCode::OK,
            Json(UpdateGroupResponse {
                name: req.name,
                configs: new_configs,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve groups",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

//todo add if group is exist
#[axum::debug_handler]
pub async fn delete_group(
    State(storage): State<Arc<StorageService>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match storage.delete_group(&name) {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve groups",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn get_group_by_name(
    State(storage): State<Arc<StorageService>>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    match storage.get_group(&name) {
        Ok(group) => (StatusCode::OK, Json(json!(group))).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve groups",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn delete_all_groups(
    State(storage): State<Arc<StorageService>>
) -> impl IntoResponse {
    match storage.delete_all_group() {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve groups",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}
