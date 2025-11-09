use crate::services::StorageService;
use axum::{extract::State, response::IntoResponse};
use std::sync::Arc;

pub async fn update_config(State(storage): State<Arc<StorageService>>) -> impl IntoResponse {
    ()
}
