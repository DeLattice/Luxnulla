use axum::{
    Router,
    routing::{any, delete, get, post},
};
use luxnulla::{DB_FILE_NAME, SOCKET};
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::SqliteConnectionManager;
use reqwest::Method;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};

use crate::{
    http::handlers::{
        config::{create_configs, delete_config_by_id},
        group::delete_all_groups,
        group_config::refresh_configs_by_group_id,
        xray::{restart_xray, stop_xray, update_xray_config},
    },
    utils::config::{xray_config_file, xray_log_file},
};
use crate::{
    http::handlers::{
        config::{delete_config_by_ids, get_paginated_configs_by_group_id},
        group::{create_group, delete_group, get_group_by_id, get_list_groups, update_group},
        xray::ws_xray_logs_handler,
    },
    services::xray::service::XrayService,
};

use crate::{
    http::handlers::xray::{
        delete_outbounds, get_outbounds, get_xray_config, get_xray_status, start_xray,
        update_outbounds,
    },
    utils,
};

pub struct AppState {
    pub db_pool: Pool<SqliteConnectionManager>,
    pub xray_service: XrayService,
}

impl AppState {
    pub fn init() -> Self {
        let app_dir = utils::config::app_config_dir();
        let db_path = app_dir.join(DB_FILE_NAME);

        let manager = SqliteConnectionManager::file(&db_path)
            .with_init(|c| c.execute_batch("PRAGMA foreign_keys = ON;"));
        let pool = Pool::new(manager).unwrap();

        AppState {
            db_pool: pool,
            xray_service: XrayService::new(xray_config_file(), xray_log_file()),
        }
    }

    pub fn get_conn(&self) -> PooledConnection<SqliteConnectionManager> {
        self.db_pool.get().unwrap()
    }
}

async fn root() -> &'static str {
    return "Server is working";
}

pub fn init() -> tokio::task::JoinHandle<()> {
    let state: Arc<AppState> = Arc::new(AppState::init());

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
                    .route(
                        "/",
                        get(get_list_groups)
                            .post(create_group)
                            .delete(delete_all_groups),
                    )
                    .route("/configs/{id}", delete(delete_config_by_id))
                    .route(
                        "/{id}",
                        get(get_group_by_id).put(update_group).delete(delete_group),
                    )
                    .route(
                        "/{id}/configs",
                        post(create_configs)
                            .get(get_paginated_configs_by_group_id)
                            .delete(delete_config_by_ids),
                    )
                    .route("/{id}/refresh", post(refresh_configs_by_group_id)),
            )
            .nest(
                "/xray",
                Router::new()
                    .route("/", get(get_xray_status))
                    .route(
                        "/outbounds",
                        get(get_outbounds)
                            .post(update_outbounds)
                            .delete(delete_outbounds),
                    )
                    .route("/on", post(start_xray))
                    .route("/off", post(stop_xray))
                    .route("/restart", post(restart_xray))
                    .route("/config", get(get_xray_config).post(update_xray_config))
                    .route("/logs/ws", any(ws_xray_logs_handler)),
            )
            .with_state(state)
            .layer(ServiceBuilder::new().layer(cors_layer));

        let listener = tokio::net::TcpListener::bind(SOCKET).await.unwrap();

        println!("http server bind on {}", SOCKET);

        axum::serve(listener, app).await.unwrap();
    })
}
