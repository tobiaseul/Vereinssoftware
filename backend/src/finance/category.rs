// src/finance/category.rs
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TransactionCategory {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

impl TransactionCategory {
    pub async fn list(pool: &PgPool) -> Result<Vec<TransactionCategory>, AppError> {
        let categories: Vec<TransactionCategory> = sqlx::query_as(
            "SELECT id, name, created_at FROM transaction_categories ORDER BY name"
        )
        .fetch_all(pool)
        .await?;

        Ok(categories)
    }

    pub async fn create(pool: &PgPool, name: String) -> Result<TransactionCategory, AppError> {
        let category: TransactionCategory = sqlx::query_as(
            "INSERT INTO transaction_categories (name) VALUES ($1) RETURNING id, name, created_at"
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        Ok(category)
    }

    pub async fn delete(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM transaction_categories WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
