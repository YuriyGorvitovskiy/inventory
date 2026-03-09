use crate::model::types::FieldType;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RawModelFile {
    pub format_version: u32,
    pub version: String,
    pub entity: RawType,
}

#[derive(Debug, Deserialize)]
pub struct RawType {
    pub name: String,
    pub table: Option<String>,
    pub description: Option<String>,
    pub id_policy: Option<String>,
    pub fields: Vec<RawField>,
}

#[derive(Debug, Deserialize)]
pub struct RawField {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: FieldType,
    pub destination_type: Option<String>,
    pub description: Option<String>,
    pub default: RawDefault,
    pub indexed: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawDefault {
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}
