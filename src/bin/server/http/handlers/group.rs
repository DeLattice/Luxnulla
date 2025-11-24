use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use futures::{StreamExt, stream};
use serde_json::json;
use std::sync::Arc;
use url::Url;

use crate::{
    http::{models::xray_config::XrayOutboundClientConfig, server::AppState},
    services::{
        common::process_config,
        db::TransactionManager,
        repository::{
            config::{ConfigModel, ConfigRepository},
            group::{GroupModel, GroupRepository},
        },
    },
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub name: String,
    pub subscribe_url: Option<Url>,
    pub configs: Vec<String>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupResponseSuccess {
    pub id: i32,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe_url: Option<Url>,
}

#[axum::debug_handler]
pub async fn create_group(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    let group = GroupModel::new(payload.name, payload.subscribe_url);

    let count = payload.configs.len();
    let configs: Vec<XrayOutboundClientConfig> = stream::iter(payload.configs)
        .map(async |raw| process_config(&raw).await.unwrap_or_default())
        .buffer_unordered(count)
        .flat_map(|v| stream::iter(v))
        .collect()
        .await;

    if configs.len() == 0 {
        return (StatusCode::BAD_REQUEST, Json(json!({}))).into_response();
    }

    let (group_id, ids) = TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        let group_id = GroupRepository::create(&tx, &group)?;

        let ids = configs
            .iter()
            .map(|config| {
                let data = serde_json::to_string(&config).unwrap();
                let extra = config
                    .extra()
                    .and_then(|extra| serde_json::to_string(&extra).ok())
                    .unwrap_or_default();

                ConfigModel::new(group_id, data, extra)
            })
            .map(|model| ConfigRepository::create(&tx, &model))
            .flatten()
            .collect::<Vec<_>>();

        Ok((group_id, ids))
    })
    .unwrap();

    (
        StatusCode::CREATED,
        Json(CreateGroupResponseSuccess {
            id: group_id,
            name: group.name,
            subscribe_url: group.subscribe_url,
        }),
    )
        .into_response()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetGroupResponseSuccess {
    pub id: i32,
    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe_url: Option<Url>,
}

#[axum::debug_handler]
pub async fn get_group_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        GroupRepository::get_by_id(&tx, id)
    }) {
        Ok(group) => match group {
            Some(group) => {
                let response = GetGroupResponseSuccess {
                    id: group.id,
                    name: group.name,
                    subscribe_url: group.subscribe_url,
                };
                (StatusCode::OK, Json(response)).into_response()
            }
            _ => (StatusCode::NOT_FOUND).into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub subscribe_url: Option<Url>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupResponse {
    pub name: Option<String>,
    pub subscribe_url: Option<Url>,
}

#[axum::debug_handler]
pub async fn update_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateGroupRequest>,
) -> impl IntoResponse {
    let result = TransactionManager::execute_with_result(&mut state.get_conn(), move |tx| {
        let Some(current_group) = GroupRepository::get_by_id(tx, id)? else {
            return Ok(false);
        };

        let updated_model = GroupModel {
            id: current_group.id,
            name: payload.name.unwrap_or(current_group.name),
            subscribe_url: payload.subscribe_url.or(current_group.subscribe_url),
        };

        GroupRepository::update(tx, &updated_model)?;
        Ok(true)
    });

    match result {
        Ok(v) => match v {
            true => StatusCode::OK.into_response(),
            false => (
                StatusCode::NOT_FOUND,
                Json(json!({"error": format!("Group with ID {} not found", id)})),
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
pub async fn delete_group(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i32>,
) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        GroupRepository::delete(&tx, id)
    }) {
        Ok(v) => match v {
            true => (StatusCode::OK).into_response(),
            false => (StatusCode::NOT_FOUND).into_response(),
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetListGroupsResponse {
    pub id: i32,
    pub name: String,
    pub subscribe_url: Option<Url>,
}

#[axum::debug_handler]
pub async fn get_list_groups(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match TransactionManager::execute_with_result(&mut state.get_conn(), |tx| {
        GroupRepository::get_all(tx)
    }) {
        Ok(groups) => {
            let groups = groups
                .into_iter()
                .map(|group| GetListGroupsResponse {
                    id: group.id,
                    name: group.name,
                    subscribe_url: group.subscribe_url,
                })
                .collect::<Vec<_>>();

            (StatusCode::OK, Json(groups)).into_response()
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": err.to_string()})),
        )
            .into_response(),
    }
}
