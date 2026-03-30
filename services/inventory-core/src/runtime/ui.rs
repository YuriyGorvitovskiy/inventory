use serde_json::{Map, Value};

use crate::runtime::business::BusinessLayer;
use crate::runtime::contracts::{
    ActionInvocation, ActionResult, ContextQuery, ContextQueryResult, FrameworkWidgetDefinition,
    NormalizedActionInvocation, RequestContext, ResolvedActionReference, ResolvedFormField,
    ResolvedFrameworkWidget, ResolvedTableColumn, ResolvedTableRow, ResolvedView, RuntimeError,
    ViewContextQueryBinding, ViewDefinition, WidgetDataMapping,
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
    async fn resolve_view(
        &self,
        view_name: &str,
        params: Map<String, Value>,
        context: RequestContext,
        definitions: &DefinitionRegistry,
        data: &D,
    ) -> Result<ResolvedView, RuntimeError>;

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
    async fn resolve_view(
        &self,
        view_name: &str,
        params: Map<String, Value>,
        context: RequestContext,
        definitions: &DefinitionRegistry,
        data: &D,
    ) -> Result<ResolvedView, RuntimeError> {
        let definition = definitions.view(view_name)?;
        let params = validate_view_params(&definition, params)?;
        let query_context = load_view_context(
            &context,
            &params,
            &definition.context_queries,
            definitions,
            data,
        )
        .await?;
        validate_interactions(definitions, &definition)?;
        let widget = resolve_widget(
            &definition.layout,
            &params,
            &query_context,
            None,
            definitions,
        )?;

        Ok(ResolvedView {
            definition,
            params,
            context: query_context,
            widget,
        })
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

fn normalize_action_input(
    invocation: ActionInvocation,
) -> Result<NormalizedActionInvocation, RuntimeError> {
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

async fn load_view_context<D: DataLayer>(
    context: &RequestContext,
    params: &Map<String, Value>,
    bindings: &[ViewContextQueryBinding],
    definitions: &DefinitionRegistry,
    data: &D,
) -> Result<Map<String, Value>, RuntimeError> {
    let mut query_context = Map::new();
    for binding in bindings {
        let definition = definitions.query(&binding.query)?;
        let query = match definition.name.as_str() {
            "inventory.items" => ContextQuery::InventoryItems,
            "inventory.item.by_id" => {
                let id = extract_i64(params, "id")?;
                ContextQuery::InventoryItemById(id)
            }
            other => {
                return Err(RuntimeError::internal(format!(
                    "unsupported view context query '{}'",
                    other
                )))
            }
        };

        let result = data.execute_context_query(context, query).await?;
        query_context.insert(binding.bind.clone(), context_query_to_value(result));
    }
    Ok(query_context)
}

fn validate_interactions(
    definitions: &DefinitionRegistry,
    view: &ViewDefinition,
) -> Result<(), RuntimeError> {
    for interaction in &view.interactions {
        if let Some(action_name) = &interaction.action {
            let _ = definitions.action(action_name)?;
        }
        if let Some(route_name) = &interaction.route_to {
            let target_view = definitions.view(route_name)?;
            for route_param in &interaction.params {
                if !target_view.params.iter().any(|param| param.name == route_param.name) {
                    return Err(RuntimeError::internal(format!(
                        "interaction '{}' routes to '{}' with unknown param '{}'",
                        interaction.event, route_name, route_param.name
                    )));
                }
            }
            for required_param in target_view.params.iter().filter(|param| param.required) {
                if !interaction.params.iter().any(|param| param.name == required_param.name) {
                    return Err(RuntimeError::internal(format!(
                        "interaction '{}' routes to '{}' without required param '{}'",
                        interaction.event, route_name, required_param.name
                    )));
                }
            }
        }
    }
    Ok(())
}

fn resolve_widget(
    definition: &FrameworkWidgetDefinition,
    params: &Map<String, Value>,
    context: &Map<String, Value>,
    row: Option<&Value>,
    definitions: &DefinitionRegistry,
) -> Result<ResolvedFrameworkWidget, RuntimeError> {
    match definition {
        FrameworkWidgetDefinition::Page { title, children } => Ok(ResolvedFrameworkWidget::Page {
            title: title.clone(),
            children: children
                .iter()
                .map(|child| resolve_widget(child, params, context, row, definitions))
                .collect::<Result<Vec<_>, _>>()?,
        }),
        FrameworkWidgetDefinition::ActionBar { actions } => {
            Ok(ResolvedFrameworkWidget::ActionBar {
                actions: actions
                    .iter()
                    .map(|action_name| {
                        let action = definitions.action(action_name)?;
                        Ok(ResolvedActionReference {
                            name: action.name,
                            description: action.description,
                        })
                    })
                    .collect::<Result<Vec<_>, RuntimeError>>()?,
            })
        }
        FrameworkWidgetDefinition::Table { rows, columns } => {
            let row_values = evaluate_mapping(rows, params, context, row)?;
            let row_values = row_values.as_array().ok_or_else(|| {
                RuntimeError::internal(format!(
                    "widget mapping '{}' did not resolve to a row array",
                    rows.bind
                ))
            })?;

            let resolved_columns = columns
                .iter()
                .map(|column| ResolvedTableColumn {
                    key: column.key.clone(),
                    header: column.header.clone(),
                    editable: column.editable,
                    editor_kind: column.editor_kind.clone(),
                })
                .collect::<Vec<_>>();

            let resolved_rows = row_values
                .iter()
                .map(|row_value| {
                    let cells = columns
                        .iter()
                        .map(|column| {
                            Ok((
                                column.key.clone(),
                                evaluate_mapping(&column.value, params, context, Some(row_value))?,
                            ))
                        })
                        .collect::<Result<Map<String, Value>, RuntimeError>>()?;

                    Ok(ResolvedTableRow {
                        cells,
                        source: row_value.clone(),
                    })
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?;

            Ok(ResolvedFrameworkWidget::Table {
                columns: resolved_columns,
                rows: resolved_rows,
            })
        }
        FrameworkWidgetDefinition::Form { fields } => Ok(ResolvedFrameworkWidget::Form {
            fields: fields
                .iter()
                .map(|field| {
                    Ok(ResolvedFormField {
                        key: field.key.clone(),
                        label: field.label.clone(),
                        value: evaluate_mapping(&field.value, params, context, row)?,
                        editor_kind: field.editor_kind.clone(),
                        editable: field.editable,
                        required: field.required,
                    })
                })
                .collect::<Result<Vec<_>, RuntimeError>>()?,
        }),
        FrameworkWidgetDefinition::Text { value } => {
            let resolved = evaluate_mapping(value, params, context, row)?;
            Ok(ResolvedFrameworkWidget::Text {
                text: value_to_text(&resolved),
            })
        }
    }
}

fn evaluate_mapping(
    mapping: &WidgetDataMapping,
    params: &Map<String, Value>,
    context: &Map<String, Value>,
    row: Option<&Value>,
) -> Result<Value, RuntimeError> {
    let Some(path) = mapping.bind.strip_prefix('$') else {
        return Ok(Value::String(mapping.bind.clone()));
    };

    let mut segments = path.split('.');
    let root = segments
        .next()
        .ok_or_else(|| RuntimeError::internal("widget data mapping is missing a root binding"))?;

    let mut current = match root {
        "params" => Value::Object(params.clone()),
        "context" => Value::Object(context.clone()),
        "row" => row.cloned().ok_or_else(|| {
            RuntimeError::internal(format!(
                "widget mapping '{}' requires a row binding",
                mapping.bind
            ))
        })?,
        other => {
            return Err(RuntimeError::internal(format!(
                "unsupported widget data root '{}'",
                other
            )))
        }
    };

    for segment in segments {
        current = current.get(segment).cloned().ok_or_else(|| {
            RuntimeError::internal(format!(
                "widget mapping '{}' could not be resolved",
                mapping.bind
            ))
        })?;
    }

    Ok(current)
}

fn context_query_to_value(result: ContextQueryResult) -> Value {
    match result {
        ContextQueryResult::InventoryItems(rows) => serde_json::json!({ "rows": rows }),
        ContextQueryResult::InventoryItem(item) => {
            let rows: Vec<_> = item.iter().cloned().collect();
            serde_json::json!({ "item": item, "rows": rows })
        }
    }
}

fn extract_i64(params: &Map<String, Value>, key: &str) -> Result<i64, RuntimeError> {
    params
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| RuntimeError::bad_request(format!("missing required view param '{key}'")))
}

fn validate_view_params(
    definition: &ViewDefinition,
    params: Map<String, Value>,
) -> Result<Map<String, Value>, RuntimeError> {
    let mut validated = Map::new();
    for param in &definition.params {
        match params.get(&param.name) {
            Some(value) => {
                validated.insert(
                    param.name.clone(),
                    coerce_param_value(&param.param_type, value.clone()).map_err(|message| {
                        RuntimeError::bad_request(format!("invalid view param '{}': {message}", param.name))
                    })?,
                );
            }
            None if param.required => {
                return Err(RuntimeError::bad_request(format!(
                    "missing required view param '{}'",
                    param.name
                )));
            }
            None => {}
        }
    }

    for (key, value) in params {
        validated.entry(key).or_insert(value);
    }

    Ok(validated)
}

fn coerce_param_value(param_type: &str, value: Value) -> Result<Value, String> {
    match param_type {
        "integer" => match value {
            Value::Number(number) if number.is_i64() || number.is_u64() => Ok(Value::Number(number)),
            Value::String(text) => text
                .parse::<i64>()
                .map(|parsed| Value::Number(parsed.into()))
                .map_err(|_| "expected integer".to_string()),
            _ => Err("expected integer".to_string()),
        },
        "boolean" => match value {
            Value::Bool(flag) => Ok(Value::Bool(flag)),
            Value::String(text) => match text.as_str() {
                "true" => Ok(Value::Bool(true)),
                "false" => Ok(Value::Bool(false)),
                _ => Err("expected boolean".to_string()),
            },
            _ => Err("expected boolean".to_string()),
        },
        "string" | "text" | "label" => match value {
            Value::String(text) => Ok(Value::String(text)),
            _ => Err("expected string".to_string()),
        },
        other => Err(format!("unsupported param type '{other}'")),
    }
}

fn value_to_text(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        other => other.to_string(),
    }
}
