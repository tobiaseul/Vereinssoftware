// src/finance/reconciliation.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Reconciliation {
    pub id: Uuid,
    pub bank_account_id: Uuid,
    pub statement_date: NaiveDate,
    pub file_name: String,
    pub matched_transaction_ids: Vec<Uuid>,
    pub status: String,
    pub uploaded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Reconciliation {
    pub async fn create(
        pool: &PgPool,
        bank_account_id: Uuid,
        statement_date: NaiveDate,
        file_name: String,
    ) -> Result<Reconciliation, AppError> {
        let reconciliation: Reconciliation = sqlx::query_as(
            "INSERT INTO reconciliations (bank_account_id, statement_date, file_name)
             VALUES ($1, $2, $3)
             RETURNING id, bank_account_id, statement_date, file_name, matched_transaction_ids, status, uploaded_at, created_at"
        )
        .bind(bank_account_id)
        .bind(statement_date)
        .bind(file_name)
        .fetch_one(pool)
        .await?;

        Ok(reconciliation)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Reconciliation>, AppError> {
        let reconciliation: Option<Reconciliation> = sqlx::query_as(
            "SELECT id, bank_account_id, statement_date, file_name, matched_transaction_ids, status, uploaded_at, created_at
             FROM reconciliations
             WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(reconciliation)
    }

    pub async fn list_by_account(
        pool: &PgPool,
        bank_account_id: Uuid,
    ) -> Result<Vec<Reconciliation>, AppError> {
        let reconciliations: Vec<Reconciliation> = sqlx::query_as(
            "SELECT id, bank_account_id, statement_date, file_name, matched_transaction_ids, status, uploaded_at, created_at
             FROM reconciliations
             WHERE bank_account_id = $1
             ORDER BY created_at DESC"
        )
        .bind(bank_account_id)
        .fetch_all(pool)
        .await?;

        Ok(reconciliations)
    }

    pub async fn update_status(
        pool: &PgPool,
        id: Uuid,
        status: String,
        matched_transaction_ids: Vec<Uuid>,
    ) -> Result<Reconciliation, AppError> {
        let reconciliation: Reconciliation = sqlx::query_as(
            "UPDATE reconciliations
             SET status = $2,
                 matched_transaction_ids = $3
             WHERE id = $1
             RETURNING id, bank_account_id, statement_date, file_name, matched_transaction_ids, status, uploaded_at, created_at"
        )
        .bind(id)
        .bind(status)
        .bind(matched_transaction_ids)
        .fetch_one(pool)
        .await?;

        Ok(reconciliation)
    }
}
