use crate::runtime::business::BusinessLayer;
use crate::runtime::contracts::{
    ActionInvocation, ActionResult, NormalizedActionInvocation, RequestContext, ResolvedItemListView,
    RuntimeError,
};
use crate::runtime::data::DataLayer;
use crate::runtime::registry::DefinitionRegistry;

#[derive(Default, Clone)]
pub struct InventoryUiLayer;

pub trait UiLayer<B, D>
where
    B: BusinessLayer<D>,
    D: DataLayer,
{
    async fn resolve_items_view(
        &self,
        context: RequestContext,
        definitions: &DefinitionRegistry,
        business: &B,
        data: &D,
    ) -> Result<ResolvedItemListView, RuntimeError>;

    async fn invoke_action(
        &self,
        invocation: ActionInvocation,
        definitions: &DefinitionRegistry,
        business: &B,
        data: &D,
    ) -> Result<ActionResult, RuntimeError>;
}

impl<B, D> UiLayer<B, D> for InventoryUiLayer
where
    B: BusinessLayer<D>,
    D: DataLayer,
{
    async fn resolve_items_view(
        &self,
        context: RequestContext,
        definitions: &DefinitionRegistry,
        business: &B,
        data: &D,
    ) -> Result<ResolvedItemListView, RuntimeError> {
        business.resolve_items_view(&context, definitions, data).await
    }

    async fn invoke_action(
        &self,
        invocation: ActionInvocation,
        definitions: &DefinitionRegistry,
        business: &B,
        data: &D,
    ) -> Result<ActionResult, RuntimeError> {
        let normalized = normalize_action_input(invocation)?;
        business.execute_action(normalized, definitions, data).await
    }
}

fn normalize_action_input(invocation: ActionInvocation) -> Result<NormalizedActionInvocation, RuntimeError> {
    match invocation {
        ActionInvocation::CreateItem {
            context,
            name,
            category,
            quantity,
        } => Ok(NormalizedActionInvocation::CreateItem {
            context,
            name: normalize_required_text("name", name)?,
            category: normalize_required_text("category", category)?,
            quantity,
        }),
        ActionInvocation::UpdateItem {
            context,
            id,
            name,
            category,
            quantity,
        } => Ok(NormalizedActionInvocation::UpdateItem {
            context,
            id,
            name: normalize_required_text("name", name)?,
            category: normalize_required_text("category", category)?,
            quantity,
        }),
        ActionInvocation::DeleteItem { context, id } => {
            Ok(NormalizedActionInvocation::DeleteItem { context, id })
        }
    }
}

fn normalize_required_text(field: &str, value: String) -> Result<String, RuntimeError> {
    let normalized = value.trim().to_string();
    if normalized.is_empty() {
        return Err(RuntimeError::bad_request(format!("{field} is required")));
    }
    Ok(normalized)
}
