mod config;
mod db;
mod model;
mod models;
mod phase1;
mod runtime;
mod schema;
mod state;
mod ui;

use axum::{routing::{get, put}, Router};
use config::{init_tracing, load_config};
use phase1::http::{
    create_item, delete_item, get_items_view, get_model, health, index, list_items, ready,
    shutdown_signal,
    update_item,
};
use runtime::CoLocatedRuntime;
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
    db::ensure_schema(&db, &config.model_dir)
        .await
        .expect("failed to ensure database schema");

    let runtime = CoLocatedRuntime::load(db.clone(), config.tenant_id, &config.model_dir)
        .expect("failed to load co-located runtime");
    info!(
        "loaded {} model definitions from {}",
        runtime.model_count(),
        config.model_dir.display()
    );

    let mut event_receiver = runtime.subscribe_events();
    tokio::spawn(async move {
        loop {
            match event_receiver.recv().await {
                Ok(event) => {
                    info!(
                        event_type = event.event_type,
                        entity_id = %event.entity_id,
                        tenant_id = %event.tenant_id,
                        "published in-process event"
                    );
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                    info!(skipped, "event subscriber lagged behind in-process stream");
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
            }
        }
    });

    let state = AppState { runtime };

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/api/model", get(get_model))
        .route("/api/views/items", get(get_items_view))
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
