use crate::schema::VectorAppend;
use im::Vector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryKey {
    pub name: String,
    pub columns: Vector<String>,
}

impl PrimaryKey {
    pub fn new<T, C>(name: impl Into<String>, columns: C) -> Self
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
        }
    }

    pub fn column(self, name: impl Into<String>) -> Self {
        Self {
            columns: self.columns.append(name.into()),
            ..self
        }
    }
}
