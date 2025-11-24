use std::{process::Stdio, sync::Arc};

use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{CloseFrame, Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use luxnulla::XRAY_CONFIG_FILE;
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    http::{models::xray_config::XrayOutboundClientConfig, server::AppState},
    services::xray::{self, file::XrayFileCore},
};

#[axum::debug_handler]
pub async fn get_xray_status() -> impl IntoResponse {
    // let status_result = xray::get_xray_status();

    (StatusCode::OK, Json("xxx".to_string())).into_response()
}

#[axum::debug_handler]
pub async fn start_xray(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.xray.start().await {
        true => (
            StatusCode::OK,
            Json("Xray started successfully".to_string()),
        )
            .into_response(),
        false => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Failed to start Xray")})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn stop_xray(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match state.xray.stop().await {
        true => (
            StatusCode::OK,
            Json("Xray stopped successfully".to_string()),
        )
            .into_response(),
        false => (
            StatusCode::BAD_REQUEST,
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
pub async fn apply_outbounds(Json(configs): Json<Vec<i32>>) -> impl IntoResponse {
    // match xray::outbounds::update_outbounds(&configs) {
    //     Ok(configs) => match xray::restart_xray().await {
    //         Ok(_) => (StatusCode::OK, Json(json!(configs))),
    //         Err(err) => (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({"error": format!("Failed to restart Xray: {}", err)})),
    //         ),
    //     }
    //     .into_response(),
    //     Err(err) => (
    //         StatusCode::INTERNAL_SERVER_ERROR,
    //         Json(json!({"error": format!("Failed to use config: {}", err)})),
    //     )
    //         .into_response(),
    // }
}

#[axum::debug_handler]
pub async fn delete_outbounds(Json(configs): Json<Vec<i32>>) -> impl IntoResponse {
    // match xray::outbounds::delete_outbounds(&configs) {
    //     Ok(configs) => match xray::restart_xray().await {
    //         Ok(_) => (StatusCode::OK, Json(json!(configs))),
    //         Err(err) => (
    //             StatusCode::INTERNAL_SERVER_ERROR,
    //             Json(json!({"error": format!("Failed to restart Xray: {}", err)})),
    //         ),
    //     }
    //     .into_response(),
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
pub async fn check_configs(Json(req): Json<Vec<XrayOutboundClientConfig>>) -> impl IntoResponse {
    xray::checker::ping(req);

    StatusCode::OK
}

#[axum::debug_handler]
pub async fn ws_xray_logs_handler(
    State(state): State<Arc<AppState>>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_socket(state, socket))
}

async fn handle_socket(state: Arc<AppState>, mut socket: WebSocket) {
    while let Ok(msg) = state.xray.logs().recv().await {
        if socket
            .send(Message::Text(Utf8Bytes::from(msg)))
            .await
            .is_err()
        {
            break;
        }
    }
}
