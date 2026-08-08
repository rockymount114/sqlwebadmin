mod config;
mod db;
mod handlers;
mod models;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use handlers::AppState;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sqlwebadmin=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = config::load_config();
    let port = config.port;

    tracing::info!(
        "Loaded config: default driver = '{}', connection string configured.",
        config.default_driver
    );

    let state = Arc::new(AppState { config });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let api_routes = Router::new()
        .route("/config", get(handlers::get_config_handler))
        .route("/connect/test", post(handlers::test_connection_handler))
        .route("/schema/databases", get(handlers::get_databases_handler))
        .route("/schema/tree", get(handlers::schema_tree_handler))
        .route("/schema/children", get(handlers::schema_children_handler))
        .route("/schema/definition", get(handlers::schema_definition_handler))
        .route("/query/execute", post(handlers::execute_query_handler))
        .route("/query/export", post(handlers::export_query_handler));

    let app = Router::new()
        .nest("/api", api_routes)
        .nest_service("/", ServeDir::new("static"))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 SQL Web Admin server listening on http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
