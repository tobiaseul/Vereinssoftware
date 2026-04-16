// backend/src/field_definitions/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB row — used internally for sqlx queries
#[derive(Debug, sqlx::FromRow)]
pub struct FieldDefinitionRow {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

/// API response — includes nested options
#[derive(Debug, Serialize)]
pub struct FieldDefinition {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub options: Vec<FieldOption>,
}

impl FieldDefinition {
    pub fn from_row(row: FieldDefinitionRow, options: Vec<FieldOption>) -> Self {
        Self {
            id: row.id,
            name: row.name,
            field_type: row.field_type,
            required: row.required,
            display_order: row.display_order,
            created_at: row.created_at,
            options,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FieldOption {
    pub id: Uuid,
    pub field_definition_id: Uuid,
    pub value: String,
    pub display_order: i32,
}

#[derive(Deserialize)]
pub struct CreateFieldRequest {
    pub name: String,
    pub field_type: String,
    pub required: Option<bool>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateFieldRequest {
    pub name: Option<String>,
    pub required: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateOptionRequest {
    pub value: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateOptionRequest {
    pub value: Option<String>,
    pub display_order: Option<i32>,
}
