#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrmType {
    pub table: String,
    pub id_policy: IdPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdPolicy {
    ImplicitInt64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrmModel {
    pub entity: OrmType,
}
