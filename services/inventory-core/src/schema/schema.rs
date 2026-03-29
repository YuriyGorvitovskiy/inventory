#[cfg(test)]
use crate::schema::vector::VectorAppend;
use crate::schema::Table;
use im::Vector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Schema {
    pub name: String,
    pub tables: Vector<Table>,
}

impl Schema {
    #[cfg(test)]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            tables: Vector::new(),
        }
    }

    #[cfg(test)]
    pub fn table(self, table: Table) -> Self {
        Self {
            tables: self.tables.append(table),
            ..self
        }
    }
}
