use crate::schema::DataType;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub default: Option<String>,
}

impl Column {
    pub fn new(name: impl Into<String>, data_type: DataType, nullable: bool, default: Option<String>) -> Self {
        Self {
            name: name.into(),
            data_type,
            nullable,
            default,
        }
    }
}
