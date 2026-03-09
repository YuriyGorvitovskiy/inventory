use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::model::error::ModelError;
use crate::model::parser::{load_model, ParsedModel};

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    by_entity: HashMap<String, ParsedModel>,
}

impl ModelRegistry {
    pub fn load_from_dir(dir: &Path) -> Result<Self, ModelError> {
        let entries = fs::read_dir(dir)
            .map_err(|err| ModelError::new(format!("failed to read models dir {}: {err}", dir.display())))?;

        let mut files: Vec<PathBuf> = entries
            .filter_map(|entry| entry.ok().map(|e| e.path()))
            .filter(|path| path.is_file())
            .filter(|path| {
                path.file_name()
                    .and_then(|n| n.to_str())
                    .map(|name| name.ends_with(".model.toml"))
                    .unwrap_or(false)
            })
            .collect();

        files.sort();

        if files.is_empty() {
            return Err(ModelError::new(format!(
                "no model files (*.model.toml) found in {}",
                dir.display()
            )));
        }

        let mut by_entity = HashMap::new();
        for file in files {
            let parsed = load_model(&file)?;
            let name = parsed.model.entity.name.clone();
            if by_entity.insert(name.clone(), parsed).is_some() {
                return Err(ModelError::new(format!(
                    "duplicate entity model '{}' in {}",
                    name,
                    dir.display()
                )));
            }
        }

        Ok(Self { by_entity })
    }

    pub fn len(&self) -> usize {
        self.by_entity.len()
    }

    pub fn get(&self, entity_name: &str) -> Option<&ParsedModel> {
        self.by_entity.get(entity_name)
    }

    pub fn entities(&self) -> impl Iterator<Item = &str> {
        self.by_entity.keys().map(|k| k.as_str())
    }
}
