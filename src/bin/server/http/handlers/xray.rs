use axum::{Json, extract::Path, response::IntoResponse};
use luxnulla::XRAY_CONFIG_FILE;
use reqwest::StatusCode;
use serde_json::json;

use crate::{
    http::services::model::xray_config::XrayOutboundClientConfig,
    services::xray::{self, file::XrayFileCore},
};

#[axum::debug_handler]
pub async fn get_xray_status() -> impl IntoResponse {
    let status_result = xray::get_xray_status();

    (StatusCode::OK, Json(status_result)).into_response()
}

#[axum::debug_handler]
pub async fn toggle_xray(Path(action): Path<String>) -> impl IntoResponse {
    let response = match action.as_str() {
        "on" => match xray::spawn_xray().await {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({"status": "Xray started successfully"})),
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to start Xray: {}", err)})),
            ),
        },
        "off" => match xray::stop_xray().await {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({"status": "Xray stopped successfully"})),
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to stop Xray: {}", err)})),
            ),
        },
        "restart" => match xray::restart_xray().await {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({"status": "Xray restarted successfully"})),
            ),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to restart Xray: {}", err)})),
            ),
        },
        _ => (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": format!("Unknown action: {}", action)})),
        ),
    };

    response.into_response()
}

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
    match xray::outbounds::update_outbounds(&configs) {
        Ok(configs) => match xray::restart_xray().await {
            Ok(_) => (StatusCode::OK, Json(json!(configs))),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to restart Xray: {}", err)})),
            ),
        }
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to use config: {}", err)})),
        )
            .into_response(),
    }
}

#[axum::debug_handler]
pub async fn delete_outbounds(Json(configs): Json<Vec<i32>>) -> impl IntoResponse {
    match xray::outbounds::delete_outbounds(&configs) {
        Ok(configs) => match xray::restart_xray().await {
            Ok(_) => (StatusCode::OK, Json(json!(configs))),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Failed to restart Xray: {}", err)})),
            ),
        }
        .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": format!("Failed to use config: {}", err)})),
        )
            .into_response(),
    }
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
