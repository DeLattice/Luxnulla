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

use crate::{
    http::{
        common::groups::process_config, services::model::xray_config::XrayOutboundClientConfig,
    },
    services::{StorageService, XrayOutboundModel, common::paginator::PaginationParams},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ReqCreateGroup {
    name: String,
    configs: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ResCreateGroup {
    pub id: i32,
    pub name: Option<String>,
    pub configs: Vec<XrayOutboundModel>,
}

pub async fn create_group(
    State(storage): State<Arc<StorageService>>,
    Json(req): Json<ReqCreateGroup>,
) -> impl IntoResponse {
    let configs = stream::iter(req.configs)
        .then(async |raw| process_config(&raw).await)
        .filter_map(|maybe| future::ready(maybe.ok()))
        .flat_map(|v| stream::iter(v))
        .collect::<Vec<_>>()
        .await;

    match storage.create_group(&req.name, configs) {
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

#[derive(Deserialize, Serialize, Debug)]
pub struct UpdateGroup {
    name: String,
    configs: Vec<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateGroupResponse {
    name: String,
    configs: Vec<XrayOutboundClientConfig>,
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
    match storage.delete_group(&id) {
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
