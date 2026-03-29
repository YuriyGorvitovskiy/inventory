use std::path::Path;

use sqlx::PgPool;
use tokio::sync::broadcast;

use crate::model::ModelRegistry;
use crate::models::Item;
use crate::runtime::contracts::{
    ContextQuery, ContextQueryResult, EventCandidate, EventEnvelope, InventoryItemRecord, PatchEnvelope,
    PatchOperation, RequestContext, RuntimeError,
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
                let items = sqlx::query_as::<_, Item>(
                    r#"
                    SELECT id, owner_service, entity_type, name, category, quantity
                    FROM inventory_items
                    ORDER BY id
                    "#,
                )
                .fetch_all(&self.db)
                .await?;

                Ok(ContextQueryResult::InventoryItems(
                    items
                        .into_iter()
                        .map(|item| item_record_from_row(context, item))
                        .collect(),
                ))
            }
            ContextQuery::InventoryItemById(id) => {
                let item = sqlx::query_as::<_, Item>(
                    r#"
                    SELECT id, owner_service, entity_type, name, category, quantity
                    FROM inventory_items
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .fetch_optional(&self.db)
                .await?;

                Ok(ContextQueryResult::InventoryItem(
                    item.map(|row| item_record_from_row(context, row)),
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
        if patch.kind != "patch_envelope" {
            return Err(RuntimeError::internal("unsupported patch envelope kind"));
        }

        let patched_item = match patch.operation {
            PatchOperation::CreateItem {
                name,
                category,
                quantity,
            } => {
                let item = sqlx::query_as::<_, Item>(
                    r#"
                    INSERT INTO inventory_items (owner_service, entity_type, name, category, quantity)
                    VALUES ($1, $2, $3, $4, $5)
                    RETURNING id, owner_service, entity_type, name, category, quantity
                    "#,
                )
                .bind("inventory-core")
                .bind("item")
                .bind(name)
                .bind(category)
                .bind(quantity)
                .fetch_one(&self.db)
                .await?;

                Some(item_record_from_row(context, item))
            }
            PatchOperation::UpdateItem {
                id,
                name,
                category,
                quantity,
            } => {
                let item = sqlx::query_as::<_, Item>(
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
                .bind(name)
                .bind(category)
                .bind(quantity)
                .bind(id)
                .fetch_optional(&self.db)
                .await?;

                match item {
                    Some(item) => Some(item_record_from_row(context, item)),
                    None => return Err(RuntimeError::not_found("item not found")),
                }
            }
            PatchOperation::DeleteItem { id } => {
                let result = sqlx::query(
                    r#"
                    DELETE FROM inventory_items
                    WHERE id = $1
                    "#,
                )
                .bind(id)
                .execute(&self.db)
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
            kind: "event_envelope",
            version: patch.version,
            event_type: event.event_type,
            entity_id,
            entity_type: event.entity_type,
            tenant_id: context.tenant_id.clone(),
        };
        self.event_stream.publish(envelope.clone());

        Ok((patched_item, envelope))
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
