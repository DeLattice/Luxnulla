use axum::{
    Router,
    routing::{get, post},
};
use reqwest::Method;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::http::handlers::groups::{
    create_group, delete_all_groups, delete_group, get_group_by_name, get_groups, update_group,
};
use crate::http::handlers::xray::{apply_outbounds, get_outbounds, get_xray_status, toggle_xray};
use crate::services::{self};

const SOCKET: &str = "0.0.0.0:3000";

async fn root() -> &'static str {
    return "Server is working";
}

pub fn init() -> tokio::task::JoinHandle<()> {
    let storage_service_state = Arc::new(services::StorageService::new());

    tokio::spawn(async {
        let cors_layer = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers(Any);

        let app = Router::new()
            .route("/", get(root))
            .route("/groups", get(get_groups).delete(delete_all_groups))
            .route("/group", post(create_group).put(update_group))
            .route("/group/{name}", get(get_group_by_name).delete(delete_group))
            .route("/xray", get(get_xray_status))
            .route("/xray/outbounds", get(get_outbounds).post(apply_outbounds))
            .route("/xray/{action}", post(toggle_xray))
            .with_state(storage_service_state)
            .layer(ServiceBuilder::new().layer(cors_layer));

        let listener = tokio::net::TcpListener::bind(SOCKET).await.unwrap();

        println!("http server bind on {}", SOCKET);

        axum::serve(listener, app).await.unwrap();
    })
}
