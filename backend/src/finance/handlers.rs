// src/finance/handlers.rs
// Note: This module is NOT exported from mod.rs to avoid lib.rs compilation issues.
// It's only used by main.rs which has access to auth and state modules.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, body::Body,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{error::AppError, state::AppState, auth::middleware::AuthClaims};

use super::{
    bank_account::{BankAccount, CreateBankAccountRequest, UpdateBankAccountRequest},
    finance_auth::{require_treasurer, require_finance_officer},
    transaction::Transaction,
    reconciliation::Reconciliation,
};
use chrono::NaiveDate;
use crate::services::reconciliation::StatementLine;

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

/// Response struct for transactions
#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub bank_account_id: Uuid,
    pub r#type: String,
    pub amount: String,
    pub date: NaiveDate,
    pub member_id: Option<Uuid>,
    pub category: String,
    pub reference: String,
    pub description: Option<String>,
    pub receipt_reference: Option<String>,
    pub reconciled: bool,
    pub version: i32,
}

impl From<Transaction> for TransactionResponse {
    fn from(tx: Transaction) -> Self {
        TransactionResponse {
            id: tx.id,
            bank_account_id: tx.bank_account_id,
            r#type: tx.r#type,
            amount: tx.amount,
            date: tx.date,
            member_id: tx.member_id,
            category: tx.category,
            reference: tx.reference,
            description: tx.description,
            receipt_reference: tx.receipt_reference,
            reconciled: tx.reconciled,
            version: tx.version,
        }
    }
}

/// Request struct for creating a transaction
#[derive(Debug, Deserialize)]
pub struct CreateTransactionRequest {
    pub r#type: String,
    pub amount: String,
    pub date: NaiveDate,
    pub member_id: Option<Uuid>,
    pub category: String,
    pub reference: String,
    pub description: Option<String>,
}

/// Request struct for updating a transaction
#[derive(Debug, Deserialize)]
pub struct UpdateTransactionRequest {
    pub version: i32,
    pub amount: String,
    pub date: NaiveDate,
    pub category: String,
    pub reference: String,
    pub description: Option<String>,
}

/// Query parameters for list transactions
#[derive(Debug, Deserialize)]
pub struct ListTransactionsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
    pub category: Option<String>,
    pub r#type: Option<String>,
    pub reconciled: Option<bool>,
}

fn default_limit() -> i64 {
    20
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

/// GET /api/v1/finance/accounts/:id/transactions
/// List all transactions for a bank account with pagination and optional filters
pub async fn list_transactions(
    State(state): State<AppState>,
    Path(account_id): Path<Uuid>,
    Query(query): Query<ListTransactionsQuery>,
) -> Result<Json<Vec<TransactionResponse>>, AppError> {
    // Verify account exists
    let _account = BankAccount::get_by_id(&state.db, account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    let transactions = Transaction::list_by_account(&state.db, account_id, query.limit, query.offset)
        .await?;

    // Filter by category, type, and reconciled status if provided
    let filtered: Vec<TransactionResponse> = transactions
        .into_iter()
        .filter(|tx| {
            if let Some(ref category_filter) = query.category {
                if tx.category != *category_filter {
                    return false;
                }
            }
            if let Some(ref type_filter) = query.r#type {
                if tx.r#type != *type_filter {
                    return false;
                }
            }
            if let Some(reconciled_filter) = query.reconciled {
                if tx.reconciled != reconciled_filter {
                    return false;
                }
            }
            true
        })
        .map(|tx| tx.into())
        .collect();

    Ok(Json(filtered))
}

