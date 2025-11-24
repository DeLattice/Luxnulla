use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde_json::json;
use std::{error::Error, sync::Arc};
use url::Url;

use crate::{
    http::{models::xray_config::XrayOutboundClientConfigModel, server::AppState},
    services::{
        common::convertors::config_models_to_xray_outbounds,
        db::TransactionManager,
        repository::{config::ConfigRepository, group::GroupRepository},
    },
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetListGroupsResponse {
    pub id: i32,
    pub name: String,
    pub subscribe_url: Option<Url>,
    pub configs: Vec<XrayOutboundClientConfigModel>,
}

#[axum::debug_handler]
pub async fn get_group_with_configs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        let Some(group) = GroupRepository::get_by_id(tx, id)? else {
            return Ok(None);
        };

        let configs = ConfigRepository::get_by_group_id(&tx, id)?;

        let outbound_configs = config_models_to_xray_outbounds(configs).unwrap();

        Ok(Some(GetListGroupsResponse {
            id: group.id,
            name: group.name,
            subscribe_url: group.subscribe_url,
            configs: outbound_configs,
        }))
    }) {
        Ok(Some(data)) => (StatusCode::OK, Json(data)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Group with ID {} not found", id)})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
