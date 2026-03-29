use im::Vector;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    pub name: String,
    pub columns: Vector<String>,
    pub unique: bool,
}

impl Index {
    #[cfg(test)]
    pub fn new<T, C>(name: impl Into<String>, columns: C, unique: bool) -> Self
    where
        T: Into<String>,
        C: IntoIterator<Item = T>,
    {
        Self {
            name: name.into(),
            columns: columns.into_iter().map(Into::into).collect(),
            unique: unique,
        }
    }
}
