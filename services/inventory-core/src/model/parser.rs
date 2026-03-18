use std::collections::HashSet;
use std::fs;
use std::path::Path;

use semver::Version;

use crate::model::error::ModelError;
use crate::model::model::{DefaultValue, Field, FieldType, Model, Type};
use crate::schema::{Column, DBType, Index, PrimaryKey, Schema, Table};
use crate::model::raw::{RawDefault, RawField, RawModelFile};

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedModel {
    pub model: Model,
    pub schema: Schema,
}

pub fn parse_model(input: &str) -> Result<ParsedModel, ModelError> {
    let raw = toml::from_str::<RawModelFile>(input)
        .map_err(|err| ModelError::new(format!("invalid TOML: {err}")))?;
    build_model(raw)
}

pub fn load_model(path: &Path) -> Result<ParsedModel, ModelError> {
    let content = fs::read_to_string(path)
        .map_err(|err| ModelError::new(format!("failed to read {}: {err}", path.display())))?;
    parse_model(&content)
}

fn build_model(raw: RawModelFile) -> Result<ParsedModel, ModelError> {
    if raw.format_version != 1 {
        return Err(ModelError::new(format!(
            "unsupported format_version {}, expected 1",
            raw.format_version
        )));
    }

    let version = Version::parse(&raw.version)
        .map_err(|err| ModelError::new(format!("invalid model version '{}': {err}", raw.version)))?;

    match raw.entity.id_policy.as_deref() {
        None | Some("implicit_int64") => {}
        Some(other) => {
            return Err(ModelError::new(format!(
                "unsupported id_policy '{other}', expected 'implicit_int64'"
            )))
        }
    }

    if raw.entity.name.trim().is_empty() {
        return Err(ModelError::new("entity.name must not be empty"));
    }

    if raw.entity.fields.is_empty() {
        return Err(ModelError::new("entity.fields must not be empty"));
    }

    let mut seen = HashSet::new();
    let mut fields = Vec::with_capacity(raw.entity.fields.len());
    for raw_field in raw.entity.fields {
        if raw_field.name == "id" {
            return Err(ModelError::new("field name 'id' is reserved"));
        }
        if !seen.insert(raw_field.name.clone()) {
            return Err(ModelError::new(format!(
                "duplicate field name '{}'",
                raw_field.name
            )));
        }
        fields.push(convert_field(raw_field)?);
    }

    let table = raw
        .entity
        .table
        .unwrap_or_else(|| format!("{}s", raw.entity.name));

    let table_schema = build_table_schema(&table, &fields);

    let entity_name = raw.entity.name.clone();

    Ok(ParsedModel {
        model: Model {
            version,
            entity: Type {
                name: entity_name.clone(),
                description: raw.entity.description.unwrap_or_default(),
                fields,
            },
        },
        schema: Schema {
            name: entity_name,
            tables: vec![table_schema],
        },
    })
}

