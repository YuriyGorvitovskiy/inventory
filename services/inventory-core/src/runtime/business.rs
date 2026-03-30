use crate::runtime::contracts::{
    ActionOutcome, ActionPlan, ActionResult, ContextQuery, ContextQueryResult, EventCandidate,
    NormalizedActionInvocation, PatchCausation, PatchEnvelope, PatchOperation, PatchTarget,
    ProjectionQuery, ProjectionResult, RequestContext, ResolvedItemListView, RuntimeError,
    OWNED_ENTITY_TYPE, OWNED_SERVICE, next_patch_id,
};
use crate::runtime::registry::DefinitionRegistry;
use crate::runtime::data::DataLayer;

#[derive(Default, Clone)]
pub struct InventoryBusinessLayer;

pub trait BusinessLayer<D: DataLayer> {
    async fn resolve_items_view(
        &self,
        context: &RequestContext,
        definitions: &DefinitionRegistry,
        data: &D,
    ) -> Result<ResolvedItemListView, RuntimeError>;

    async fn execute_action(
        &self,
        invocation: NormalizedActionInvocation,
        definitions: &DefinitionRegistry,
        data: &D,
    ) -> Result<ActionResult, RuntimeError>;
}

impl<D: DataLayer> BusinessLayer<D> for InventoryBusinessLayer {
    async fn resolve_items_view(
        &self,
        context: &RequestContext,
        definitions: &DefinitionRegistry,
        data: &D,
    ) -> Result<ResolvedItemListView, RuntimeError> {
        let ProjectionResult::InventoryItems(rows) = data
            .load_projection(context, ProjectionQuery::InventoryItemList)
            .await?;

        let definition = definitions.view("inventory.item.list")?;
        ensure_query_definitions_loaded(definitions, &definition.context_queries)?;

        Ok(ResolvedItemListView {
            definition,
            kind: "view_definition",
            version: "1.0.0",
            name: "inventory.item.list",
            title: "Household Inventory",
            rows,
        })
    }

    async fn execute_action(
        &self,
        invocation: NormalizedActionInvocation,
        definitions: &DefinitionRegistry,
        data: &D,
    ) -> Result<ActionResult, RuntimeError> {
        let context = invocation.context().clone();
        let definition = definitions.action(invocation.definition_name())?;
        ensure_query_definitions_loaded(definitions, &definition.context_queries)?;
        let plan = plan_action(&context, invocation, data).await?;
        let deleted_id = plan_deleted_id(&plan);
        let (item, event) = data.apply_patch(&context, plan.patch, plan.event).await?;

        let outcome = match item {
            Some(item) => ActionOutcome::Item(item),
            None => match deleted_id {
                Some(id) => ActionOutcome::Deleted { id },
                None => return Err(RuntimeError::internal("delete action completed without target id")),
            },
        };

        Ok(ActionResult {
            outcome,
            event,
            definition,
        })
    }
}

fn ensure_query_definitions_loaded(
    definitions: &DefinitionRegistry,
    query_names: &[String],
) -> Result<(), RuntimeError> {
    for query_name in query_names {
        let _ = definitions.query(query_name)?;
    }
    Ok(())
}

