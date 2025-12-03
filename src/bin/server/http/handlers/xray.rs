use axum::{
    Json,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, Utf8Bytes, WebSocket},
    },
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::{
    fs::File,
    io::{AsyncBufReadExt, BufReader},
};

use crate::{
    http::server::AppState,
    services::{
        db::transaction::run_db_transaction,
        repository::config::ConfigRepository,
        xray::{
            self,
            file::{observatory::ObservatoryOps, routing::RoutingOps},
        },
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
            StatusCode::LOCKED,
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
            StatusCode::LOCKED,
            Json(json!({"error": format!("Failed to stop Xray")})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn restart_xray(State(state): State<Arc<AppState>>) -> impl IntoResponse {}

#[axum::debug_handler]
pub async fn get_outbounds(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match xray::outbounds::get_outbounds(&state.xray_file) {
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
    let configs_from_db_result = run_db_transaction(&mut state.get_conn(), |tx| {
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

    match xray::outbounds::update_outbounds(&state.xray_file, configs_to_update.as_slice()) {
        Ok(updated_configs) => {
            let ids = ids.as_slice();

            state.xray_file.set_balancer_ids(ids);
            state.xray_file.set_observatory_ids(ids);

            (StatusCode::OK, Json(updated_configs)).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to update Xray outbounds: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn delete_outbounds(
    State(state): State<Arc<AppState>>,
    Json(configs): Json<Vec<i32>>,
) -> impl IntoResponse {
    match xray::outbounds::delete_outbounds(&state.xray_file, &configs) {
        Ok(configs) => (StatusCode::OK, Json(configs)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to use config: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn get_xray_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(state.xray_file.read_with_json(|root| root.clone())),
    )
}

#[axum::debug_handler]
pub async fn update_xray_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<Value>,
) -> impl IntoResponse {
    match state.xray_file.write_full_config(&config) {
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
