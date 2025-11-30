use axum::{
    extract::{Json, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use futures::{StreamExt, stream};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

use crate::{
    http::{
        models::xray_config::{
            ExtraOutboundClientConfig, XrayOutboundClientConfig, XrayOutboundClientConfigModel,
        },
        server::AppState,
    },
    services::{
        common::{
            convertors::{config_model_to_xray_outbound, config_models_to_xray_outbounds},
            paginator::PaginationParams,
            process_config,
        },
        db::TransactionManager,
        repository::config::{ConfigModel, ConfigRepository},
    },
};

#[derive(Serialize)]
struct CreateConfigsResponseSuccess {
    id: i32,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<ExtraOutboundClientConfig>,

    #[serde(flatten)]
    pub config: XrayOutboundClientConfig,
}

//todo | replace process_config => light version without destructions
#[axum::debug_handler]
pub async fn create_configs(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i32>,
    Json(configs): Json<Vec<String>>,
) -> impl IntoResponse {
    let count = configs.len();

    let configs = stream::iter(configs)
        .map(async |raw| process_config(&raw).await.unwrap_or_default())
        .buffer_unordered(count)
        .flat_map(|v| stream::iter(v))
        .collect::<Vec<_>>()
        .await;

    if configs.len() == 0 {
        return (StatusCode::BAD_REQUEST, Json(json!({}))).into_response();
    }

    let result: Result<Vec<XrayOutboundClientConfigModel>, _> =
        TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
            let result = configs
                .iter()
                .map(|config| {
                    let data = serde_json::to_string(&config).unwrap();
                    let extra = config
                        .extra()
                        .and_then(|extra| serde_json::to_string(&extra).ok())
                        .unwrap_or_default();

                    ConfigModel::new(group_id, data, extra)
                })
                .filter_map(|mut model| {
                    model.id = ConfigRepository::create(&tx, &model).ok()?;
                    config_model_to_xray_outbound(model).ok()
                })
                .collect();

            Ok(result)
        });

    match result {
        Ok(configs) => (StatusCode::CREATED, Json(configs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, Json("error".to_string())).into_response(),
    }
}

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
//

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

#[axum::debug_handler]
pub async fn delete_config_by_id(
    State(state): State<Arc<AppState>>,
    Path(config_id): Path<i32>,
) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        ConfigRepository::delete(tx, config_id)
    }) {
        Ok(true) => (StatusCode::OK).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn delete_config_by_ids(
    State(state): State<Arc<AppState>>,
    Json(config_ids): Json<Vec<i32>>,
) -> impl IntoResponse {
    if config_ids.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "No config IDs provided for deletion"})),
        )
            .into_response();
    }

    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        ConfigRepository::delete_by_ids(tx, config_ids.as_slice())
    }) {
        Ok(true) => (StatusCode::OK).into_response(),
        Ok(false) => (StatusCode::NOT_FOUND).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
