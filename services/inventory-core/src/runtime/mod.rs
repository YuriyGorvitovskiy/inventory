pub mod business;
pub mod contracts;
pub mod data;
pub mod events;
pub mod registry;
pub mod ui;

use std::path::Path;

use crate::runtime::business::{BusinessLayer, InventoryBusinessLayer};
use crate::runtime::contracts::{
    health_response, parsed_models_to_response, readiness_response, ActionInvocation, ActionResult,
    RequestContext, ResolvedItemListView, RuntimeError, RuntimeModelApiResponse,
};
use crate::runtime::data::{DataLayer, InventoryDataLayer};
use crate::runtime::events::InProcessEventStream;
use crate::runtime::registry::{DefinitionRegistry, RouteCatalog};
use crate::runtime::ui::{InventoryUiLayer, UiLayer};
use crate::{
    models::{HealthResponse, ItemResponse, ReadinessResponse},
    ui::INDEX_HTML,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct CoLocatedRuntime<U = InventoryUiLayer, B = InventoryBusinessLayer, D = InventoryDataLayer> {
    definitions: DefinitionRegistry,
    routes: RouteCatalog,
    ui: U,
    business: B,
    data: D,
}

impl CoLocatedRuntime<InventoryUiLayer, InventoryBusinessLayer, InventoryDataLayer> {
    pub fn load(db: PgPool, tenant_id: String, model_dir: &Path) -> Result<Self, RuntimeError> {
        let event_stream = InProcessEventStream::new(128);
        let data = InventoryDataLayer::load(db, tenant_id, model_dir, event_stream)?;
        let definitions = DefinitionRegistry::load_from_dir(model_dir)?;
        let routes = RouteCatalog::load_from_dir(model_dir)?;
        Ok(Self {
            definitions,
            routes,
            ui: InventoryUiLayer::default(),
            business: InventoryBusinessLayer::default(),
            data,
        })
    }
}

impl<U, B, D> CoLocatedRuntime<U, B, D>
where
    U: UiLayer<B, D>,
    B: BusinessLayer<D>,
    D: DataLayer,
{
    pub fn model_count(&self) -> usize {
        self.data.model_registry().len()
    }

    pub fn health(&self) -> HealthResponse {
        health_response()
    }

    pub fn readiness(&self) -> ReadinessResponse {
        readiness_response()
    }

    pub fn index_html(&self) -> &'static str {
        INDEX_HTML
    }

    pub fn request_context(&self) -> RequestContext {
        self.data.request_context()
    }

    pub async fn resolve_items_view(&self) -> Result<ResolvedItemListView, RuntimeError> {
        self.ui
            .resolve_items_view(
                self.data.request_context(),
                &self.definitions,
                &self.business,
                &self.data,
            )
            .await
    }

    pub async fn invoke_action(
        &self,
        invocation: ActionInvocation,
    ) -> Result<ActionResult, RuntimeError> {
        self.ui
            .invoke_action(invocation, &self.definitions, &self.business, &self.data)
            .await
    }

    pub fn models(&self) -> &crate::model::ModelRegistry {
        self.data.model_registry()
    }

    pub fn describe_models(&self) -> RuntimeModelApiResponse {
        let mut names: Vec<_> = self.models().entities().collect();
        names.sort_unstable();

        let parsed_models = names
            .into_iter()
            .filter_map(|name| self.models().get(name))
            .collect();

        parsed_models_to_response(parsed_models)
    }

    pub async fn list_items(&self) -> Result<Vec<ItemResponse>, RuntimeError> {
        let view = self.resolve_items_view().await?;
        Ok(view.rows.into_iter().map(Into::into).collect())
    }

    pub async fn dispatch(
        &self,
        route_name: &str,
        request: RuntimeRequest,
    ) -> Result<RuntimeResponse, RuntimeError> {
        let route = self.routes.route(route_name)?;
        let _route_kind = route.kind;
        let _route_version = route.version;

        match route.target.as_str() {
            "health" => Ok(RuntimeResponse::Health(self.health())),
            "ready" => Ok(RuntimeResponse::Ready(self.readiness())),
            "index" => Ok(RuntimeResponse::IndexHtml(self.index_html())),
            "model.describe" => Ok(RuntimeResponse::Model(self.describe_models())),
            "view.inventory.item.list" => Ok(RuntimeResponse::ItemsView(self.resolve_items_view().await?)),
            "items.list" => Ok(RuntimeResponse::Items(self.list_items().await?)),
            "action.inventory.item.create"
            | "action.inventory.item.update"
            | "action.inventory.item.delete" => match request {
                RuntimeRequest::Action(invocation) => {
                    Ok(RuntimeResponse::Action(self.invoke_action(invocation).await?))
                }
                RuntimeRequest::Empty => Err(RuntimeError::bad_request(format!(
                    "route '{route_name}' requires an action invocation"
                ))),
            },
            other => Err(RuntimeError::internal(format!(
                "unsupported runtime route target '{other}'"
            ))),
        }
    }

    pub fn subscribe_events(&self) -> tokio::sync::broadcast::Receiver<contracts::EventEnvelope> {
        self.data.subscribe_events()
    }
}

pub enum RuntimeRequest {
    Empty,
    Action(ActionInvocation),
}

pub enum RuntimeResponse {
    Health(HealthResponse),
    Ready(ReadinessResponse),
    IndexHtml(&'static str),
    Model(RuntimeModelApiResponse),
    ItemsView(ResolvedItemListView),
    Items(Vec<ItemResponse>),
    Action(ActionResult),
}

#[cfg(test)]
mod tests;
