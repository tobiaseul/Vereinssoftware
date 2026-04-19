// src/finance/transaction.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub bank_account_id: Uuid,
    pub version: i32,
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub amount: String,
    pub date: NaiveDate,
    pub member_id: Option<Uuid>,
    pub category: String,
    pub reference: String,
    pub description: Option<String>,
    pub receipt_reference: Option<String>,
    pub reconciled: bool,
    pub reconciliation_id: Option<Uuid>,
    pub transfer_pair_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    pub async fn create(
        pool: &PgPool,
        bank_account_id: Uuid,
        tx_type: String,
        amount: String,
        date: NaiveDate,
        member_id: Option<Uuid>,
        category: String,
        reference: String,
        description: Option<String>,
    ) -> Result<Transaction, AppError> {
        let transaction: Transaction = sqlx::query_as(
            "INSERT INTO transactions (bank_account_id, type, amount, date, member_id, category, reference, description)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             RETURNING id, bank_account_id, version, type, amount, date, member_id, category, reference, description, receipt_reference, reconciled, reconciliation_id, transfer_pair_id, deleted_at, created_at, updated_at"
        )
        .bind(bank_account_id)
        .bind(tx_type)
        .bind(amount)
        .bind(date)
        .bind(member_id)
        .bind(category)
        .bind(reference)
        .bind(description)
        .fetch_one(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Transaction>, AppError> {
        let transaction: Option<Transaction> = sqlx::query_as(
            "SELECT id, bank_account_id, version, type, amount, date, member_id, category, reference, description, receipt_reference, reconciled, reconciliation_id, transfer_pair_id, deleted_at, created_at, updated_at
             FROM transactions
             WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn list_by_account(
        pool: &PgPool,
        bank_account_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>, AppError> {
        let transactions: Vec<Transaction> = sqlx::query_as(
            "SELECT id, bank_account_id, version, type, amount, date, member_id, category, reference, description, receipt_reference, reconciled, reconciliation_id, transfer_pair_id, deleted_at, created_at, updated_at
             FROM transactions
             WHERE bank_account_id = $1 AND deleted_at IS NULL
             ORDER BY date DESC
             LIMIT $2 OFFSET $3"
        )
        .bind(bank_account_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        Ok(transactions)
    }

    pub async fn update(
        &self,
        pool: &PgPool,
        expected_version: i32,
        amount: String,
        date: NaiveDate,
        category: String,
        reference: String,
        description: Option<String>,
    ) -> Result<Option<Transaction>, AppError> {
        let transaction: Option<Transaction> = sqlx::query_as(
            "UPDATE transactions
             SET amount = $2,
                 date = $3,
                 category = $4,
                 reference = $5,
                 description = $6,
                 version = version + 1,
                 updated_at = NOW()
             WHERE id = $1 AND version = $7 AND deleted_at IS NULL
             RETURNING id, bank_account_id, version, type, amount, date, member_id, category, reference, description, receipt_reference, reconciled, reconciliation_id, transfer_pair_id, deleted_at, created_at, updated_at"
        )
        .bind(self.id)
        .bind(amount)
        .bind(date)
        .bind(category)
        .bind(reference)
        .bind(description)
        .bind(expected_version)
        .fetch_optional(pool)
        .await?;

        Ok(transaction)
    }

    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            "UPDATE transactions SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn hard_delete(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM transactions WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_reconciliation_id(
        pool: &PgPool,
        id: Uuid,
        reconciliation_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            "UPDATE transactions SET reconciliation_id = $2, reconciled = true WHERE id = $1"
        )
        .bind(id)
        .bind(reconciliation_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
