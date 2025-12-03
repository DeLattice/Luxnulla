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
        db::transaction::{run_db_transaction, run_transaction},
        repository::config::{ConfigModel, ConfigRepository},
        xray,
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

    let result = run_transaction(&mut state.get_conn(), |tx| {
        let configs = configs
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
            .collect::<Vec<_>>();

        Ok(configs)
    });

    match result {
        Ok(configs) => (StatusCode::CREATED, Json(configs)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn get_configs_by_group_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match run_db_transaction(&mut state.get_conn(), |tx| {
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
    match run_db_transaction(&mut state.get_conn(), |tx| {
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

#[axum::debug_handler]
pub async fn delete_config_by_id(
    State(state): State<Arc<AppState>>,
    Path(config_id): Path<i32>,
) -> impl IntoResponse {
    match run_transaction(&mut state.get_conn(), |tx| {
        ConfigRepository::delete(tx, config_id)?;
        xray::outbounds::delete_outbound(&state.xray_file, config_id)?;

        Ok(true)
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

    match run_transaction(&mut state.get_conn(), |tx| {
        ConfigRepository::delete_by_ids(tx, config_ids.as_slice())?;
        xray::outbounds::delete_outbounds(&state.xray_file, &config_ids)?;

        Ok(true)
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
