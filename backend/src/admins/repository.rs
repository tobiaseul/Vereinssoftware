// src/admins/repository.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::Admin;

pub async fn list_admins(db: &PgPool) -> Result<Vec<Admin>, AppError> {
    let admins = sqlx::query_as!(
        Admin,
        r#"SELECT id, username, role as "role: String", created_at FROM admins ORDER BY created_at"#
    )
    .fetch_all(db)
    .await?;
    Ok(admins)
}

pub async fn create_admin(db: &PgPool, username: &str, password_hash: &str, role: &str) -> Result<Admin, AppError> {
    let admin = sqlx::query_as_unchecked!(
        Admin,
        r#"INSERT INTO admins (username, password_hash, role)
           VALUES ($1, $2, $3::admin_role)
           RETURNING id, username, role, created_at"#,
        username, password_hash, role
    )
    .fetch_one(db)
    .await?;
    Ok(admin)
}

pub async fn delete_admin(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query!("DELETE FROM admins WHERE id = $1", id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_password(db: &PgPool, id: Uuid, password_hash: &str) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "UPDATE admins SET password_hash = $1 WHERE id = $2",
        password_hash, id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
