use std::path::Path;

use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::model::ModelRegistry;
use crate::models::Item;
use crate::runtime::contracts::{
    ContextQuery, ContextQueryResult, EventCandidate, EventEnvelope, InventoryItemRecord, PatchEnvelope,
    PatchOperation, ProjectionQuery, ProjectionResult, RequestContext, RuntimeError, OWNED_ENTITY_TYPE,
    OWNED_SERVICE,
};
use crate::runtime::events::InProcessEventStream;

pub trait DataLayer {
    fn request_context(&self) -> RequestContext;
    fn model_registry(&self) -> &ModelRegistry;
    fn subscribe_events(&self) -> broadcast::Receiver<EventEnvelope>;
    async fn execute_context_query(
        &self,
        context: &RequestContext,
        query: ContextQuery,
    ) -> Result<ContextQueryResult, RuntimeError>;
    async fn load_projection(
        &self,
        context: &RequestContext,
        query: ProjectionQuery,
    ) -> Result<ProjectionResult, RuntimeError>;
    async fn apply_patch(
        &self,
        context: &RequestContext,
        patch: PatchEnvelope,
        event: EventCandidate,
    ) -> Result<(Option<InventoryItemRecord>, EventEnvelope), RuntimeError>;
}

#[derive(Clone)]
pub struct InventoryDataLayer {
    db: PgPool,
    context: RequestContext,
    model_registry: ModelRegistry,
    event_stream: InProcessEventStream,
}

impl InventoryDataLayer {
    pub fn load(
        db: PgPool,
        tenant_id: String,
        model_dir: &Path,
        event_stream: InProcessEventStream,
    ) -> Result<Self, RuntimeError> {
        let model_registry = ModelRegistry::load_from_dir(model_dir)?;
        Ok(Self {
            db,
            context: RequestContext { tenant_id },
            model_registry,
            event_stream,
        })
    }
}

impl DataLayer for InventoryDataLayer {
    fn request_context(&self) -> RequestContext {
        self.context.clone()
    }

    fn model_registry(&self) -> &ModelRegistry {
        &self.model_registry
    }

    fn subscribe_events(&self) -> broadcast::Receiver<EventEnvelope> {
        self.event_stream.subscribe()
    }

    async fn execute_context_query(
        &self,
        context: &RequestContext,
        query: ContextQuery,
    ) -> Result<ContextQueryResult, RuntimeError> {
        match query {
            ContextQuery::InventoryItems => {
                let ProjectionResult::InventoryItems(items) =
                    self.load_projection(context, ProjectionQuery::InventoryItemList).await?;
                Ok(ContextQueryResult::InventoryItems(items))
            }
            ContextQuery::InventoryItemById(id) => {
                let item_table = self.item_table_name()?;
                let item = sqlx::query_as::<_, Item>(&format!(
                    r#"
                    SELECT id, owner_service, entity_type, name, category, quantity
                    FROM {item_table}
                    WHERE id = $1
                      AND owner_service = $2
                      AND entity_type = $3
                    "#,
                ))
                .bind(id)
                .bind(OWNED_SERVICE)
                .bind(OWNED_ENTITY_TYPE)
                .fetch_optional(&self.db)
                .await?;

                Ok(ContextQueryResult::InventoryItem(
                    item.map(|row| item_record_from_row(context, row)),
                ))
            }
        }
    }

    async fn load_projection(
        &self,
        context: &RequestContext,
        query: ProjectionQuery,
    ) -> Result<ProjectionResult, RuntimeError> {
        let item_table = self.item_table_name()?;
        match query {
            ProjectionQuery::InventoryItemList => {
                let items = sqlx::query_as::<_, Item>(&format!(
                    r#"
                    SELECT id, owner_service, entity_type, name, category, quantity
                    FROM {item_table}
                    WHERE owner_service = $1
                      AND entity_type = $2
                    ORDER BY id
                    "#,
                ))
                .bind(OWNED_SERVICE)
                .bind(OWNED_ENTITY_TYPE)
                .fetch_all(&self.db)
                .await?;

                Ok(ProjectionResult::InventoryItems(
                    items
                        .into_iter()
                        .map(|item| item_record_from_row(context, item))
                        .collect(),
                ))
            }
        }
    }

