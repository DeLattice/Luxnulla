use axum::{
    Router,
    routing::{get, post},
};
use reqwest::Method;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::http::handlers::groups::{
    create_group, delete_all_groups, delete_group, get_all_groups, get_paginated_group_configs,
    get_group_by_name, update_group,
};
use crate::http::handlers::xray::{apply_outbounds, get_outbounds, get_xray_status, toggle_xray};
use crate::services::{self};

const SOCKET: &str = "0.0.0.0:3000";

async fn root() -> &'static str {
    return "Server is working";
}

pub fn init() -> tokio::task::JoinHandle<()> {
    let storage_service_state = Arc::new(services::StorageService::new());

    let cors_layer = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(Any);

    tokio::spawn(async {
        let app = Router::new()
            .route("/", get(root))
            .nest(
                "/groups",
                Router::new().route("/", get(get_all_groups).delete(delete_all_groups)),
            )
            .nest(
                "/group",
                Router::new()
                    .route(
                        "/",
                        get(get_group_by_name)
                            .post(create_group)
                            .put(update_group)
                            .delete(delete_group),
                    )
                    .nest(
                        "/{name}",
                        Router::new().route("/configs", get(get_paginated_group_configs)),
                    ),
            )
            .nest(
                "/xray",
                Router::new()
                    .route("/", get(get_xray_status))
                    .route("/outbounds", get(get_outbounds).post(apply_outbounds))
                    .route("/{action}", post(toggle_xray)),
            )
            .with_state(storage_service_state)
            .layer(ServiceBuilder::new().layer(cors_layer));

        let listener = tokio::net::TcpListener::bind(SOCKET).await.unwrap();

        println!("http server bind on {}", SOCKET);

        axum::serve(listener, app).await.unwrap();
    })
}
