use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use serde_json::{Map, Value};
use std::collections::HashMap;
use tracing::info;

use crate::{
    models::{CreateItemRequest, ItemResponse, RuntimeActionRequest, UpdateItemRequest},
    runtime::contracts::{
        ActionInvocation, ActionOutcome, ResolvedView, RuntimeError, RuntimeModelApiResponse,
    },
    runtime::{RuntimeRequest, RuntimeResponse},
    state::AppState,
};

pub async fn health(State(state): State<AppState>) -> Json<crate::models::HealthResponse> {
    match state
        .runtime
        .dispatch("health", RuntimeRequest::Empty)
        .await
        .expect("health route should be valid")
    {
        RuntimeResponse::Health(response) => Json(response),
        _ => unreachable!("health route must return health response"),
    }
}

pub async fn ready(State(state): State<AppState>) -> Json<crate::models::ReadinessResponse> {
    match state
        .runtime
        .dispatch("ready", RuntimeRequest::Empty)
        .await
        .expect("ready route should be valid")
    {
        RuntimeResponse::Ready(response) => Json(response),
        _ => unreachable!("ready route must return readiness response"),
    }
}

pub async fn index(State(state): State<AppState>) -> Html<&'static str> {
    match state
        .runtime
        .dispatch("root.index", RuntimeRequest::Empty)
        .await
        .expect("index route should be valid")
    {
        RuntimeResponse::IndexHtml(html) => Html(html),
        _ => unreachable!("index route must return html"),
    }
}

pub async fn get_model(State(state): State<AppState>) -> Json<RuntimeModelApiResponse> {
    match state
        .runtime
        .dispatch("api.model.describe", RuntimeRequest::Empty)
        .await
        .expect("model route should be valid")
    {
        RuntimeResponse::Model(model) => Json(model),
        _ => unreachable!("model route must return model response"),
    }
}

pub async fn get_items_view(
    State(state): State<AppState>,
) -> Result<Json<ResolvedView>, (StatusCode, String)> {
    resolve_view_response(&state, "inventory.item.list", Map::new()).await
}

pub async fn get_view(
    Path(view_name): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<ResolvedView>, (StatusCode, String)> {
    let params = query_params_to_view_params(query)?;
    resolve_view_response(&state, &view_name, params).await
}

async fn resolve_view_response(
    state: &AppState,
    view_name: &str,
    params: Map<String, Value>,
) -> Result<Json<ResolvedView>, (StatusCode, String)> {
    let view = state
        .runtime
        .resolve_view(view_name, params)
        .await
        .map_err(runtime_error)?;

    Ok(Json(view))
}

pub async fn list_items(
    State(state): State<AppState>,
) -> Result<Json<Vec<ItemResponse>>, (StatusCode, String)> {
    match state
        .runtime
        .dispatch("api.items.list", RuntimeRequest::Empty)
        .await
        .map_err(runtime_error)?
    {
        RuntimeResponse::Items(items) => Ok(Json(items)),
        _ => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected runtime list response".to_string(),
        )),
    }
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), (StatusCode, String)> {
    let response = dispatch_action(
        &state,
        "api.items.create",
        ActionInvocation::CreateItem {
            context: state.runtime.request_context(),
            name: input.name,
            category: input.category,
            quantity: input.quantity,
        },
    )
    .await?;
    let RuntimeResponse::Action(result) = response else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected runtime action response".to_string(),
        ));
    };

    info!(
        action = result.definition.name,
        event_type = result.event.event_type,
        entity_id = %result.event.entity_id,
        "completed runtime action"
    );

    match result.outcome {
        ActionOutcome::Item(item) => Ok((StatusCode::CREATED, Json(item.into()))),
        ActionOutcome::Deleted { .. } => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected delete outcome for create item action".to_string(),
        )),
    }
}

pub async fn update_item(
    Path(id): Path<i64>,
    State(state): State<AppState>,
    Json(input): Json<UpdateItemRequest>,
) -> Result<Json<ItemResponse>, (StatusCode, String)> {
    let response = dispatch_action(
        &state,
        "api.items.update",
        ActionInvocation::UpdateItem {
            context: state.runtime.request_context(),
            id,
            name: input.name,
            category: input.category,
            quantity: input.quantity,
        },
    )
    .await?;
    let RuntimeResponse::Action(result) = response else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected runtime action response".to_string(),
        ));
    };

    info!(
        action = result.definition.name,
        event_type = result.event.event_type,
        entity_id = %result.event.entity_id,
        "completed runtime action"
    );

    match result.outcome {
        ActionOutcome::Item(item) => Ok(Json(item.into())),
        ActionOutcome::Deleted { .. } => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected delete outcome for update item action".to_string(),
        )),
    }
}

