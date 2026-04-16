use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FieldDefinition {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateFieldRequest {
    pub name: String,
    pub field_type: String, // "text" | "number" | "date" | "boolean"
    pub required: Option<bool>,
    pub display_order: Option<i32>,
}
