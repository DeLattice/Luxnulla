use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use futures::{
    future,
    stream::{self, StreamExt},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use url::Url;

use crate::{
    http::{
        common::groups::process_config, services::model::xray_config::XrayOutboundClientConfig,
    },
    services::{
        StorageService, XrayOutboundModel,
        common::paginator::PaginationParams,
        xray::{self, outbounds::delete_outbounds},
    },
};

//todo make multi type | one for subsystem group and one for subsystem REST API
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroup {
    name: String,
    sub_url: Option<Url>,
    configs: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupResponse {
    pub id: i32,
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_url: Option<Url>,

    pub configs: Vec<XrayOutboundModel>,
}

pub async fn create_group(
    State(storage): State<Arc<StorageService>>,
    Json(req): Json<CreateGroup>,
) -> impl IntoResponse {
    let configs = stream::iter(req.configs)
        .then(async |raw| process_config(&raw).await)
        .filter_map(|maybe| future::ready(maybe.ok()))
        .flat_map(|v| stream::iter(v))
        .collect::<Vec<_>>()
        .await;

    match storage.create_group(&req.name, configs, req.sub_url) {
        Ok(group_configs) => (StatusCode::CREATED, Json(json!(group_configs))).into_response(),
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

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateGroup {
    name: String,
    configs: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateGroupResponse {
    name: String,
    configs: Vec<XrayOutboundModel>,
}

//todo rename group if get different (req != payload.name) name
#[axum::debug_handler]
pub async fn update_group(
    State(storage): State<Arc<StorageService>>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateGroup>,
) -> impl IntoResponse {
    // match storage.upsert_group(group) {
    //     Ok(_) => (
    //         StatusCode::OK,
    //         Json(UpdateGroupResponse {
    //             name: req.name,
    //             configs: req.payload,
    //         }),
    //     )
    //         .into_response(),
    //     Err(e) => (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!({
    //             "error": "Failed to retrieve groups",
    //             "details": e.to_string()
    //         })),
    //     )
    //         .into_response(),
    // }
    ()
}

#[axum::debug_handler]
pub async fn delete_group(
    State(storage): State<Arc<StorageService>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    let group = match storage.get_group(&id) {
        Ok(group) => group,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to retrieve group",
                    "details": e.to_string()
                })),
            )
                .into_response();
        }
    };

    let config_ids = group
        .configs
        .iter()
        .map(|config| config.id)
        .collect::<Vec<i32>>();

    match delete_outbounds(&config_ids) {
        Ok(_) => match storage.delete_group(&id) {
            Ok(_) => (StatusCode::OK).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to retrieve groups",
                    "details": e.to_string()
                })),
            )
                .into_response(),
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to delete outbounds",
                    "details": e.to_string()
                })),
            )
                .into_response();
        }
    }
}

#[axum::debug_handler]
pub async fn delete_all_groups(State(storage): State<Arc<StorageService>>) -> impl IntoResponse {
    match storage.delete_all_groups() {
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
pub async fn get_group(
    State(storage): State<Arc<StorageService>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match storage.get_group(&id) {
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
pub async fn refresh_group(
    State(storage): State<Arc<StorageService>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match storage.get_group(&id) {
        Ok(group) => {
            let sub_url = group
                .sub_url
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "Group has no sub_url")
                })
                .unwrap();

            match xray::fetcher::get_configs(sub_url.as_str()).await {
                Ok(configs) => {
                    let configs = configs
                        .into_iter()
                        .map(|config| XrayOutboundClientConfig::new(&config))
                        .collect::<Vec<_>>();

                    (StatusCode::OK, Json(json!(configs))).into_response()
                }
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({
                        "error": "Failed to fetch configs from sub_url",
                        "details": e.to_string()
                    })),
                )
                    .into_response(),
            }
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to retrieve group",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn get_list_group_names(State(storage): State<Arc<StorageService>>) -> impl IntoResponse {
    match storage.list_groups() {
        Ok(groups) => (StatusCode::OK, Json(json!(groups))).into_response(),
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

pub async fn get_paginated_group_configs(
    State(storage): State<Arc<StorageService>>,
    Path(id): Path<i32>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    match storage.get_paginated_group_configs(&id, &pagination) {
        Ok(data) => (StatusCode::OK, Json(json!(data.unwrap().configs))).into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "Failed to retrieve groups",
                "details": e.to_string()
            })),
        )
            .into_response(),
    }
}
