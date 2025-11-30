use std::{process::Stdio, sync::Arc};

use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use elux::XRAY_CONFIG_FILE;
use reqwest::StatusCode;
use serde_json::{Value, json};
use tokio::{
    fs::{self, File},
    io::{AsyncBufReadExt, BufReader},
};

use crate::{
    http::{models::xray_config::XrayOutboundClientConfig, server::AppState},
    services::{
        db::TransactionManager,
        repository::config::{ConfigModel, ConfigRepository},
        xray::{self, file::XrayFileCore},
    },
    utils::config::AppPaths,
};

#[axum::debug_handler]
pub async fn get_xray_status(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.xray_service.status().await)).into_response()
}

#[axum::debug_handler]
pub async fn start_xray(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.xray_service.start().await {
        true => (StatusCode::OK).into_response(),
        false => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to start Xray")})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn stop_xray(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.xray_service.stop().await {
        true => (StatusCode::OK,).into_response(),
        false => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to stop Xray")})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn restart_xray(State(state): State<Arc<AppState>>) -> impl IntoResponse {}

#[axum::debug_handler]
pub async fn get_outbounds() -> impl IntoResponse {
    match xray::outbounds::get_outbounds() {
        Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to get config: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn update_outbounds(
    State(state): State<Arc<AppState>>,
    Json(ids): Json<Vec<i32>>,
) -> impl IntoResponse {
    let configs_from_db_result =
        TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
            ConfigRepository::get_by_ids(tx, ids.as_slice())
        });

    let configs_to_update = match configs_from_db_result {
        Ok(configs_opt) => configs_opt.unwrap_or_default(),
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to retrieve configs from database: {}", e)})),
            )
                .into_response();
        }
    };

    match xray::outbounds::update_outbounds(configs_to_update.as_slice()) {
        Ok(updated_configs) => (StatusCode::OK, Json(updated_configs)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to update Xray outbounds: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn delete_outbounds(Json(configs): Json<Vec<i32>>) -> impl IntoResponse {
    // match xray::outbounds::delete_outbounds(&configs) {
    //     Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
    //     Err(err) => (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!({"error": format!("Failed to use config: {}", err)})),
    //     )
    //         .into_response(),
    // }
}

#[axum::debug_handler]
pub async fn get_xray_config() -> impl IntoResponse {
    let xray_core = XrayFileCore::new(XRAY_CONFIG_FILE);

    match xray_core.read_xray_file() {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to use config: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn update_xray_config(Json(config): Json<Value>) -> impl IntoResponse {
    let xray_core = XrayFileCore::new(XRAY_CONFIG_FILE);

    match xray_core.write_full_config(&config) {
        Ok(()) => (StatusCode::OK).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to use config: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn ws_xray_logs_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(state, socket))
}

async fn handle_socket(state: Arc<AppState>, mut socket: WebSocket) {
    if let Ok(file) = File::open(&AppPaths::get().xray_log).await {
        let mut reader = BufReader::new(file).lines();

        while let Ok(Some(line)) = reader.next_line().await {
            if socket
                .send(Message::Text(Utf8Bytes::from(line)))
                .await
                .is_err()
            {
                return;
            }
        }
    }

    let mut rx = state.xray_service.logs();

    while let Ok(msg) = rx.recv().await {
        if socket
            .send(Message::Text(Utf8Bytes::from(msg)))
            .await
            .is_err()
        {
            break;
        }
    }
}
