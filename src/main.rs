use std::{env, net::SocketAddr};

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
    routing::{delete, get, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

#[derive(Clone)]
struct AppState {
    db: PgPool,
}

#[derive(Serialize, FromRow)]
struct Item {
    id: i64,
    name: String,
    manufacturer: Option<String>,
    category: Option<String>,
    sku: Option<String>,
    quantity: i32,
    location: Option<String>,
    description: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
struct CreateItem {
    name: String,
    manufacturer: Option<String>,
    category: Option<String>,
    sku: Option<String>,
    quantity: i32,
    location: Option<String>,
    description: Option<String>,
}

type UpdateItem = CreateItem;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "inventory=info,tower_http=info".into()),
        )
        .init();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/inventory".to_string());
    let port: u16 = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("PORT must be a valid u16");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::query(include_str!("../db/schema.sql"))
        .execute(&db)
        .await?;

    let app_state = AppState { db };

    let app = Router::new()
        .route("/", get(index))
        .route("/api/items", get(list_items).post(create_item))
        .route("/api/items/:id", put(update_item).delete(delete_item))
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn index() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        Html(include_str!("index.html")),
    )
}

async fn list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<Item>>, (StatusCode, String)> {
    let items = sqlx::query_as::<_, Item>(
        r#"
        SELECT id, name, manufacturer, category, sku, quantity, location, description, created_at, updated_at
        FROM items
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(items))
}

async fn create_item(
    State(state): State<AppState>,
    Json(payload): Json<CreateItem>,
) -> Result<(StatusCode, Json<Item>), (StatusCode, String)> {
    validate_item(&payload)?;

    let item = sqlx::query_as::<_, Item>(
        r#"
        INSERT INTO items (name, manufacturer, category, sku, quantity, location, description)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, name, manufacturer, category, sku, quantity, location, description, created_at, updated_at
        "#,
    )
    .bind(payload.name)
    .bind(payload.manufacturer)
    .bind(payload.category)
    .bind(payload.sku)
    .bind(payload.quantity)
    .bind(payload.location)
    .bind(payload.description)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(item)))
}

async fn update_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateItem>,
) -> Result<Json<Item>, (StatusCode, String)> {
    validate_item(&payload)?;

    let item = sqlx::query_as::<_, Item>(
        r#"
        UPDATE items
        SET name = $1,
            manufacturer = $2,
            category = $3,
            sku = $4,
            quantity = $5,
            location = $6,
            description = $7,
            updated_at = now()
        WHERE id = $8
        RETURNING id, name, manufacturer, category, sku, quantity, location, description, created_at, updated_at
        "#,
    )
    .bind(payload.name)
    .bind(payload.manufacturer)
    .bind(payload.category)
    .bind(payload.sku)
    .bind(payload.quantity)
    .bind(payload.location)
    .bind(payload.description)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    match item {
        Some(item) => Ok(Json(item)),
        None => Err((StatusCode::NOT_FOUND, format!("Item {id} not found"))),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query("DELETE FROM items WHERE id = $1")
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, format!("Item {id} not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}

fn validate_item(item: &CreateItem) -> Result<(), (StatusCode, String)> {
    if item.name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".to_string()));
    }

    if item.quantity < 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "quantity cannot be negative".to_string(),
        ));
    }

    Ok(())
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
