use crate::model::registry::ModelRegistry;

use std::fs;

#[test]
fn loads_all_model_files_from_directory() {
    let dir = std::env::temp_dir().join("inventory-core-model-registry-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("test dir must be created");

    let item = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "item"
fields = [{ name = "name", type = "string", default = "" }]
"#;
    let category = r#"
format_version = 1
version = "1.0.0"

[entity]
name = "category"
fields = [{ name = "name", type = "string", default = "" }]
"#;

    fs::write(dir.join("item.model.toml"), item).expect("item model must be written");
    fs::write(dir.join("category.model.toml"), category).expect("category model must be written");

    let registry = ModelRegistry::load_from_dir(&dir).expect("registry should load");

    assert_eq!(registry.len(), 2);
    assert!(registry.get("item").is_some());
    assert!(registry.get("category").is_some());

    fs::remove_dir_all(&dir).expect("test dir must be removed");
}

#[test]
fn rejects_empty_model_directory() {
    let dir = std::env::temp_dir().join("inventory-core-model-registry-empty-test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("test dir must be created");

    let err = ModelRegistry::load_from_dir(&dir).expect_err("empty dir must fail");
    assert!(err.to_string().contains("no model files"));

    fs::remove_dir_all(&dir).expect("test dir must be removed");
}
