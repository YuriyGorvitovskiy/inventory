use std::collections::HashSet;
use std::fs;
use std::path::Path;

use semver::Version;

use crate::model::error::ModelError;
use crate::model::model::{DefaultValue, Field, FieldType, Model, Type};
use crate::model::orm::{IdPolicy, OrmModel, OrmType};
use crate::model::raw::{RawDefault, RawField, RawModelFile};

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedModel {
    pub model: Model,
    pub orm: OrmModel,
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

    let id_policy = match raw.entity.id_policy.as_deref() {
        None | Some("implicit_int64") => IdPolicy::ImplicitInt64,
        Some(other) => {
            return Err(ModelError::new(format!(
                "unsupported id_policy '{other}', expected 'implicit_int64'"
            )))
        }
    };

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

    Ok(ParsedModel {
        model: Model {
            version,
            entity: Type {
                name: raw.entity.name,
                description: raw.entity.description.unwrap_or_default(),
                fields,
            },
        },
        orm: OrmModel {
            entity: OrmType { table, id_policy },
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
