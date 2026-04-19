// src/finance/handlers.rs
// Note: This module is NOT exported from mod.rs to avoid lib.rs compilation issues.
// It's only used by main.rs which has access to auth and state modules.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState, auth::middleware::AuthClaims};

use super::{
    bank_account::{BankAccount, CreateBankAccountRequest, UpdateBankAccountRequest},
    finance_auth::require_treasurer,
};

/// Response struct for bank accounts
#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub name: String,
    pub iban: String,
    pub bank_name: String,
    pub balance: String,
    pub is_active: bool,
}

impl From<BankAccount> for AccountResponse {
    fn from(account: BankAccount) -> Self {
        AccountResponse {
            id: account.id,
            name: account.name,
            iban: account.iban,
            bank_name: account.bank_name,
            balance: account.balance,
            is_active: account.is_active,
        }
    }
}

/// Query parameters for hard delete
#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub hard: bool,
}

/// GET /api/v1/finance/accounts
/// List all non-deleted bank accounts
pub async fn list_accounts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AccountResponse>>, AppError> {
    let accounts = BankAccount::list(&state.db).await?;
    let response: Vec<AccountResponse> = accounts.into_iter().map(|a| a.into()).collect();
    Ok(Json(response))
}

/// POST /api/v1/finance/accounts
/// Create a new bank account (requires Treasurer role)
pub async fn create_account(
    State(state): State<AppState>,
    claims: AuthClaims,
    Json(payload): Json<CreateBankAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), AppError> {
    require_treasurer(&state.db, claims.0.sub).await
        .map_err(|_| AppError::Forbidden)?;

    let account = BankAccount::create(&state.db, payload.name, payload.iban, payload.bank_name).await?;
    let response: AccountResponse = account.into();

    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/finance/accounts/:id
/// Get a specific bank account by ID
pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<AccountResponse>, AppError> {
    let account = BankAccount::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    Ok(Json(account.into()))
}

/// PUT /api/v1/finance/accounts/:id
/// Update a bank account (requires Treasurer role)
pub async fn update_account(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateBankAccountRequest>,
) -> Result<Json<AccountResponse>, AppError> {
    require_treasurer(&state.db, claims.0.sub).await
        .map_err(|_| AppError::Forbidden)?;

    let mut account = BankAccount::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    let updated = account
        .update(&state.db, payload.name, payload.bank_name, payload.is_active)
        .await?;

    Ok(Json(updated.into()))
}

/// DELETE /api/v1/finance/accounts/:id
/// Delete a bank account - soft delete by default, hard delete with ?hard=true (requires Treasurer role)
pub async fn delete_account(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteQuery>,
) -> Result<StatusCode, AppError> {
    require_treasurer(&state.db, claims.0.sub).await
        .map_err(|_| AppError::Forbidden)?;

    let found = if query.hard {
        BankAccount::hard_delete(&state.db, id).await?
    } else {
        BankAccount::soft_delete(&state.db, id).await?
    };

    if !found {
        return Err(AppError::NotFound("Bank account not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}