pub async fn delete_item(
    Path(id): Path<i64>,
    State(state): State<AppState>,
) -> Result<StatusCode, (StatusCode, String)> {
    let response = dispatch_action(
        &state,
        "api.items.delete",
        ActionInvocation::DeleteItem {
            context: state.runtime.request_context(),
            id,
        },
    )
    .await?;
    let RuntimeResponse::Action(result) = response else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected runtime action response".to_string(),
        ));
    };

    match result.outcome {
        ActionOutcome::Deleted { id: deleted_id } => {
            info!(
                action = result.definition.name,
                event_type = result.event.event_type,
                entity_id = %result.event.entity_id,
                deleted_id,
                "completed runtime action"
            );
        }
        ActionOutcome::Item(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "unexpected item outcome for delete item action".to_string(),
            ));
        }
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn execute_action(
    Path(action_name): Path<String>,
    State(state): State<AppState>,
    Json(input): Json<RuntimeActionRequest>,
) -> Result<Json<crate::runtime::contracts::ActionResult>, (StatusCode, String)> {
    let route_name = action_route_name(&action_name)?;
    let invocation = action_invocation_from_request(&state, &action_name, input)?;
    let response = dispatch_action(&state, route_name, invocation).await?;
    let RuntimeResponse::Action(result) = response else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "unexpected runtime action response".to_string(),
        ));
    };

    log_action_result(&result);
    Ok(Json(result))
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

fn runtime_error(err: RuntimeError) -> (StatusCode, String) {
    (err.status(), err.message().to_string())
}

fn query_params_to_view_params(
    query: HashMap<String, String>,
) -> Result<Map<String, Value>, (StatusCode, String)> {
    let mut params = Map::new();
    for (key, value) in query {
        let parsed = if let Ok(integer) = value.parse::<i64>() {
            Value::Number(integer.into())
        } else if value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("false") {
            Value::Bool(value.eq_ignore_ascii_case("true"))
        } else {
            Value::String(value)
        };
        params.insert(key, parsed);
    }
    Ok(params)
}

async fn dispatch_action(
    state: &AppState,
    route_name: &str,
    invocation: ActionInvocation,
) -> Result<RuntimeResponse, (StatusCode, String)> {
    state
        .runtime
        .dispatch(route_name, RuntimeRequest::Action(invocation))
        .await
        .map_err(runtime_error)
}

fn action_route_name(action_name: &str) -> Result<&'static str, (StatusCode, String)> {
    match action_name {
        "inventory.item.create" => Ok("api.items.create"),
        "inventory.item.update" => Ok("api.items.update"),
        "inventory.item.delete" => Ok("api.items.delete"),
        other => Err((
            StatusCode::NOT_FOUND,
            format!("unknown runtime action '{other}'"),
        )),
    }
}

fn action_invocation_from_request(
    state: &AppState,
    action_name: &str,
    input: RuntimeActionRequest,
) -> Result<ActionInvocation, (StatusCode, String)> {
    match action_name {
        "inventory.item.create" => Ok(ActionInvocation::CreateItem {
            context: state.runtime.request_context(),
            name: required_text_field(&input.fields, "name")?,
            category: required_text_field(&input.fields, "category")?,
            quantity: required_i64_field(&input.fields, "quantity")?,
        }),
        "inventory.item.update" => Ok(ActionInvocation::UpdateItem {
            context: state.runtime.request_context(),
            id: input
                .target_id
                .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing target_id".to_string()))?,
            name: required_text_field(&input.fields, "name")?,
            category: required_text_field(&input.fields, "category")?,
            quantity: required_i64_field(&input.fields, "quantity")?,
        }),
        "inventory.item.delete" => Ok(ActionInvocation::DeleteItem {
            context: state.runtime.request_context(),
            id: input
                .target_id
                .ok_or_else(|| (StatusCode::BAD_REQUEST, "missing target_id".to_string()))?,
        }),
        other => Err((
            StatusCode::NOT_FOUND,
            format!("unknown runtime action '{other}'"),
        )),
    }
}

fn required_text_field(
    fields: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<String, (StatusCode, String)> {
    fields
        .get(key)
        .and_then(serde_json::Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("missing text field '{key}'")))
}

fn required_i64_field(
    fields: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Result<i64, (StatusCode, String)> {
    fields
        .get(key)
        .and_then(serde_json::Value::as_i64)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("missing integer field '{key}'")))
}

fn log_action_result(result: &crate::runtime::contracts::ActionResult) {
    match &result.outcome {
        ActionOutcome::Item(_) => {
            info!(
                action = result.definition.name,
                event_type = result.event.event_type,
                entity_id = %result.event.entity_id,
                "completed runtime action"
            );
        }
        ActionOutcome::Deleted { id: deleted_id } => {
            info!(
                action = result.definition.name,
                event_type = result.event.event_type,
                entity_id = %result.event.entity_id,
                deleted_id,
                "completed runtime action"
            );
        }
    }
}
