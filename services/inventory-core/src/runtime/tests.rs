use crate::model::ModelRegistry;
use crate::runtime::business::{BusinessLayer, InventoryBusinessLayer};
use crate::runtime::contracts::{
    ActionInvocation, ContextQuery, ContextQueryResult, EventCandidate, EventEnvelope,
    InventoryItemRecord, PatchEnvelope, ProjectionQuery, ProjectionResult, RequestContext,
    RuntimeError, OWNED_ENTITY_TYPE, OWNED_SERVICE,
};
use crate::runtime::data::DataLayer;
use crate::runtime::events::InProcessEventStream;
use crate::runtime::registry::DefinitionRegistry;
use crate::runtime::ui::{InventoryUiLayer, UiLayer};
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(1);

#[derive(Clone)]
struct FakeDataLayer {
    context: RequestContext,
    model_registry: ModelRegistry,
    items: Vec<InventoryItemRecord>,
    applied_patches: Arc<Mutex<Vec<PatchEnvelope>>>,
}

impl FakeDataLayer {
    fn new(items: Vec<InventoryItemRecord>) -> Self {
        let dir = unique_temp_dir("inventory-runtime-test-models");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir should exist");
        fs::write(
            dir.join("item.model.toml"),
            r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
table = "inventory_items"
fields = [
  { name = "name", type = "label", required = true, default = "", indexed = true },
  { name = "category", type = "label", required = true, default = "" },
  { name = "quantity", type = "integer", required = true, default = 0, conflict_resolution = { mode = "increment" } }
]
"#,
        )
        .expect("model should be written");

        let model_registry =
            ModelRegistry::load_from_dir(&dir).expect("model registry should load");
        let _ = fs::remove_dir_all(&dir);

        Self {
            context: RequestContext {
                tenant_id: "tenant-test".to_string(),
            },
            model_registry,
            items,
            applied_patches: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl DataLayer for FakeDataLayer {
    fn request_context(&self) -> RequestContext {
        self.context.clone()
    }

    fn model_registry(&self) -> &ModelRegistry {
        &self.model_registry
    }

    fn subscribe_events(&self) -> broadcast::Receiver<EventEnvelope> {
        InProcessEventStream::new(4).subscribe()
    }

    async fn execute_context_query(
        &self,
        _context: &RequestContext,
        query: ContextQuery,
    ) -> Result<ContextQueryResult, RuntimeError> {
        match query {
            ContextQuery::InventoryItems => {
                Ok(ContextQueryResult::InventoryItems(self.items.clone()))
            }
            ContextQuery::InventoryItemById(id) => Ok(ContextQueryResult::InventoryItem(
                self.items.iter().find(|item| item.id == id).cloned(),
            )),
        }
    }

    async fn load_projection(
        &self,
        _context: &RequestContext,
        query: ProjectionQuery,
    ) -> Result<ProjectionResult, RuntimeError> {
        match query {
            ProjectionQuery::InventoryItemList => {
                Ok(ProjectionResult::InventoryItems(self.items.clone()))
            }
        }
    }

    async fn apply_patch(
        &self,
        context: &RequestContext,
        patch: PatchEnvelope,
        event: EventCandidate,
    ) -> Result<(Option<InventoryItemRecord>, EventEnvelope), RuntimeError> {
        self.applied_patches
            .lock()
            .expect("patch log should lock")
            .push(patch.clone());

        let item = InventoryItemRecord {
            id: event.entity_id_hint.unwrap_or(42),
            entity_id: format!(
                "{}.inventory-core.item.{}",
                context.tenant_id,
                event.entity_id_hint.unwrap_or(42)
            ),
            name: match patch.operation {
                crate::runtime::contracts::PatchOperation::CreateItem { ref name, .. }
                | crate::runtime::contracts::PatchOperation::UpdateItem { ref name, .. } => {
                    name.clone()
                }
                crate::runtime::contracts::PatchOperation::DeleteItem { .. } => {
                    "deleted".to_string()
                }
            },
            category: match patch.operation {
                crate::runtime::contracts::PatchOperation::CreateItem { ref category, .. }
                | crate::runtime::contracts::PatchOperation::UpdateItem { ref category, .. } => {
                    category.clone()
                }
                crate::runtime::contracts::PatchOperation::DeleteItem { .. } => {
                    "deleted".to_string()
                }
            },
            quantity: 1,
        };

        let maybe_item = match patch.operation {
            crate::runtime::contracts::PatchOperation::DeleteItem { .. } => None,
            _ => Some(item.clone()),
        };

        Ok((
            maybe_item,
            EventEnvelope {
                kind: "event",
                version: patch.version,
                patch_id: patch.patch_id,
                event_type: event.event_type,
                entity_id: item.entity_id,
                entity_type: event.entity_type,
                service: OWNED_SERVICE,
                tenant_id: context.tenant_id.clone(),
                action_name: patch.causation.action_name,
                payload: serde_json::json!({
                    "name": item.name,
                    "category": item.category,
                    "quantity": item.quantity,
                }),
            },
        ))
    }
}

fn write_runtime_definitions(dir: &std::path::Path) {
    fs::write(
        dir.join("inventory.items.query.toml"),
        r#"
kind = "context_query_definition"
version = "1.0.0"
name = "inventory.items"
root_entity = "item"
description = "List items"
"#,
    )
    .expect("query definition should be written");
    fs::write(
        dir.join("inventory.item.by_id.query.toml"),
        r#"
kind = "context_query_definition"
version = "1.0.0"
name = "inventory.item.by_id"
root_entity = "item"
description = "Item by id"
"#,
    )
    .expect("query definition should be written");
    fs::write(
        dir.join("inventory.item.create.action.toml"),
        r#"
kind = "action_definition"
version = "1.0.0"
name = "inventory.item.create"
description = "Create item"
context_queries = ["inventory.items"]
"#,
    )
    .expect("action definition should be written");
    fs::write(
        dir.join("inventory.item.update.action.toml"),
        r#"
kind = "action_definition"
version = "1.0.0"
name = "inventory.item.update"
description = "Update item"
context_queries = ["inventory.item.by_id"]
"#,
    )
    .expect("action definition should be written");
    fs::write(
        dir.join("inventory.item.delete.action.toml"),
        r#"
kind = "action_definition"
version = "1.0.0"
name = "inventory.item.delete"
description = "Delete item"
context_queries = ["inventory.item.by_id"]
"#,
    )
    .expect("action definition should be written");
    fs::write(
        dir.join("inventory.item.list.view.toml"),
        r#"
kind = "view_definition"
version = "1.0.0"
name = "inventory.item.list"
entity_scope = "item"
params = []
context_queries = [{ query = "inventory.items", bind = "items" }]
layout = { type = "page", title = "Inventory", children = [
  { type = "action_bar", actions = ["inventory.item.create"] },
  { type = "table", rows = { bind = "$context.items.rows" }, columns = [
    { key = "name", header = "Name", value = { bind = "$row.name" }, editable = true, editor_kind = "label" },
    { key = "category", header = "Category", value = { bind = "$row.category" }, editable = true, editor_kind = "label" },
    { key = "quantity", header = "Quantity", value = { bind = "$row.quantity" }, editable = true, editor_kind = "integer" }
  ] }
] }
interactions = [
  { event = "create", action = "inventory.item.create" },
  { event = "update", route_to = "inventory.item.editor", params = [{ name = "id", value = { bind = "$row.id" } }] },
  { event = "delete", action = "inventory.item.delete" }
]
"#,
    )
    .expect("view definition should be written");
    fs::write(
        dir.join("inventory.item.editor.view.toml"),
        r#"
kind = "view_definition"
version = "1.0.0"
name = "inventory.item.editor"
entity_scope = "item"
params = [{ name = "id", type = "integer", required = true }]
context_queries = [{ query = "inventory.item.by_id", bind = "item" }]
layout = { type = "page", title = "Edit Item", children = [
  { type = "action_bar", actions = [] },
  { type = "text", value = { bind = "$context.item.item.entity_id" } },
  { type = "form", fields = [
    { key = "name", label = "Name", value = { bind = "$context.item.item.name" }, editor_kind = "label", editable = true, required = true },
    { key = "category", label = "Category", value = { bind = "$context.item.item.category" }, editor_kind = "label", editable = true, required = true },
    { key = "quantity", label = "Quantity", value = { bind = "$context.item.item.quantity" }, editor_kind = "integer", editable = true, required = true }
  ] }
] }
interactions = [
  { event = "save", action = "inventory.item.update" },
  { event = "delete", action = "inventory.item.delete" },
  { event = "back", route_to = "inventory.item.list" }
]
"#,
    )
    .expect("editor view definition should be written");
}

fn test_definition_registry() -> DefinitionRegistry {
    let dir = unique_temp_dir("inventory-runtime-test-definitions");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir should exist");
    write_runtime_definitions(&dir);
    let registry =
        DefinitionRegistry::load_from_dir(&dir).expect("definition registry should load");
    let _ = fs::remove_dir_all(&dir);
    registry
}

fn unique_temp_dir(prefix: &str) -> std::path::PathBuf {
    let id = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}-{}-{id}", std::process::id()))
}

#[tokio::test]
async fn ui_normalizes_item_input_before_business_execution() {
    let ui = InventoryUiLayer::default();
    let business = InventoryBusinessLayer::default();
    let data = FakeDataLayer::new(vec![]);
    let definitions = test_definition_registry();

    let result = ui
        .invoke_action(
            ActionInvocation::CreateItem {
                context: data.request_context(),
                name: "  Milk  ".to_string(),
                category: "  Dairy  ".to_string(),
                quantity: 1,
            },
            &definitions,
            &business,
            &data,
        )
        .await
        .expect("action should succeed");

    match result.outcome {
        crate::runtime::contracts::ActionOutcome::Item(item) => {
            assert_eq!(item.name, "Milk");
            assert_eq!(item.category, "Dairy");
        }
        crate::runtime::contracts::ActionOutcome::Deleted { .. } => {
            panic!("expected create action to return item outcome")
        }
    }
}

#[tokio::test]
async fn business_rejects_duplicate_item_with_normalized_input() {
    let business = InventoryBusinessLayer::default();
    let data = FakeDataLayer::new(vec![InventoryItemRecord {
        id: 1,
        entity_id: "tenant-test.inventory-core.item.1".to_string(),
        name: "Milk".to_string(),
        category: "Dairy".to_string(),
        quantity: 1,
    }]);
    let definitions = test_definition_registry();

    let err = business
        .execute_action(
            crate::runtime::contracts::NormalizedActionInvocation::CreateItem {
                context: data.request_context(),
                name: "Milk".to_string(),
                category: "Dairy".to_string(),
                quantity: 1,
            },
            &definitions,
            &data,
        )
        .await
        .expect_err("duplicate item should fail");

    assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    assert!(err.message().contains("same name"));
}

#[tokio::test]
async fn ui_resolves_view_with_registry_backed_definition_and_framework_widgets() {
    let ui = InventoryUiLayer::default();
    let data = FakeDataLayer::new(vec![InventoryItemRecord {
        id: 7,
        entity_id: "tenant-test.inventory-core.item.7".to_string(),
        name: "Soap".to_string(),
        category: "Cleaning".to_string(),
        quantity: 2,
    }]);
    let definitions = test_definition_registry();

    let view = <InventoryUiLayer as UiLayer<InventoryBusinessLayer, FakeDataLayer>>::resolve_view(
        &ui,
        "inventory.item.list",
        serde_json::Map::new(),
        data.request_context(),
        &definitions,
        &data,
    )
    .await
    .expect("view should resolve");

    assert_eq!(view.definition.name, "inventory.item.list");
    assert!(view.context.contains_key("items"));

    let crate::runtime::contracts::ResolvedFrameworkWidget::Page { title, children } = view.widget
    else {
        panic!("expected resolved page widget");
    };
    assert_eq!(title, "Inventory");
    assert_eq!(children.len(), 2);

    let crate::runtime::contracts::ResolvedFrameworkWidget::Table { rows, .. } = &children[1]
    else {
        panic!("expected resolved table widget");
    };
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].cells["name"], serde_json::json!("Soap"));
}

