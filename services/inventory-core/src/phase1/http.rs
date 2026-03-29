use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
    Json,
};
use tracing::info;

use crate::{
    models::{CreateItemRequest, ItemResponse, UpdateItemRequest},
    runtime::contracts::{
        ActionInvocation, ActionOutcome, ResolvedItemListView, RuntimeError, RuntimeModelApiResponse,
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
) -> Result<Json<ResolvedItemListView>, (StatusCode, String)> {
    match state
        .runtime
        .dispatch("api.items.view", RuntimeRequest::Empty)
        .await
        .map_err(runtime_error)?
    {
        RuntimeResponse::ItemsView(view) => Ok(Json(view)),
        _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "unexpected runtime view response".to_string())),
    }
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
        _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "unexpected runtime list response".to_string())),
    }
}

pub async fn create_item(
    State(state): State<AppState>,
    Json(input): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<ItemResponse>), (StatusCode, String)> {
    let response = state
        .runtime
        .dispatch(
            "api.items.create",
            RuntimeRequest::Action(ActionInvocation::CreateItem {
                context: state.runtime.request_context(),
                name: input.name,
                category: input.category,
                quantity: input.quantity,
            }),
        )
        .await
        .map_err(runtime_error)?;
    let RuntimeResponse::Action(result) = response else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "unexpected runtime action response".to_string()));
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
    let response = state
        .runtime
        .dispatch(
            "api.items.update",
            RuntimeRequest::Action(ActionInvocation::UpdateItem {
                context: state.runtime.request_context(),
                id,
                name: input.name,
                category: input.category,
                quantity: input.quantity,
            }),
        )
        .await
        .map_err(runtime_error)?;
    let RuntimeResponse::Action(result) = response else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "unexpected runtime action response".to_string()));
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
    let response = state
        .runtime
        .dispatch(
            "api.items.delete",
            RuntimeRequest::Action(ActionInvocation::DeleteItem {
                context: state.runtime.request_context(),
                id,
            }),
        )
        .await
        .map_err(runtime_error)?;
    let RuntimeResponse::Action(result) = response else {
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "unexpected runtime action response".to_string()));
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
