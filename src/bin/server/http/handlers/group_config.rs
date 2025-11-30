use axum::{
    Json,
    extract::{Path, State},
    response::IntoResponse,
};
use reqwest::StatusCode;
use serde_json::json;
use std::sync::Arc;
use url::Url;

use crate::{
    http::{models::xray_config::XrayOutboundClientConfigModel, server::AppState},
    services::{
        common::{
            convertors::{config_model_to_xray_outbound, config_models_to_xray_outbounds},
            process_config,
        },
        db::TransactionManager,
        repository::{
            config::{ConfigModel, ConfigRepository},
            group::GroupRepository,
        },
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

#[axum::debug_handler]
pub async fn refresh_configs_by_group_id(
    State(state): State<Arc<AppState>>,
    Path(group_id): Path<i32>,
) -> impl IntoResponse {
    let group = TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        GroupRepository::get_by_id(tx, group_id)
    });

    match group {
        Ok(Some(group)) => {
            let Some(current_sub_url) = group.subscribe_url.as_ref() else {
                return (
                    StatusCode::NOT_FOUND,
                    Json(json!({"error": "Not found subscribe URL"})),
                )
                    .into_response();
            };

            let configs = process_config(current_sub_url.as_str()).await.unwrap();

            match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
                ConfigRepository::delete_by_group_id(&tx, group_id)?;

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
                    .collect::<Vec<_>>();

                Ok(result)
            }) {
                Ok(result) => (StatusCode::OK, Json(result)).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": format!("Group with ID {} not found", group_id)})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
