use crate::schema::{Column, DataType, Index, PrimaryKey, VectorAppend};
use im::Vector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub name: String,
    pub columns: Vector<Column>,
    pub primary_key: PrimaryKey,
    pub indexes: Vector<Index>,
}

impl Table {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            columns: Vector::new(),
            primary_key: PrimaryKey::new("", std::iter::empty::<String>()),
            indexes: Vector::new(),
        }
    }

    pub fn column(self, name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            columns: self.columns.append(Column::new(name, data_type, false, None)),
            ..self
        }
    }

    pub fn column_nullable(self, name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            columns: self.columns.append(Column::new(name, data_type, true, None)),
            ..self
        }
    }

    pub fn column_default(self, name: impl Into<String>, data_type: DataType, default: impl Into<String>) -> Self {
        Self {
            columns: self.columns.append(Column::new(name, data_type, false, Some(default.into()))),
            ..self
        }
    }

    pub fn primary_key<T, C>(self, name: impl Into<String>, columns: C) -> Self
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        Self {
            primary_key: PrimaryKey::new(name, columns),
            ..self
        }
    }

    pub fn index<T, C>(self, name: impl Into<String>, columns: C) -> Self
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        Self {
            indexes: self.indexes.append(Index::new(name, columns, false)),
            ..self
        }
    }

    pub fn index_unique<T, C>(self, name: impl Into<String>, columns: C) -> Self
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        Self {
            indexes: self.indexes.append(Index::new(name, columns, true)),
            ..self
        }
    }

}
