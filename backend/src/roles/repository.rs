use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::Role;

pub async fn list_roles(db: &PgPool) -> Result<Vec<Role>, AppError> {
    Ok(sqlx::query_as!(Role, "SELECT id, name, created_at FROM roles ORDER BY name")
        .fetch_all(db).await?)
}

pub async fn create_role(db: &PgPool, name: &str) -> Result<Role, AppError> {
    Ok(sqlx::query_as!(Role,
        "INSERT INTO roles (name) VALUES ($1) RETURNING id, name, created_at", name)
        .fetch_one(db).await?)
}

pub async fn delete_role(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query!("DELETE FROM roles WHERE id = $1", id)
        .execute(db).await?.rows_affected() > 0)
}