fn convert_field(raw_field: RawField) -> Result<Field, ModelError> {
    if raw_field.name.trim().is_empty() {
        return Err(ModelError::new("field.name must not be empty"));
    }

    let field_type = raw_field.field_type;

    let default = match field_type {
        FieldType::Boolean => match raw_field.default {
            RawDefault::Boolean(v) => DefaultValue::Boolean(v),
            _ => return Err(ModelError::new(type_mismatch(&raw_field.name, "boolean"))),
        },
        FieldType::Integer => match raw_field.default {
            RawDefault::Integer(v) => DefaultValue::Integer(v),
            _ => return Err(ModelError::new(type_mismatch(&raw_field.name, "integer"))),
        },
        FieldType::Float => match raw_field.default {
            RawDefault::Integer(v) => DefaultValue::Float(v as f64),
            RawDefault::Float(v) => DefaultValue::Float(v),
            _ => return Err(ModelError::new(type_mismatch(&raw_field.name, "float"))),
        },
        FieldType::Timestamp => match raw_field.default {
            RawDefault::String(v) if !v.is_empty() => DefaultValue::Timestamp(v),
            _ => {
                return Err(ModelError::new(
                    "timestamp default must be a non-empty string (for example 'now')",
                ))
            }
        },
        FieldType::String => match raw_field.default {
            RawDefault::String(v) => DefaultValue::String(v),
            _ => return Err(ModelError::new(type_mismatch(&raw_field.name, "string"))),
        },
        FieldType::Text => match raw_field.default {
            RawDefault::String(v) => DefaultValue::Text(v),
            _ => return Err(ModelError::new(type_mismatch(&raw_field.name, "text"))),
        },
        FieldType::Reference => match raw_field.default {
            RawDefault::Integer(v) if v >= 0 => DefaultValue::ReferenceId(v),
            RawDefault::String(v) if v == "none" => DefaultValue::ReferenceId(0),
            _ => {
                return Err(ModelError::new(
                    "reference default must be a non-negative integer or 'none'",
                ))
            }
        },
    };

    match field_type {
        FieldType::Reference => {
            if raw_field.destination_type.is_none() {
                return Err(ModelError::new(format!(
                    "field '{}' with type 'reference' requires destination_type",
                    raw_field.name
                )));
            }
        }
        _ => {
            if raw_field.destination_type.is_some() {
                return Err(ModelError::new(format!(
                    "field '{}' has destination_type but is not reference",
                    raw_field.name
                )));
            }
        }
    }

    Ok(Field {
        name: raw_field.name,
        field_type,
        destination_type: raw_field.destination_type.unwrap_or_default(),
        description: raw_field.description.unwrap_or_default(),
        default,
        indexed: raw_field.indexed.unwrap_or(false),
    })
}

fn type_mismatch(field_name: &str, expected: &str) -> String {
    format!("field '{field_name}' default type mismatch, expected {expected}")
}

fn build_table_schema(table: &str, fields: &[Field]) -> Table {
    let mut columns = vec![Column {
        name: "id".to_string(),
        data_type: DBType::BigInt,
        nullable: false,
        default: Some("GENERATED BY DEFAULT AS IDENTITY".to_string()),
    }];

    let primary_key = PrimaryKey {
        name: format!("pk_{table}"),
        columns: vec!["id".to_string()],
    };
    let mut indexes = Vec::new();

    for field in fields {
        columns.push(Column {
            name: field.name.clone(),
            data_type: data_type_for_field(field.field_type),
            nullable: false,
            default: Some(sql_default_for_field(&field.default)),
        });

        if should_generate_index(field) {
            indexes.push(Index {
                name: format!("idx_{table}_{}", field.name),
                columns: vec![field.name.clone()],
                unique: false,
            });
        }
    }

    Table {
        name: table.to_string(),
        columns,
        primary_key,
        indexes,
    }
}

fn data_type_for_field(field_type: FieldType) -> DBType {
    match field_type {
        FieldType::Boolean => DBType::Boolean,
        FieldType::Integer | FieldType::Reference => DBType::BigInt,
        FieldType::Float => DBType::DoublePrecision,
        FieldType::Timestamp => DBType::TimestampWithTimeZone,
        FieldType::String => DBType::VarChar850,
        FieldType::Text => DBType::Text,
    }
}

fn sql_default_for_field(default: &DefaultValue) -> String {
    match default {
        DefaultValue::Boolean(value) => value.to_string(),
        DefaultValue::Integer(value) => value.to_string(),
        DefaultValue::Float(value) => value.to_string(),
        DefaultValue::Timestamp(value) => {
            if value == "now" {
                "NOW()".to_string()
            } else {
                format!("'{}'", value.replace('\'', "''"))
            }
        }
        DefaultValue::String(value) | DefaultValue::Text(value) => {
            format!("'{}'", value.replace('\'', "''"))
        }
        DefaultValue::ReferenceId(value) => value.to_string(),
    }
}

fn should_generate_index(field: &Field) -> bool {
    field.indexed || matches!(field.field_type, FieldType::Reference)
}
