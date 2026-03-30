use std::collections::HashSet;

use crate::model::model::Field;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityMapping {
    pub entity_name: String,
    pub table_name: String,
    pub primary_key_column: String,
    pub primary_key_index: String,
    pub fields: Vec<FieldMapping>,
}

impl EntityMapping {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn field(&self, field_name: &str) -> Option<&FieldMapping> {
        self.fields.iter().find(|field| field.field_name == field_name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMapping {
    pub field_name: String,
    pub column_name: String,
    pub index_name: Option<String>,
}

pub fn build_entity_mapping(
    entity_name: &str,
    table_override: Option<&str>,
    fields: &[Field],
) -> EntityMapping {
    let mut used_columns = HashSet::new();
    let mut used_indexes = HashSet::new();

    let table_base = table_override
        .filter(|value| !value.trim().is_empty())
        .map(normalize_identifier)
        .unwrap_or_else(|| format!("{}s", normalize_identifier(entity_name)));
    let table_name = unique_identifier(table_base, &mut HashSet::new());
    let primary_key_column = "id".to_string();
    let primary_key_index = format!("pk_{}", table_name);

    let fields = fields
        .iter()
        .map(|field| {
            let column_name =
                unique_identifier(normalize_identifier(&field.name), &mut used_columns);

            let index_name = if should_generate_index(field) {
                Some(unique_identifier(
                    format!("idx_{}_{}", table_name, column_name),
                    &mut used_indexes,
                ))
            } else {
                None
            };

            FieldMapping {
                field_name: field.name.clone(),
                column_name,
                index_name,
            }
        })
        .collect();

    EntityMapping {
        entity_name: entity_name.to_string(),
        table_name,
        primary_key_column,
        primary_key_index,
        fields,
    }
}

pub fn normalize_identifier(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut normalized = String::new();
    let mut pending_separator = false;

    for (index, ch) in chars.iter().copied().enumerate() {
        if ch.is_ascii_alphanumeric() {
            let prev = index.checked_sub(1).and_then(|prev| chars.get(prev)).copied();
            let next = chars.get(index + 1).copied();
            let starts_new_word = (ch.is_ascii_uppercase()
                && matches!(prev, Some(prev) if prev.is_ascii_lowercase() || prev.is_ascii_digit()))
                || matches!(
                    (prev, next),
                    (Some(prev), Some(next))
                        if prev.is_ascii_uppercase() && ch.is_ascii_uppercase() && next.is_ascii_lowercase()
                );

            if starts_new_word && !normalized.is_empty() && !normalized.ends_with('_') {
                normalized.push('_');
            } else if pending_separator && !normalized.is_empty() && !normalized.ends_with('_') {
                normalized.push('_');
            }

            normalized.push(ch.to_ascii_lowercase());
            pending_separator = false;
        } else {
            pending_separator = !normalized.is_empty();
        }
    }

    let trimmed = normalized.trim_matches('_');
    if trimmed.is_empty() {
        "unnamed".to_string()
    } else {
        trimmed.to_string()
    }
}

fn unique_identifier(base: String, used: &mut HashSet<String>) -> String {
    if used.insert(base.clone()) {
        return base;
    }

    let mut attempt = 2;
    loop {
        let candidate = format!("{base}_{attempt}");
        if used.insert(candidate.clone()) {
            return candidate;
        }
        attempt += 1;
    }
}

fn should_generate_index(field: &Field) -> bool {
    field.indexed || matches!(field.field_type, crate::model::model::FieldType::Reference)
}