#[tokio::test]
async fn ui_resolves_parameterized_editor_view_from_context_item() {
    let ui = InventoryUiLayer::default();
    let data = FakeDataLayer::new(vec![InventoryItemRecord {
        id: 7,
        entity_id: "tenant-test.inventory-core.item.7".to_string(),
        name: "Soap".to_string(),
        category: "Cleaning".to_string(),
        quantity: 2,
    }]);
    let definitions = test_definition_registry();

    let mut params = serde_json::Map::new();
    params.insert("id".to_string(), serde_json::json!(7));

    let view = <InventoryUiLayer as UiLayer<InventoryBusinessLayer, FakeDataLayer>>::resolve_view(
        &ui,
        "inventory.item.editor",
        params,
        data.request_context(),
        &definitions,
        &data,
    )
    .await
    .expect("editor view should resolve");

    assert_eq!(view.definition.name, "inventory.item.editor");
    assert_eq!(view.context["item"]["item"]["name"], serde_json::json!("Soap"));
    assert_eq!(view.context["item"]["rows"][0]["id"], serde_json::json!(7));

    let crate::runtime::contracts::ResolvedFrameworkWidget::Page { children, .. } = view.widget else {
        panic!("expected page widget");
    };
    let crate::runtime::contracts::ResolvedFrameworkWidget::Form { fields } = &children[2] else {
        panic!("expected form widget");
    };
    assert_eq!(fields[0].key, "name");
    assert_eq!(fields[0].value, serde_json::json!("Soap"));
}

