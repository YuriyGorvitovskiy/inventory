use crate::schema::{Table, VectorAppend};
use im::Vector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub name: String,
    pub tables: Vector<Table>,
}

impl Schema {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tables: Vector::new(),
        }
    }

    pub fn table(self, table: Table) -> Self {
        Self {
            tables: self.tables.append(table),
            ..self
        }
    }
}
