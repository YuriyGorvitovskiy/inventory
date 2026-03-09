use crate::model::ModelRegistry;
use sqlx::PgPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub tenant_id: String,
    pub model_registry: Arc<ModelRegistry>,
}
