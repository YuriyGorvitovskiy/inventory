mod config;
mod db;
mod model;
mod handlers;
mod models;
mod state;
mod ui;

use axum::{routing::{get, put}, Router};
use config::{init_tracing, load_config};
use handlers::{
    create_item, delete_item, health, index, list_items, ready, shutdown_signal, update_item,
};
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tracing::info;

#[tokio::main]
async fn main() {
    init_tracing();

    let config = load_config();
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.db_url)
        .await
        .expect("failed to connect to postgres");
    db::ensure_schema(&db)
        .await
        .expect("failed to ensure database schema");

    let state = AppState {
        db,
        tenant_id: config.tenant_id,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/items", get(list_items).post(create_item))
        .route("/api/items/{id}", put(update_item).delete(delete_item))
        .with_state(state);

    info!("starting inventory-core on {}", config.addr);

    let listener = tokio::net::TcpListener::bind(config.addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}
