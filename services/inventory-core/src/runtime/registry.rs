use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::runtime::contracts::{
    ActionDefinition, ContextQueryDefinition, RuntimeError, ViewDefinition,
};

#[derive(Debug, Clone)]
pub struct DefinitionRegistry {
    actions: HashMap<String, ActionDefinition>,
    queries: HashMap<String, ContextQueryDefinition>,
    views: HashMap<String, ViewDefinition>,
}

impl DefinitionRegistry {
    pub fn load_from_dir(dir: &Path) -> Result<Self, RuntimeError> {
        let files = definition_files(dir)?;

        let mut actions = HashMap::new();
        let mut queries = HashMap::new();
        let mut views = HashMap::new();

        for file in files {
            let content = fs::read_to_string(&file).map_err(|err| {
                RuntimeError::internal(format!("failed to read {}: {err}", file.display()))
            })?;
            let header: RawDefinitionHeader = toml::from_str(&content).map_err(|err| {
                RuntimeError::internal(format!(
                    "invalid definition TOML in {}: {err}",
                    file.display()
                ))
            })?;

            match header.kind.as_str() {
                "action_definition" => {
                    let definition: ActionDefinition = toml::from_str(&content).map_err(|err| {
                        RuntimeError::internal(format!(
                            "invalid action definition in {}: {err}",
                            file.display()
                        ))
                    })?;
                    insert_unique(&mut actions, definition.name.to_string(), definition, &file)?;
                }
                "context_query_definition" => {
                    let definition: ContextQueryDefinition =
                        toml::from_str(&content).map_err(|err| {
                            RuntimeError::internal(format!(
                                "invalid query definition in {}: {err}",
                                file.display()
                            ))
                        })?;
                    insert_unique(&mut queries, definition.name.to_string(), definition, &file)?;
                }
                "view_definition" => {
                    let definition: ViewDefinition = toml::from_str(&content).map_err(|err| {
                        RuntimeError::internal(format!(
                            "invalid view definition in {}: {err}",
                            file.display()
                        ))
                    })?;
                    insert_unique(&mut views, definition.name.to_string(), definition, &file)?;
                }
                other => {
                    return Err(RuntimeError::internal(format!(
                        "unsupported definition kind '{other}' in {}",
                        file.display()
                    )));
                }
            }
        }

        Ok(Self {
            actions,
            queries,
            views,
        })
    }

    pub fn action(&self, name: &str) -> Result<ActionDefinition, RuntimeError> {
        self.actions
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::internal(format!("missing action definition '{name}'")))
    }

    pub fn query(&self, name: &str) -> Result<ContextQueryDefinition, RuntimeError> {
        self.queries.get(name).cloned().ok_or_else(|| {
            RuntimeError::internal(format!("missing context query definition '{name}'"))
        })
    }

    pub fn view(&self, name: &str) -> Result<ViewDefinition, RuntimeError> {
        self.views
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::internal(format!("missing view definition '{name}'")))
    }
}

#[derive(Debug, Clone)]
pub struct RouteCatalog {
    routes: HashMap<String, RuntimeRoute>,
}

impl RouteCatalog {
    pub fn load_from_dir(dir: &Path) -> Result<Self, RuntimeError> {
        let files = route_files(dir)?;
        let mut routes = HashMap::new();

        for file in files {
            let content = fs::read_to_string(&file).map_err(|err| {
                RuntimeError::internal(format!("failed to read {}: {err}", file.display()))
            })?;
            let route: RuntimeRoute = toml::from_str(&content).map_err(|err| {
                RuntimeError::internal(format!(
                    "invalid route definition in {}: {err}",
                    file.display()
                ))
            })?;
            insert_unique(&mut routes, route.name.clone(), route, &file)?;
        }

        Ok(Self { routes })
    }