#[tokio::test]
async fn ui_rejects_missing_required_editor_view_param() {
    let ui = InventoryUiLayer::default();
    let data = FakeDataLayer::new(vec![]);
    let definitions = test_definition_registry();

    let err = <InventoryUiLayer as UiLayer<InventoryBusinessLayer, FakeDataLayer>>::resolve_view(
        &ui,
        "inventory.item.editor",
        serde_json::Map::new(),
        data.request_context(),
        &definitions,
        &data,
    )
    .await
    .expect_err("missing id param should fail");

    assert_eq!(err.status(), axum::http::StatusCode::BAD_REQUEST);
    assert!(err.message().contains("missing required view param 'id'"));
}

#[tokio::test]
async fn business_builds_patch_against_owned_mapping_and_returns_committed_event_metadata() {
    let business = InventoryBusinessLayer::default();
    let data = FakeDataLayer::new(vec![]);
    let definitions = test_definition_registry();

    let result = business
        .execute_action(
            crate::runtime::contracts::NormalizedActionInvocation::CreateItem {
                context: data.request_context(),
                name: "Milk".to_string(),
                category: "Dairy".to_string(),
                quantity: 2,
            },
            &definitions,
            &data,
        )
        .await
        .expect("create item should succeed");

    let patches = data.applied_patches.lock().expect("patch log should lock");
    assert_eq!(patches.len(), 1);
    assert_eq!(patches[0].kind, "patch");
    assert_eq!(patches[0].tenant_id, "tenant-test");
    assert_eq!(patches[0].target.service, OWNED_SERVICE);
    assert_eq!(patches[0].target.entity_type, OWNED_ENTITY_TYPE);
    assert_eq!(patches[0].target.table, "inventory_items");
    assert_eq!(patches[0].causation.action_name, "inventory.item.create");

    assert_eq!(result.event.kind, "event");
    assert_eq!(result.event.patch_id, patches[0].patch_id);
    assert_eq!(result.event.service, OWNED_SERVICE);
    assert_eq!(result.event.action_name, "inventory.item.create");
    assert_eq!(result.event.payload["name"], "Milk");
}

#[tokio::test]
async fn business_computes_quantity_delta_for_increment_semantics() {
    let business = InventoryBusinessLayer::default();
    let data = FakeDataLayer::new(vec![InventoryItemRecord {
        id: 4,
        entity_id: "tenant-test.inventory-core.item.4".to_string(),
        name: "Milk".to_string(),
        category: "Dairy".to_string(),
        quantity: 5,
    }]);
    let definitions = test_definition_registry();

    let _ = business
        .execute_action(
            crate::runtime::contracts::NormalizedActionInvocation::UpdateItem {
                context: data.request_context(),
                id: 4,
                name: "Milk".to_string(),
                category: "Dairy".to_string(),
                quantity: 7,
            },
            &definitions,
            &data,
        )
        .await
        .expect("update should succeed");

    let patches = data.applied_patches.lock().expect("patch log should lock");
    match &patches[0].operation {
        crate::runtime::contracts::PatchOperation::UpdateItem { quantity_delta, .. } => {
            assert_eq!(*quantity_delta, 2);
        }
        other => panic!("expected update patch, got {other:?}"),
    }
}
