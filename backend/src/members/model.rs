// src/members/model.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Member {
    pub id: Uuid,
    pub version: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub membership_type: String,
    pub joined_at: NaiveDate,
    pub left_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub custom_fields: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateMemberRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub membership_type: String,
    pub joined_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub custom_fields: Option<Value>,
}

#[derive(Deserialize)]
pub struct UpdateMemberRequest {
    pub version: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub membership_type: String,
    pub joined_at: NaiveDate,
    pub left_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub custom_fields: Option<Value>,
}

#[derive(Deserialize)]
pub struct MemberListQuery {
    pub search: Option<String>,
    pub membership_type: Option<String>,
    pub include_left: Option<bool>,
}