async fn plan_action<D: DataLayer>(
    context: &RequestContext,
    invocation: NormalizedActionInvocation,
    data: &D,
) -> Result<ActionPlan, RuntimeError> {
    match invocation {
        NormalizedActionInvocation::CreateItem {
            name,
            category,
            quantity,
            ..
        } => {
            validate_name(&name)?;
            validate_category(&category)?;
            validate_quantity(quantity)?;

            let ContextQueryResult::InventoryItems(existing) = data
                .execute_context_query(context, ContextQuery::InventoryItems)
                .await?
            else {
                return Err(RuntimeError::internal(
                    "unexpected context result for inventory item create",
                ));
            };

            let duplicate = existing.iter().any(|item| {
                item.name.eq_ignore_ascii_case(&name) && item.category.eq_ignore_ascii_case(&category)
            });
            if duplicate {
                return Err(RuntimeError::bad_request(
                    "an item with the same name already exists in this category",
                ));
            }

            Ok(ActionPlan {
                patch: PatchEnvelope {
                    kind: "patch",
                    version: "1.0.0",
                    patch_id: next_patch_id(),
                    tenant_id: context.tenant_id.clone(),
                    target: item_patch_target(data, None)?,
                    causation: PatchCausation {
                        action_name: "inventory.item.create",
                    },
                    operation: PatchOperation::CreateItem {
                        name,
                        category,
                        quantity,
                    },
                },
                event: EventCandidate {
                    event_type: "inventory.item.created",
                    entity_type: "item",
                    entity_id_hint: None,
                },
            })
        }
        NormalizedActionInvocation::UpdateItem {
            id,
            name,
            category,
            quantity,
            ..
        } => {
            validate_name(&name)?;
            validate_category(&category)?;
            validate_quantity(quantity)?;

            let ContextQueryResult::InventoryItem(existing) = data
                .execute_context_query(context, ContextQuery::InventoryItemById(id))
                .await?
            else {
                return Err(RuntimeError::internal(
                    "unexpected context result for inventory item update",
                ));
            };

            if existing.is_none() {
                return Err(RuntimeError::not_found("item not found"));
            }
            let existing = existing.expect("existing item checked above");
            let quantity_delta = quantity - existing.quantity;

            Ok(ActionPlan {
                patch: PatchEnvelope {
                    kind: "patch",
                    version: "1.0.0",
                    patch_id: next_patch_id(),
                    tenant_id: context.tenant_id.clone(),
                    target: item_patch_target(data, Some(id))?,
                    causation: PatchCausation {
                        action_name: "inventory.item.update",
                    },
                    operation: PatchOperation::UpdateItem {
                        id,
                        name,
                        category,
                        quantity,
                        quantity_delta,
                    },
                },
                event: EventCandidate {
                    event_type: "inventory.item.updated",
                    entity_type: "item",
                    entity_id_hint: Some(id),
                },
            })
        }
        NormalizedActionInvocation::DeleteItem { id, .. } => {
            let ContextQueryResult::InventoryItem(existing) = data
                .execute_context_query(context, ContextQuery::InventoryItemById(id))
                .await?
            else {
                return Err(RuntimeError::internal(
                    "unexpected context result for inventory item delete",
                ));
            };

            if existing.is_none() {
                return Err(RuntimeError::not_found("item not found"));
            }

            Ok(ActionPlan {
                patch: PatchEnvelope {
                    kind: "patch",
                    version: "1.0.0",
                    patch_id: next_patch_id(),
                    tenant_id: context.tenant_id.clone(),
                    target: item_patch_target(data, Some(id))?,
                    causation: PatchCausation {
                        action_name: "inventory.item.delete",
                    },
                    operation: PatchOperation::DeleteItem { id },
                },
                event: EventCandidate {
                    event_type: "inventory.item.deleted",
                    entity_type: "item",
                    entity_id_hint: Some(id),
                },
            })
        }
    }
}

fn item_patch_target<D: DataLayer>(data: &D, id: Option<i64>) -> Result<PatchTarget, RuntimeError> {
    let parsed = data
        .model_registry()
        .get(OWNED_ENTITY_TYPE)
        .ok_or_else(|| RuntimeError::internal("item model is not loaded"))?;

    Ok(PatchTarget {
        service: OWNED_SERVICE,
        entity_type: OWNED_ENTITY_TYPE,
        table: parsed.mapping.table_name.clone(),
        id,
    })
}

fn plan_deleted_id(plan: &ActionPlan) -> Option<i64> {
    match plan.patch.operation {
        PatchOperation::DeleteItem { id } => Some(id),
        _ => None,
    }
}

fn validate_name(name: &str) -> Result<(), RuntimeError> {
    if name.is_empty() {
        return Err(RuntimeError::bad_request("name is required"));
    }
    Ok(())
}

fn validate_category(category: &str) -> Result<(), RuntimeError> {
    if category.is_empty() {
        return Err(RuntimeError::bad_request("category is required"));
    }
    Ok(())
}

fn validate_quantity(quantity: i64) -> Result<(), RuntimeError> {
    if quantity < 0 {
        return Err(RuntimeError::bad_request("quantity must be >= 0"));
    }
    Ok(())
}