    async fn apply_patch(
        &self,
        context: &RequestContext,
        patch: PatchEnvelope,
        event: EventCandidate,
    ) -> Result<(Option<InventoryItemRecord>, EventEnvelope), RuntimeError> {
        if patch.kind != "patch" {
            return Err(RuntimeError::internal("unsupported patch envelope kind"));
        }
        if patch.tenant_id != context.tenant_id {
            return Err(RuntimeError::bad_request("patch tenant does not match request context"));
        }
        if patch.target.service != OWNED_SERVICE || patch.target.entity_type != OWNED_ENTITY_TYPE {
            return Err(RuntimeError::bad_request("patch target is not owned by inventory-core"));
        }

        let item_table = self.item_table_name()?;
        if patch.target.table != item_table {
            return Err(RuntimeError::bad_request("patch target table does not match active mapping"));
        }

        let mut tx = self.db.begin().await?;

        let quantity_mode = self.quantity_conflict_mode()?;

        let patched_item = match patch.operation {
            PatchOperation::CreateItem {
                name,
                category,
                quantity,
            } => {
                let item = sqlx::query_as::<_, Item>(&format!(
                    r#"
                    INSERT INTO {item_table} (owner_service, entity_type, name, category, quantity)
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING id, owner_service, entity_type, name, category, quantity
                    "#,
                ))
                .bind(OWNED_SERVICE)
                .bind(OWNED_ENTITY_TYPE)
                .bind(name)
                .bind(category)
                .bind(quantity)
                .fetch_one(&mut *tx)
                .await?;

                Some(item_record_from_row(context, item))
            }
            PatchOperation::UpdateItem {
                id,
                name,
                category,
                quantity,
                quantity_delta,
            } => {
                let item = match quantity_mode {
                    crate::model::model::ConflictResolutionMode::Increment => {
                        sqlx::query_as::<_, Item>(&format!(
                            r#"
                            UPDATE {item_table}
                            SET name = $1,
                                category = $2,
                                quantity = quantity + $3,
                                updated_at = NOW()
                            WHERE id = $4
                              AND owner_service = $5
                              AND entity_type = $6
                            RETURNING id, owner_service, entity_type, name, category, quantity
                            "#,
                        ))
                        .bind(name)
                        .bind(category)
                        .bind(quantity_delta)
                        .bind(id)
                        .bind(OWNED_SERVICE)
                        .bind(OWNED_ENTITY_TYPE)
                        .fetch_optional(&mut *tx)
                        .await?
                    }
                    _ => {
                        sqlx::query_as::<_, Item>(&format!(
                            r#"
                            UPDATE {item_table}
                            SET name = $1,
                                category = $2,
                                quantity = $3,
                                updated_at = NOW()
                            WHERE id = $4
                              AND owner_service = $5
                              AND entity_type = $6
                            RETURNING id, owner_service, entity_type, name, category, quantity
                            "#,
                        ))
                        .bind(name)
                        .bind(category)
                        .bind(quantity)
                        .bind(id)
                        .bind(OWNED_SERVICE)
                        .bind(OWNED_ENTITY_TYPE)
                        .fetch_optional(&mut *tx)
                        .await?
                    }
                };

                match item {
                    Some(item) => Some(item_record_from_row(context, item)),
                    None => return Err(RuntimeError::not_found("item not found")),
                }
            }
            PatchOperation::DeleteItem { id } => {
                if patch.target.id != Some(id) {
                    return Err(RuntimeError::bad_request("patch target row id does not match delete operation"));
                }

                let result = sqlx::query(&format!(
                    r#"
                    DELETE FROM {item_table}
                    WHERE id = $1
                      AND owner_service = $2
                      AND entity_type = $3
                    "#,
                ))
                .bind(id)
                .bind(OWNED_SERVICE)
                .bind(OWNED_ENTITY_TYPE)
                .execute(&mut *tx)
                .await?;

                if result.rows_affected() == 0 {
                    return Err(RuntimeError::not_found("item not found"));
                }

                None
            }
        };

        let entity_id = patched_item
            .as_ref()
            .map(|item| item.entity_id.clone())
            .unwrap_or_else(|| {
                let row_id = event.entity_id_hint.unwrap_or_default();
                format!("{}.inventory-core.{}.{}", context.tenant_id, event.entity_type, row_id)
            });

        let envelope = EventEnvelope {
            kind: "event",
            version: patch.version,
            patch_id: patch.patch_id,
            event_type: event.event_type,
            entity_id,
            entity_type: event.entity_type,
            service: OWNED_SERVICE,
            tenant_id: context.tenant_id.clone(),
            action_name: patch.causation.action_name,
            payload: event_payload(&patched_item),
        };

        tx.commit().await?;
        self.event_stream.publish(envelope.clone());

        Ok((patched_item, envelope))
    }
}

impl InventoryDataLayer {
    fn item_table_name(&self) -> Result<String, RuntimeError> {
        self.model_registry
            .get(OWNED_ENTITY_TYPE)
            .map(|parsed| parsed.mapping.table_name.clone())
            .ok_or_else(|| RuntimeError::internal("item model mapping is not loaded"))
    }

    fn quantity_conflict_mode(&self) -> Result<crate::model::model::ConflictResolutionMode, RuntimeError> {
        self.model_registry
            .get(OWNED_ENTITY_TYPE)
            .and_then(|parsed| parsed.model.entity.fields.iter().find(|field| field.name == "quantity"))
            .map(|field| field.conflict_resolution.mode)
            .ok_or_else(|| RuntimeError::internal("quantity field metadata is not loaded"))
    }
}

fn item_record_from_row(context: &RequestContext, item: Item) -> InventoryItemRecord {
    InventoryItemRecord {
        id: item.id,
        entity_id: format!(
            "{}.{}.{}.{}",
            context.tenant_id, item.owner_service, item.entity_type, item.id
        ),
        name: item.name,
        category: item.category,
        quantity: item.quantity,
    }
}

fn event_payload(item: &Option<InventoryItemRecord>) -> serde_json::Value {
    match item {
        Some(item) => serde_json::json!({
            "id": item.id,
            "entity_id": item.entity_id,
            "name": item.name,
            "category": item.category,
            "quantity": item.quantity,
        }),
        None => serde_json::json!({ "deleted": true }),
    }
}