/// POST /api/v1/finance/accounts/:id/transactions
/// Create a new transaction (requires Finance Officer+ role)
pub async fn create_transaction(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<(StatusCode, Json<TransactionResponse>), AppError> {
    require_finance_officer(&state.db, claims.0.sub)
        .await
        .map_err(|_| AppError::Forbidden)?;

    // Verify account exists
    let _account = BankAccount::get_by_id(&state.db, account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    // Validate amount > 0
    let amount_val: f64 = payload
        .amount
        .parse()
        .map_err(|_| AppError::Validation(vec![("amount".into(), "Invalid amount format".into())]))?;

    if amount_val <= 0.0 {
        return Err(AppError::Validation(vec![("amount".into(), "Amount must be greater than 0".into())]));
    }

    // Validate date not in future
    let today = chrono::Local::now().naive_local().date();
    if payload.date > today {
        return Err(AppError::Validation(vec![("date".into(), "Date cannot be in the future".into())]));
    }

    let transaction = Transaction::create(
        &state.db,
        account_id,
        payload.r#type,
        payload.amount,
        payload.date,
        payload.member_id,
        payload.category,
        payload.reference,
        payload.description,
    )
    .await?;

    let response: TransactionResponse = transaction.into();
    Ok((StatusCode::CREATED, Json(response)))
}

/// GET /api/v1/finance/transactions/:id
/// Get a specific transaction by ID
pub async fn get_transaction(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<TransactionResponse>, AppError> {
    let transaction = Transaction::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

    Ok(Json(transaction.into()))
}

/// PUT /api/v1/finance/transactions/:id
/// Update a transaction (requires Finance Officer+ role, returns 409 on version mismatch)
pub async fn update_transaction(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTransactionRequest>,
) -> Result<Json<TransactionResponse>, AppError> {
    require_finance_officer(&state.db, claims.0.sub)
        .await
        .map_err(|_| AppError::Forbidden)?;

    let transaction = Transaction::get_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

    // Validate amount > 0
    let amount_val: f64 = payload
        .amount
        .parse()
        .map_err(|_| AppError::Validation(vec![("amount".into(), "Invalid amount format".into())]))?;

    if amount_val <= 0.0 {
        return Err(AppError::Validation(vec![("amount".into(), "Amount must be greater than 0".into())]));
    }

    // Validate date not in future
    let today = chrono::Local::now().naive_local().date();
    if payload.date > today {
        return Err(AppError::Validation(vec![("date".into(), "Date cannot be in the future".into())]));
    }

    let updated = transaction
        .update(
            &state.db,
            payload.version,
            payload.amount,
            payload.date,
            payload.category,
            payload.reference,
            payload.description,
        )
        .await?
        .ok_or_else(|| {
            AppError::Conflict {
                current_version: transaction.version,
                submitted_version: payload.version,
            }
        })?;

    Ok(Json(updated.into()))
}

/// DELETE /api/v1/finance/transactions/:id
/// Soft delete a transaction - hard delete with ?hard=true (requires Finance Officer+ role)
pub async fn delete_transaction(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(id): Path<Uuid>,
    Query(query): Query<DeleteQuery>,
) -> Result<StatusCode, AppError> {
    require_finance_officer(&state.db, claims.0.sub)
        .await
        .map_err(|_| AppError::Forbidden)?;

    let found = if query.hard {
        Transaction::hard_delete(&state.db, id).await?
    } else {
        Transaction::soft_delete(&state.db, id).await?
    };

    if !found {
        return Err(AppError::NotFound("Transaction not found".into()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Receipt upload/download and reconciliation request/response structs

#[derive(Debug, Serialize)]
pub struct ReconciliationResponse {
    pub id: Uuid,
    pub bank_account_id: Uuid,
    pub statement_date: NaiveDate,
    pub status: String,
    pub matched_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct StartReconciliationRequest {
    pub statement_date: NaiveDate,
    pub file_name: String,
    pub statement_lines: Vec<StatementLineRequest>,
}

#[derive(Debug, Deserialize)]
pub struct StatementLineRequest {
    pub date: NaiveDate,
    pub amount: String,
    pub reference: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmReconciliationRequest {
    pub matched_transaction_ids: Vec<Uuid>,
}

// Receipt handlers

/// POST /api/v1/finance/transactions/:id/receipt
/// Upload a receipt file for a transaction (requires Finance Officer+ role)
pub async fn upload_receipt(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(transaction_id): Path<Uuid>,
    body: Body,
) -> Result<Json<TransactionResponse>, AppError> {
    require_finance_officer(&state.db, claims.0.sub)
        .await
        .map_err(|_| AppError::Forbidden)?;

    // Verify transaction exists
    let transaction = Transaction::get_by_id(&state.db, transaction_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

    // Read request body
    let bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to read request body: {}", e)))?;

    // Validate file size (10MB max)
    const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;
    if bytes.len() > MAX_FILE_SIZE {
        return Err(AppError::Validation(vec![(
            "file".into(),
            "File size exceeds 10MB limit".into(),
        )]));
    }

    // Use file storage from state to save the file
    let file_storage = &state.file_storage;
    let filename = format!("{}.receipt", transaction_id);
    let receipt_reference = file_storage
        .save(transaction_id, bytes.to_vec(), &filename)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Failed to save file: {}", e)))?;

    // Update transaction with receipt reference
    let updated = transaction
        .set_receipt_reference(&state.db, receipt_reference)
        .await?;

    Ok(Json(updated.into()))
}

/// GET /api/v1/finance/transactions/:id/receipt/:ref
/// Download a receipt file (returns raw bytes)
pub async fn download_receipt(
    State(state): State<AppState>,
    Path((transaction_id, receipt_ref)): Path<(Uuid, String)>,
) -> Result<Vec<u8>, AppError> {
    // Verify transaction exists
    let _transaction = Transaction::get_by_id(&state.db, transaction_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaction not found".into()))?;

    // Validate receipt_ref to prevent path traversal
    if receipt_ref.contains("..") || receipt_ref.contains("/") {
        return Err(AppError::Validation(vec![(
            "receipt_ref".into(),
            "Invalid receipt reference".into(),
        )]));
    }

    let reference_path = format!("receipts/{}/{}", transaction_id, receipt_ref);

    let file_storage = &state.file_storage;
    file_storage
        .get(&reference_path)
        .await
        .map_err(|e| AppError::NotFound(e.to_string()))
}

// Reconciliation handlers

/// POST /api/v1/finance/accounts/:id/reconciliation
/// Start a new reconciliation process (requires Treasurer role)
pub async fn start_reconciliation(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path(account_id): Path<Uuid>,
    Json(payload): Json<StartReconciliationRequest>,
) -> Result<(StatusCode, Json<ReconciliationResponse>), AppError> {
    require_treasurer(&state.db, claims.0.sub)
        .await
        .map_err(|_| AppError::Forbidden)?;

    // Verify account exists
    let _account = BankAccount::get_by_id(&state.db, account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    // Create reconciliation record
    let reconciliation = Reconciliation::create(
        &state.db,
        account_id,
        payload.statement_date,
        payload.file_name,
    )
    .await?;

    // Convert statement lines to StatementLine structs
    let statement_lines: Vec<StatementLine> = payload
        .statement_lines
        .into_iter()
        .map(|line| StatementLine {
            date: line.date,
            amount: line.amount,
            reference: line.reference,
        })
        .collect();

    // Call reconciliation service to match transactions
    let recon_service = &state.reconciliation_service;
    let matches = recon_service
        .match_transactions(statement_lines, account_id, &state.db)
        .await
        .map_err(|e| AppError::Internal(anyhow::anyhow!("Reconciliation failed: {}", e)))?;

    // Count matched transactions
    let matched_count = matches.iter().filter(|(_, tx_id)| tx_id.is_some()).count();

    let response = ReconciliationResponse {
        id: reconciliation.id,
        bank_account_id: reconciliation.bank_account_id,
        statement_date: reconciliation.statement_date,
        status: reconciliation.status,
        matched_count,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// PUT /api/v1/finance/accounts/:id/reconciliation/:rec-id
/// Confirm a reconciliation and mark transactions as reconciled (requires Treasurer role)
pub async fn confirm_reconciliation(
    State(state): State<AppState>,
    claims: AuthClaims,
    Path((account_id, reconciliation_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<ConfirmReconciliationRequest>,
) -> Result<Json<ReconciliationResponse>, AppError> {
    require_treasurer(&state.db, claims.0.sub)
        .await
        .map_err(|_| AppError::Forbidden)?;

    // Verify account exists
    let _account = BankAccount::get_by_id(&state.db, account_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Bank account not found".into()))?;

    // Verify reconciliation exists
    let reconciliation = Reconciliation::get_by_id(&state.db, reconciliation_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Reconciliation not found".into()))?;

    // Verify reconciliation belongs to the account
    if reconciliation.bank_account_id != account_id {
        return Err(AppError::NotFound("Reconciliation not found".into()));
    }

    // Update reconciliation status and matched transaction IDs
    let updated_reconciliation = Reconciliation::update_status(
        &state.db,
        reconciliation_id,
        "Complete".to_string(),
        payload.matched_transaction_ids.clone(),
    )
    .await?;

    // Mark each matched transaction as reconciled
    for transaction_id in payload.matched_transaction_ids {
        Transaction::set_reconciliation_id(&state.db, transaction_id, reconciliation_id).await?;
    }

    let response = ReconciliationResponse {
        id: updated_reconciliation.id,
        bank_account_id: updated_reconciliation.bank_account_id,
        statement_date: updated_reconciliation.statement_date,
        status: updated_reconciliation.status,
        matched_count: updated_reconciliation.matched_transaction_ids.len(),
    };

    Ok(Json(response))
}
