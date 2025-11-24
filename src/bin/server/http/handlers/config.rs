use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;

use crate::{
    http::server::AppState,
    services::{
        common::{convertors::config_models_to_xray_outbounds, paginator::PaginationParams},
        db::TransactionManager,
        repository::config::ConfigRepository,
    },
};

#[axum::debug_handler]
pub async fn get_configs_by_group_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        ConfigRepository::get_by_group_id(&tx, id)
    }) {
        Ok(data) => match config_models_to_xray_outbounds(data) {
            Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn get_paginated_configs_by_group_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Query(pagination): Query<PaginationParams>,
) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        ConfigRepository::get_by_group_id_with_pagination(&tx, id, pagination)
    }) {
        Ok(data) => match config_models_to_xray_outbounds(data) {
            Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
// #[axum::debug_handler]
// pub async fn get_config(
//     State(state): State<Arc<AppState>>,
//     Path((_, config_id)): Path<(i32, i32)>,
// ) -> impl IntoResponse {
//     match TransactionManager::execute_with_result(&mut db, |tx| {
//         ConfigManager::get_in_transaction(tx, config_id)
//     }) {
//         Ok(Some(config)) => match config_model_to_xray_outbound(config) {
//             Ok(data) => (StatusCode::OK, Json(data)).into_response(),
//             Err(e) => (
//                 StatusCode::INTERNAL_SERVER_ERROR,
//                 Json(json!({"error": e.to_string()})),
//             )
//                 .into_response(),
//         },
//         Ok(None) => (
//             StatusCode::NOT_FOUND,
//             Json(json!({"error": "Config not found"})),
//         )
//             .into_response(),
//         Err(e) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": e.to_string()})),
//         )
//             .into_response(),
//     }
// }

// #[axum::debug_handler]
// pub async fn add_config(
//     State(state): State<Arc<AppState>>,
//     Path(group_id): Path<i32>,
//     Json(payload): Json<AddConfigRequest>,
// ) -> impl IntoResponse {
//     let config = ConfigModel::new(group_id, payload.data, payload.extra);

//     match TransactionManager::execute_with_result(&mut db, |tx| {
//         ConfigManager::create_in_transaction(tx, &config)
//     }) {
//         Ok(id) => (StatusCode::CREATED, Json(json!({"id": id}))).into_response(),
//         Err(e) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": e.to_string()})),
//         )
//             .into_response(),
//     }
// }

// #[axum::debug_handler]
// pub async fn update_config(
//     State(state): State<Arc<AppState>>,
//     Path((_, config_id)): Path<(i32, i32)>,
//     Json(payload): Json<UpdateConfigRequest>,
// ) -> impl IntoResponse {
//     let config = ConfigModel {
//         id: config_id,
//         group_id: 0,
//         data: payload.data,
//         extra: payload.extra,
//     };

//     match TransactionManager::execute_with_result(&mut db, |tx| {
//         ConfigManager::update_in_transaction(tx, &config)
//     }) {
//         Ok(_) => (StatusCode::OK, Json(json!({"message": "Updated"}))).into_response(),
//         Err(e) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": e.to_string()})),
//         )
//             .into_response(),
//     }
// }

// #[axum::debug_handler]
// pub async fn delete_config(
//     State(state): State<Arc<AppState>>,
//     Path((_, config_id)): Path<(i32, i32)>,
// ) -> impl IntoResponse {
//     let mut db = get_db(&state).await;

//     match TransactionManager::execute_with_result(&mut db, |tx| {
//         ConfigManager::delete_in_transaction(tx, config_id)
//     }) {
//         Ok(_) => (StatusCode::OK, Json(json!({"message": "Deleted"}))).into_response(),
//         Err(e) => (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(json!({"error": e.to_string()})),
//         )
//             .into_response(),
//     }
// }

#[derive(serde::Deserialize)]
pub struct AddConfigRequest {
    pub data: String,
    pub extra: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateConfigRequest {
    pub data: String,
    pub extra: String,
}