    pub fn route(&self, name: &str) -> Result<RuntimeRoute, RuntimeError> {
        self.routes
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::internal(format!("missing runtime route '{name}'")))
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RuntimeRoute {
    pub kind: String,
    pub version: String,
    pub name: String,
    pub target: String,
}

#[derive(Debug, Deserialize)]
struct RawDefinitionHeader {
    kind: String,
}

fn definition_files(dir: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
    collect_matching_files(dir, |name| {
        name.ends_with(".action.toml")
            || name.ends_with(".query.toml")
            || name.ends_with(".view.toml")
    })
}

fn route_files(dir: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
    collect_matching_files(dir, |name| name.ends_with(".route.toml"))
}

fn collect_matching_files(
    dir: &Path,
    predicate: impl Fn(&str) -> bool,
) -> Result<Vec<PathBuf>, RuntimeError> {
    let entries = fs::read_dir(dir).map_err(|err| {
        RuntimeError::internal(format!(
            "failed to read definitions dir {}: {err}",
            dir.display()
        ))
    })?;

    let mut files: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_file())
        .filter(|path| {
            path.file_name()
                .and_then(|n| n.to_str())
                .map(&predicate)
                .unwrap_or(false)
        })
        .collect();
    files.sort();
    Ok(files)
}

fn insert_unique<T>(
    map: &mut HashMap<String, T>,
    key: String,
    value: T,
    file: &Path,
) -> Result<(), RuntimeError> {
    if map.insert(key.clone(), value).is_some() {
        return Err(RuntimeError::internal(format!(
            "duplicate runtime definition '{key}' in {}",
            file.display()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{DefinitionRegistry, RouteCatalog};
    use std::fs;

    #[test]
    fn loads_definition_registry_from_files() {
        let dir =
            std::env::temp_dir().join(format!("inventory-runtime-defs-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir should be created");
        fs::write(
            dir.join("inventory.items.query.toml"),
            r#"
kind = "context_query_definition"
version = "1.0.0"
name = "inventory.items"
root_entity = "item"
description = "List items"
"#,
        )
        .expect("query should be written");
        fs::write(
            dir.join("inventory.item.create.action.toml"),
            r#"
kind = "action_definition"
version = "1.0.0"
name = "inventory.item.create"
description = "Create item"
context_queries = ["inventory.items"]
"#,
        )
        .expect("action should be written");
        fs::write(
            dir.join("inventory.item.list.view.toml"),
            r#"
kind = "view_definition"
version = "1.0.0"
name = "inventory.item.list"
entity_scope = "item"
params = []
context_queries = [{ query = "inventory.items", bind = "items" }]
layout = { type = "page", title = "Inventory", children = [
  { type = "action_bar", actions = ["inventory.item.create"] },
  { type = "table", rows = { bind = "$context.items.rows" }, columns = [
    { key = "name", header = "Name", value = { bind = "$row.name" }, editable = true, editor_kind = "label" }
  ] }
] }
interactions = [{ event = "create", action = "inventory.item.create" }]
"#,
        )
        .expect("view should be written");

        let registry = DefinitionRegistry::load_from_dir(&dir).expect("registry should load");
        assert_eq!(
            registry.query("inventory.items").expect("query").name,
            "inventory.items"
        );
        assert_eq!(
            registry
                .action("inventory.item.create")
                .expect("action")
                .name,
            "inventory.item.create"
        );
        assert_eq!(
            registry.view("inventory.item.list").expect("view").name,
            "inventory.item.list"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_route_catalog_from_files() {
        let dir =
            std::env::temp_dir().join(format!("inventory-runtime-routes-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("temp dir should be created");
        fs::write(
            dir.join("api.items.list.route.toml"),
            r#"
kind = "runtime_route"
version = "1.0.0"
name = "api.items.list"
target = "items.list"
"#,
        )
        .expect("route should be written");

        let catalog = RouteCatalog::load_from_dir(&dir).expect("route catalog should load");
        assert_eq!(
            catalog.route("api.items.list").expect("route").target,
            "items.list"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
