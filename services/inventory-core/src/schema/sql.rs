#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SqlParameter {
    String(String),
    Int64(i64),
    Boolean(bool),
    Timestamp(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlStatement {
    pub sql: String,
    pub parameters: Vec<SqlParameter>,
}

impl SqlStatement {
    pub fn new(sql: impl Into<String>) -> Self {
        Self {
            sql: sql.into(),
            parameters: Vec::new(),
        }
    }
}
