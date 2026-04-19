use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::{ReconciliationError, ReconciliationService, StatementLine};

pub struct ManualReconciliationService;

#[async_trait]
impl ReconciliationService for ManualReconciliationService {
    async fn match_transactions(
        &self,
        statement_lines: Vec<StatementLine>,
        account_id: Uuid,
        pool: &PgPool,
    ) -> Result<Vec<(StatementLine, Option<Uuid>)>, ReconciliationError> {
        // Fetch unreconciled transactions for this account
        let unreconciled_transactions: Vec<(Uuid, String, String)> = sqlx::query_as(
            "SELECT id, amount, date::TEXT FROM transactions WHERE bank_account_id = $1 AND reconciled = false AND deleted_at IS NULL"
        )
        .bind(account_id)
        .fetch_all(pool)
        .await
        .map_err(|e| ReconciliationError::DatabaseError(e.to_string()))?;

        // For each statement line, try to find a matching transaction
        let mut results = Vec::new();

        for statement_line in statement_lines {
            let matched_transaction_id = unreconciled_transactions
                .iter()
                .find(|(_, amount, date)| {
                    amount == &statement_line.amount && date == &statement_line.date.to_string()
                })
                .map(|(id, _, _)| *id);

            results.push((statement_line, matched_transaction_id));
        }

        Ok(results)
    }
}
