use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Json,
};
use serde::Serialize;
use serde_json::Value;

use crate::{
    model::{
        model::{DefaultValue, FieldType},
        ParsedModel,
    },
    models::{
        item_to_response, CreateItemRequest, HealthResponse, Item, ItemResponse, ReadinessResponse,
        UpdateItemRequest,
    },
    state::AppState,
    ui::INDEX_HTML,
};

#[derive(Serialize)]
pub struct ModelApiResponse {
    classes: Vec<ModelClassResponse>,
}

#[derive(Serialize)]
pub struct ModelClassResponse {
    name: String,
    version: String,
    description: String,
    table: String,
    fields: Vec<ModelFieldResponse>,
}

#[derive(Serialize)]
pub struct ModelFieldResponse {
    name: String,
    field_type: String,
    destination_type: String,
    description: String,
    default_value: Value,
    indexed: bool,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "inventory-core",
    })
}

pub async fn ready() -> Json<ReadinessResponse> {
    Json(ReadinessResponse {
        status: "ready",
        checks: ["app"],
    })
}

pub async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

pub async fn get_model(State(state): State<AppState>) -> Json<ModelApiResponse> {
    let mut names: Vec<_> = state.model_registry.entities().collect();
    names.sort_unstable();

    let classes = names
        .into_iter()
        .filter_map(|name| state.model_registry.get(name))
        .map(parsed_to_class)
        .collect();

    Json(ModelApiResponse { classes })
}

pub async fn list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<ItemResponse>>, (StatusCode, String)> {
    let items = sqlx::query_as::<_, Item>(
        r#"
        SELECT id, owner_service, entity_type, name, category, quantity
        FROM inventory_items
        ORDER BY id
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let response = items
        .into_iter()
        .map(|item| item_to_response(&state.tenant_id, item))
        .collect();

    Ok(Json(response))
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), (StatusCode, String)> {
    validate_item_input(&input.name, &input.category, input.quantity)?;

    let item = sqlx::query_as::<_, Item>(
        r#"
        INSERT INTO inventory_items (owner_service, entity_type, name, category, quantity)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, owner_service, entity_type, name, category, quantity
        "#,
    )
    .bind("inventory-core")
    .bind("item")
    .bind(input.name.trim())
    .bind(input.category.trim())
    .bind(input.quantity)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok((
        StatusCode::CREATED,
        Json(item_to_response(&state.tenant_id, item)),
    ))
}

pub async fn update_item(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(input): Json<UpdateItemRequest>,
) -> Result<Json<ItemResponse>, (StatusCode, String)> {
    validate_item_input(&input.name, &input.category, input.quantity)?;

    let updated = sqlx::query_as::<_, Item>(
        r#"
        UPDATE inventory_items
        SET name = $1,
            category = $2,
            quantity = $3,
            updated_at = NOW()
        WHERE id = $4
        RETURNING id, owner_service, entity_type, name, category, quantity
        "#,
    )
    .bind(input.name.trim())
    .bind(input.category.trim())
    .bind(input.quantity)
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?;

    match updated {
        Some(item) => Ok(Json(item_to_response(&state.tenant_id, item))),
        None => Err((StatusCode::NOT_FOUND, "item not found".to_string())),
    }
}

pub async fn delete_item(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let result = sqlx::query(
        r#"
        DELETE FROM inventory_items
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "item not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        if let Ok(mut sigterm) = signal(SignalKind::terminate()) {
            let _ = sigterm.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}

fn validate_item_input(name: &str, category: &str, quantity: i64) -> Result<(), (StatusCode, String)> {
    if name.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "name is required".to_string()));
    }
    if category.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, "category is required".to_string()));
    }
    if quantity < 0 {
        return Err((StatusCode::BAD_REQUEST, "quantity must be >= 0".to_string()));
    }
    Ok(())
}

fn internal_error(err: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("database error: {err}"),
    )
}

fn parsed_to_class(parsed: &ParsedModel) -> ModelClassResponse {
    ModelClassResponse {
        name: parsed.model.entity.name.clone(),
        version: parsed.model.version.to_string(),
        description: parsed.model.entity.description.clone(),
        table: parsed.schema.tables[0].name.clone(),
        fields: parsed
            .model
            .entity
            .fields
            .iter()
            .map(|field| ModelFieldResponse {
                name: field.name.clone(),
                field_type: field_type_to_str(field.field_type).to_string(),
                destination_type: field.destination_type.clone(),
                description: field.description.clone(),
                default_value: default_to_json(&field.default),
                indexed: field.indexed,
            })
            .collect(),
    }
}

fn field_type_to_str(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::Boolean => "boolean",
        FieldType::Integer => "integer",
        FieldType::Float => "float",
        FieldType::Timestamp => "timestamp",
        FieldType::String => "string",
        FieldType::Text => "text",
        FieldType::Reference => "reference",
    }
}

fn default_to_json(default: &DefaultValue) -> Value {
    match default {
        DefaultValue::Boolean(v) => Value::Bool(*v),
        DefaultValue::Integer(v) => Value::Number((*v).into()),
        DefaultValue::Float(v) => serde_json::Number::from_f64(*v)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(v.to_string())),
        DefaultValue::Timestamp(v) => Value::String(v.clone()),
        DefaultValue::String(v) => Value::String(v.clone()),
        DefaultValue::Text(v) => Value::String(v.clone()),
        DefaultValue::ReferenceId(v) => Value::Number((*v).into()),
    }
}
