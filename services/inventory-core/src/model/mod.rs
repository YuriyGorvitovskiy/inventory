mod error;
mod mapping;
pub mod model;
mod parser;
mod raw;
pub mod registry;

pub(crate) use parser::ParsedModel;
pub(crate) use error::ModelError;
pub(crate) use registry::ModelRegistry;
pub(crate) use mapping::EntityMapping;

#[cfg(test)]
pub(crate) use model::DefaultValue;
#[cfg(test)]
pub(crate) use parser::{load_model, parse_model};
#[cfg(test)]
pub(crate) use mapping::normalize_identifier;

#[cfg(test)]
mod tests;
