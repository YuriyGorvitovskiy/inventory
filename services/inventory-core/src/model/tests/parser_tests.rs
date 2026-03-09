use crate::model::{load_model, parse_model, DefaultValue, IdPolicy};

use std::fs;

#[test]
fn parses_valid_model_and_normalizes_reference_none() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [
  { name = "name", type = "string", default = "" },
  { name = "category_id", type = "reference", default = "none", destination_type = "category" },
  { name = "quantity", type = "integer", default = 0 },
  { name = "created_at", type = "timestamp", default = "now" }
]
"#;

    let parsed = parse_model(src).expect("model should parse");

    assert_eq!(parsed.model.version.to_string(), "1.0.0");
    assert_eq!(parsed.model.entity.name, "item");
    assert_eq!(parsed.model.entity.fields.len(), 4);
    assert_eq!(parsed.model.entity.description, "");
    assert_eq!(parsed.orm.entity.table, "items");
    assert_eq!(parsed.orm.entity.id_policy, IdPolicy::ImplicitInt64);
    assert_eq!(
        parsed.model.entity.fields[1].default,
        DefaultValue::ReferenceId(0),
        "'none' must normalize to 0"
    );
    assert_eq!(parsed.model.entity.fields[1].destination_type, "category");
    assert_eq!(parsed.model.entity.fields[0].destination_type, "");
}

#[test]
fn rejects_reserved_id_field() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "id", type = "integer", default = 1 }]
"#;

    let err = parse_model(src).expect_err("reserved id field must fail");
    assert!(err.to_string().contains("reserved"));
}

#[test]
fn rejects_duplicate_field_names() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [
  { name = "name", type = "string", default = "a" },
  { name = "name", type = "string", default = "b" }
]
"#;

    let err = parse_model(src).expect_err("duplicate fields must fail");
    assert!(err.to_string().contains("duplicate field name"));
}

#[test]
fn rejects_reference_without_destination_type() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "category_id", type = "reference", default = 0 }]
"#;

    let err = parse_model(src).expect_err("destination_type required");
    assert!(err.to_string().contains("requires destination_type"));
}

#[test]
fn rejects_destination_type_for_non_reference() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "name", type = "string", default = "", destination_type = "category" }]
"#;

    let err = parse_model(src).expect_err("destination_type on non-reference must fail");
    assert!(err.to_string().contains("is not reference"));
}

#[test]
fn rejects_invalid_reference_default_literal() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "category_id", type = "reference", default = "unknown", destination_type = "category" }]
"#;

    let err = parse_model(src).expect_err("invalid reference literal must fail");
    assert!(err
        .to_string()
        .contains("reference default must be a non-negative integer or 'none'"));
}

#[test]
fn rejects_integer_field_with_string_default() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "quantity", type = "integer", default = "0" }]
"#;

    let err = parse_model(src).expect_err("typed default mismatch must fail");
    assert!(err.to_string().contains("expected integer"));
}

#[test]
fn rejects_unsupported_format_version() {
    let src = r#"
format_version = 2
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "name", type = "string", default = "" }]
"#;

    let err = parse_model(src).expect_err("unsupported version must fail");
    assert!(err.to_string().contains("unsupported format_version"));
}

#[test]
fn rejects_invalid_semver_model_version() {
    let src = r#"
format_version = 1
version = "one"

[entity]
name = "item"
fields = [{ name = "name", type = "string", default = "" }]
"#;

    let err = parse_model(src).expect_err("invalid semver model version must fail");
    assert!(err.to_string().contains("invalid model version"));
}

#[test]
fn rejects_invalid_id_policy() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
id_policy = "custom"
fields = [{ name = "name", type = "string", default = "" }]
"#;

    let err = parse_model(src).expect_err("invalid id policy must fail");
    assert!(err.to_string().contains("unsupported id_policy"));
}

#[test]
fn loads_model_from_file() {
    let src = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "category"
fields = [{ name = "name", type = "string", default = "" }]
"#;

    let path = std::env::temp_dir().join("inventory-core-model-test.toml");
    fs::write(&path, src).expect("test model should be written");
    let parsed = load_model(&path).expect("model should load");
    fs::remove_file(&path).expect("test model should be removed");

    assert_eq!(parsed.model.entity.name, "category");
}
