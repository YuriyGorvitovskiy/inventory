use semver::Version;
use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub version: Version,
    pub entity: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub name: String,
    pub description: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
    pub destination_type: String,
    pub description: String,
    pub default: DefaultValue,
    pub indexed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    Boolean,
    Integer,
    Float,
    Timestamp,
    String,
    Text,
    Reference,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DefaultValue {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    Timestamp(String),
    String(String),
    Text(String),
    ReferenceId(i64),
}
