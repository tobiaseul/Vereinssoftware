// src/finance/bank_account.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BankAccount {
    pub id: Uuid,
    pub name: String,
    pub iban: String,
    pub bank_name: String,
    #[sqlx(default)]
    #[serde(serialize_with = "serialize_decimal")]
    pub balance: sqlx::types::BigDecimal,
    pub is_active: bool,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn serialize_decimal<S>(value: &sqlx::types::BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_f64(value.to_string().parse().unwrap_or(0.0))
}

#[derive(Debug, Deserialize)]
pub struct CreateBankAccountRequest {
    pub name: String,
    pub iban: String,
    pub bank_name: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateBankAccountRequest {
    pub name: Option<String>,
    pub bank_name: Option<String>,
    pub is_active: Option<bool>,
}

impl BankAccount {
    pub async fn create(
        pool: &PgPool,
        name: String,
        iban: String,
        bank_name: String,
    ) -> Result<BankAccount, AppError> {
        let account: BankAccount = sqlx::query_as(
            "INSERT INTO bank_accounts (name, iban, bank_name)
             VALUES ($1, $2, $3)
             RETURNING id, name, iban, bank_name, balance, is_active, deleted_at, created_at, updated_at"
        )
        .bind(name)
        .bind(iban)
        .bind(bank_name)
        .fetch_one(pool)
        .await?;

        Ok(account)
    }

    pub async fn list(pool: &PgPool) -> Result<Vec<BankAccount>, AppError> {
        let accounts: Vec<BankAccount> = sqlx::query_as(
            "SELECT id, name, iban, bank_name, balance, is_active, deleted_at, created_at, updated_at
             FROM bank_accounts
             WHERE deleted_at IS NULL
             ORDER BY name"
        )
        .fetch_all(pool)
        .await?;

        Ok(accounts)
    }

    pub async fn get_by_id(pool: &PgPool, id: Uuid) -> Result<Option<BankAccount>, AppError> {
        let account: Option<BankAccount> = sqlx::query_as(
            "SELECT id, name, iban, bank_name, balance, is_active, deleted_at, created_at, updated_at
             FROM bank_accounts
             WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;

        Ok(account)
    }

    pub async fn update(
        &mut self,
        pool: &PgPool,
        name: Option<String>,
        bank_name: Option<String>,
        is_active: Option<bool>,
    ) -> Result<BankAccount, AppError> {
        let account: BankAccount = sqlx::query_as(
            "UPDATE bank_accounts
             SET name = COALESCE($2, name),
                 bank_name = COALESCE($3, bank_name),
                 is_active = COALESCE($4, is_active),
                 updated_at = NOW()
             WHERE id = $1 AND deleted_at IS NULL
             RETURNING id, name, iban, bank_name, balance, is_active, deleted_at, created_at, updated_at"
        )
        .bind(self.id)
        .bind(name)
        .bind(bank_name)
        .bind(is_active)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

        *self = account.clone();
        Ok(account)
    }

    pub async fn soft_delete(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            "UPDATE bank_accounts SET deleted_at = NOW() WHERE id = $1 AND deleted_at IS NULL"
        )
        .bind(id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn hard_delete(pool: &PgPool, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM bank_accounts WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}
