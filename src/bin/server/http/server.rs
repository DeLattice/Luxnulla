use axum::{
    Router,
    routing::{get, post},
};
use reqwest::Method;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::http::handlers::{
    groups::refresh_group,
    xray::{
        apply_outbounds, delete_outbounds, get_outbounds, get_xray_config, get_xray_status,
        toggle_xray,
    },
};
use crate::http::handlers::{
    groups::{
        create_group, delete_all_groups, delete_group, get_group, get_list_group_names,
        get_paginated_group_configs, update_group,
    },
    xray::check_configs,
};
use crate::services::{self};

const SOCKET: &str = "0.0.0.0:8400";

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
                Router::new()
                    .nest(
                        "/{id}",
                        Router::new()
                            .route(
                                "/",
                                get(get_group)
                                    .put(update_group)
                                    .delete(delete_group)
                                    .patch(refresh_group),
                            )
                            .route("/configs", get(get_paginated_group_configs)),
                    )
                    .route(
                        "/",
                        get(get_list_group_names)
                            .post(create_group)
                            .delete(delete_all_groups),
                    ),
            )
            .nest(
                "/xray",
                Router::new()
                    .route("/", get(get_xray_status))
                    .route(
                        "/outbounds",
                        get(get_outbounds)
                            .post(apply_outbounds)
                            .delete(delete_outbounds),
                    )
                    .route("/{action}", post(toggle_xray))
                    .route("/config", get(get_xray_config))
                    .route("/ping", post(check_configs)),
            )
            .with_state(storage_service_state)
            .layer(ServiceBuilder::new().layer(cors_layer));

        let listener = tokio::net::TcpListener::bind(SOCKET).await.unwrap();

        println!("http server bind on {}", SOCKET);

        axum::serve(listener, app).await.unwrap();
    })
}
