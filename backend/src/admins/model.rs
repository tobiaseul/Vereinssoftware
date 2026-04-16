// src/admins/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Admin {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>, // "Admin" | "SuperAdmin", defaults to "Admin"
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub password: String,
}
