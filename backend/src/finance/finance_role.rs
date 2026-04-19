// src/finance/finance_role.rs
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FinanceRole {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AdminFinanceRole {
    pub admin_id: Uuid,
    pub finance_role_id: Uuid,
}

impl AdminFinanceRole {
    pub async fn assign(
        pool: &PgPool,
        admin_id: Uuid,
        finance_role_id: Uuid,
    ) -> Result<AdminFinanceRole, AppError> {
        let admin_finance_role: AdminFinanceRole = sqlx::query_as(
            "INSERT INTO admin_finance_roles (admin_id, finance_role_id)
             VALUES ($1, $2)
             RETURNING admin_id, finance_role_id"
        )
        .bind(admin_id)
        .bind(finance_role_id)
        .fetch_one(pool)
        .await?;

        Ok(admin_finance_role)
    }

    pub async fn list_by_admin(
        pool: &PgPool,
        admin_id: Uuid,
    ) -> Result<Vec<FinanceRole>, AppError> {
        let roles: Vec<FinanceRole> = sqlx::query_as(
            "SELECT fr.id, fr.name
             FROM finance_roles fr
             INNER JOIN admin_finance_roles afr ON fr.id = afr.finance_role_id
             WHERE afr.admin_id = $1"
        )
        .bind(admin_id)
        .fetch_all(pool)
        .await?;

        Ok(roles)
    }

    pub async fn remove(
        pool: &PgPool,
        admin_id: Uuid,
        finance_role_id: Uuid,
    ) -> Result<bool, AppError> {
        let result = sqlx::query(
            "DELETE FROM admin_finance_roles WHERE admin_id = $1 AND finance_role_id = $2"
        )
        .bind(admin_id)
        .bind(finance_role_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }
}
