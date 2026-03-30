use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    model::{
        model::{ConflictResolutionMode, DefaultValue, FieldType},
        ParsedModel,
    },
    models::{HealthResponse, ItemResponse, ReadinessResponse},
};

pub const OWNED_SERVICE: &str = "inventory-core";
pub const OWNED_ENTITY_TYPE: &str = "item";
static PATCH_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct RequestContext {
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextQueryDefinition {
    pub kind: String,
    pub version: String,
    pub name: String,
    pub root_entity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionDefinition {
    pub kind: String,
    pub version: String,
    pub name: String,
    pub description: String,
    pub context_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub kind: String,
    pub version: String,
    pub name: String,
    pub entity_scope: String,
    #[serde(default)]
    pub params: Vec<ViewParamDefinition>,
    #[serde(default)]
    pub context_queries: Vec<ViewContextQueryBinding>,
    pub layout: FrameworkWidgetDefinition,
    #[serde(default)]
    pub interactions: Vec<ViewInteractionDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewParamDefinition {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewContextQueryBinding {
    pub query: String,
    pub bind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInteractionDefinition {
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route_to: Option<String>,
    #[serde(default)]
    pub params: Vec<ViewInteractionParamDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewInteractionParamDefinition {
    pub name: String,
    pub value: WidgetDataMapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FrameworkWidgetDefinition {
    Page {
        title: String,
        #[serde(default)]
        children: Vec<FrameworkWidgetDefinition>,
    },
    ActionBar {
        actions: Vec<String>,
    },
    Table {
        rows: WidgetDataMapping,
        columns: Vec<TableColumnDefinition>,
    },
    Form {
        fields: Vec<FormFieldDefinition>,
    },
    Text {
        value: WidgetDataMapping,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetDataMapping {
    pub bind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumnDefinition {
    pub key: String,
    pub header: String,
    pub value: WidgetDataMapping,
    #[serde(default)]
    pub editable: bool,
    pub editor_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFieldDefinition {
    pub key: String,
    pub label: String,
    pub value: WidgetDataMapping,
    pub editor_kind: String,
    #[serde(default = "default_true")]
    pub editable: bool,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItemRecord {
    pub id: i64,
    pub entity_id: String,
    pub name: String,
    pub category: String,
    pub quantity: i64,
}

impl From<InventoryItemRecord> for ItemResponse {
    fn from(value: InventoryItemRecord) -> Self {
        Self {
            id: value.id,
            entity_id: value.entity_id,
            name: value.name,
            category: value.category,
            quantity: value.quantity,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ActionInvocation {
    CreateItem {
        context: RequestContext,
        name: String,
        category: String,
        quantity: i64,
    },
    UpdateItem {
        context: RequestContext,
        id: i64,
        name: String,
        category: String,
        quantity: i64,
    },
    DeleteItem {
        context: RequestContext,
        id: i64,
    },
}

#[derive(Debug, Clone)]
pub enum NormalizedActionInvocation {
    CreateItem {
        context: RequestContext,
        name: String,
        category: String,
        quantity: i64,
    },
    UpdateItem {
        context: RequestContext,
        id: i64,
        name: String,
        category: String,
        quantity: i64,
    },
    DeleteItem {
        context: RequestContext,
        id: i64,
    },
}

impl NormalizedActionInvocation {
    pub fn context(&self) -> &RequestContext {
        match self {
            Self::CreateItem { context, .. }
            | Self::UpdateItem { context, .. }
            | Self::DeleteItem { context, .. } => context,
        }
    }

    pub fn definition_name(&self) -> &'static str {
        match self {
            Self::CreateItem { .. } => "inventory.item.create",
            Self::UpdateItem { .. } => "inventory.item.update",
            Self::DeleteItem { .. } => "inventory.item.delete",
        }
    }
}

#[derive(Debug, Clone)]
pub enum ContextQuery {
    InventoryItems,
    InventoryItemById(i64),
}

#[derive(Debug, Clone)]
pub enum ContextQueryResult {
    InventoryItems(Vec<InventoryItemRecord>),
    InventoryItem(Option<InventoryItemRecord>),
}

#[derive(Debug, Clone)]
pub enum ProjectionQuery {
    InventoryItemList,
}

#[derive(Debug, Clone)]
pub enum ProjectionResult {
    InventoryItems(Vec<InventoryItemRecord>),
}

#[derive(Debug, Clone)]
pub enum PatchOperation {
    CreateItem {
        name: String,
        category: String,
        quantity: i64,
    },
    UpdateItem {
        id: i64,
        name: String,
        category: String,
        quantity: i64,
        quantity_delta: i64,
    },
    DeleteItem {
        id: i64,
    },
}

#[derive(Debug, Clone)]
pub struct PatchEnvelope {
    pub kind: &'static str,
    pub version: &'static str,
    pub patch_id: String,
    pub tenant_id: String,
    pub target: PatchTarget,
    pub causation: PatchCausation,
    pub operation: PatchOperation,
}

#[derive(Debug, Clone)]
pub struct PatchTarget {
    pub service: &'static str,
    pub entity_type: &'static str,
    pub table: String,
    pub id: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct PatchCausation {
    pub action_name: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct EventEnvelope {
    pub kind: &'static str,
    pub version: &'static str,
    pub patch_id: String,
    pub event_type: &'static str,
    pub entity_id: String,
    pub entity_type: &'static str,
    pub service: &'static str,
    pub tenant_id: String,
    pub action_name: &'static str,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct EventCandidate {
    pub event_type: &'static str,
    pub entity_type: &'static str,
    pub entity_id_hint: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ActionPlan {
    pub patch: PatchEnvelope,
    pub event: EventCandidate,
}

#[derive(Debug, Clone, Serialize)]
pub enum ActionOutcome {
    Item(InventoryItemRecord),
    Deleted { id: i64 },
}

#[derive(Debug, Clone, Serialize)]
pub struct ActionResult {
    pub outcome: ActionOutcome,
    pub event: EventEnvelope,
    pub definition: ActionDefinition,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedView {
    pub definition: ViewDefinition,
    pub params: Map<String, Value>,
    pub context: Map<String, Value>,
    pub widget: ResolvedFrameworkWidget,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResolvedFrameworkWidget {
    Page {
        title: String,
        children: Vec<ResolvedFrameworkWidget>,
    },
    ActionBar {
        actions: Vec<ResolvedActionReference>,
    },
    Table {
        columns: Vec<ResolvedTableColumn>,
        rows: Vec<ResolvedTableRow>,
    },
    Form {
        fields: Vec<ResolvedFormField>,
    },
    Text {
        text: String,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedActionReference {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedTableColumn {
    pub key: String,
    pub header: String,
    pub editable: bool,
    pub editor_kind: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedTableRow {
    pub cells: Map<String, Value>,
    pub source: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResolvedFormField {
    pub key: String,
    pub label: String,
    pub value: Value,
    pub editor_kind: String,
    pub editable: bool,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeModelApiResponse {
    pub classes: Vec<RuntimeModelClassResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeModelClassResponse {
    pub name: String,
    pub version: String,
    pub description: String,
    pub table: String,
    pub fields: Vec<RuntimeModelFieldResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeModelFieldResponse {
    pub name: String,
    pub column: String,
    pub field_type: String,
    pub destination_type: String,
    pub description: String,
    pub default_value: serde_json::Value,
    pub indexed: bool,
    pub required: bool,
    pub conflict_resolution: String,
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    status: StatusCode,
    message: String,
}

impl RuntimeError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl From<sqlx::Error> for RuntimeError {
    fn from(value: sqlx::Error) -> Self {
        Self::internal(format!("database error: {value}"))
    }
}

impl From<crate::model::ModelError> for RuntimeError {
    fn from(value: crate::model::ModelError) -> Self {
        Self::internal(value.to_string())
    }
}

pub fn health_response() -> HealthResponse {
    HealthResponse {
        status: "ok",
        service: "inventory-core",
    }
}

pub fn readiness_response() -> ReadinessResponse {
    ReadinessResponse {
        status: "ready",
        checks: ["app"],
    }
}

pub fn parsed_models_to_response(parsed_models: Vec<&ParsedModel>) -> RuntimeModelApiResponse {
    RuntimeModelApiResponse {
        classes: parsed_models.into_iter().map(parsed_to_class).collect(),
    }
}

fn parsed_to_class(parsed: &ParsedModel) -> RuntimeModelClassResponse {
    RuntimeModelClassResponse {
        name: parsed.model.entity.name.clone(),
        version: parsed.model.version.to_string(),
        description: parsed.model.entity.description.clone(),
        table: parsed.mapping.table_name.clone(),
        fields: parsed
            .model
            .entity
            .fields
            .iter()
            .zip(parsed.mapping.fields.iter())
            .map(|(field, mapping)| RuntimeModelFieldResponse {
                name: field.name.clone(),
                column: mapping.column_name.clone(),
                field_type: field_type_to_str(field.field_type).to_string(),
                destination_type: field.destination_type.clone(),
                description: field.description.clone(),
                default_value: default_to_json(&field.default),
                indexed: field.indexed,
                required: field.required,
                conflict_resolution: conflict_resolution_to_str(field.conflict_resolution.mode)
                    .to_string(),
            })
            .collect(),
    }
}

pub fn next_patch_id() -> String {
    let sequence = PATCH_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("pat_{sequence:08}")
}

pub fn field_type_to_str(field_type: FieldType) -> &'static str {
    match field_type {
        FieldType::Label => "label",
        FieldType::Boolean => "boolean",
        FieldType::Integer => "integer",
        FieldType::Float => "float",
        FieldType::Timestamp => "timestamp",
        FieldType::String => "string",
        FieldType::Text => "text",
        FieldType::Reference => "reference",
    }
}

fn default_true() -> bool {
    true
}

fn default_to_json(default: &DefaultValue) -> serde_json::Value {
    match default {
        DefaultValue::Boolean(v) => serde_json::Value::Bool(*v),
        DefaultValue::Integer(v) => serde_json::Value::Number((*v).into()),
        DefaultValue::Float(v) => serde_json::Number::from_f64(*v)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::Null),
        DefaultValue::Label(v)
        | DefaultValue::Timestamp(v)
        | DefaultValue::String(v)
        | DefaultValue::Text(v) => serde_json::Value::String(v.clone()),
        DefaultValue::ReferenceId(v) => serde_json::Value::Number((*v).into()),
    }
}

fn conflict_resolution_to_str(mode: ConflictResolutionMode) -> &'static str {
    match mode {
        ConflictResolutionMode::LastChangeWins => "last_change_wins",
        ConflictResolutionMode::Increment => "increment",
        ConflictResolutionMode::Decrement => "decrement",
        ConflictResolutionMode::InsertBefore => "insert_before",
        ConflictResolutionMode::InsertAfter => "insert_after",
    }
}
