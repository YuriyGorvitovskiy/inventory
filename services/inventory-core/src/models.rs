use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
}

#[derive(Serialize)]
pub struct ReadinessResponse {
    pub status: &'static str,
    pub checks: [&'static str; 1],
}

#[derive(Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub owner_service: String,
    pub entity_type: String,
    pub name: String,
    pub category: String,
    pub quantity: i64,
}

#[derive(Serialize)]
pub struct ItemResponse {
    pub id: i64,
    pub entity_id: String,
    pub name: String,
    pub category: String,
    pub quantity: i64,
}

#[derive(Deserialize)]
pub struct CreateItemRequest {
    pub name: String,
    pub category: String,
    pub quantity: i64,
}

#[derive(Deserialize)]
pub struct UpdateItemRequest {
    pub name: String,
    pub category: String,
    pub quantity: i64,
}

pub fn item_to_response(tenant_id: &str, item: Item) -> ItemResponse {
    let entity_id = format!(
        "{}.{}.{}.{}",
        tenant_id, item.owner_service, item.entity_type, item.id
    );
    ItemResponse {
        id: item.id,
        entity_id,
        name: item.name,
        category: item.category,
        quantity: item.quantity,
    }
}
