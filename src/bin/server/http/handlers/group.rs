use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde_json::json;
use std::sync::Arc;
use url::Url;

use crate::{
    http::server::AppState,
    services::{
        db::transaction::run_db_transaction,
        repository::group::{GroupModel, GroupRepository},
    },
};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub name: String,
    pub subscribe_url: Option<Url>,
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

    match run_db_transaction(&mut state.get_conn(), |tx| {
        GroupRepository::create(&tx, &group)
    }) {
        Ok(group_id) => (
            StatusCode::CREATED,
            Json(CreateGroupResponseSuccess {
                id: group_id,
                name: group.name,
                subscribe_url: group.subscribe_url,
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
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
    match run_db_transaction(&mut state.get_conn(), |tx| {
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
    let result = run_db_transaction(&mut state.get_conn(), move |tx| {
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
    match run_db_transaction(&mut state.get_conn(), |tx| GroupRepository::delete(&tx, id)) {
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
    match run_db_transaction(&mut state.get_conn(), |tx| GroupRepository::get_all(tx)) {
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

#[axum::debug_handler]
pub async fn delete_all_groups(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    match run_db_transaction(&mut state.get_conn(), |tx| {
        let groups = GroupRepository::get_all(&tx)?;

        let result = groups
            .iter()
            .map(|group| GroupRepository::delete(&tx, group.id))
            .collect::<Vec<_>>();

        Ok(result)
    }) {
        Ok(_) => (StatusCode::OK).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
