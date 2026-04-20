use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use std::path::PathBuf;
use vereinssoftware_backend::finance::{BankAccount, Transaction, Reconciliation};
use vereinssoftware_backend::storage::{LocalFileStorage, FileStorage};

// Note: These tests require a test database. Configure via DATABASE_URL environment variable.
// Run with: cargo test --test test_integration -- --ignored

#[tokio::test]
#[ignore]
async fn test_complete_financial_workflow() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM transactions WHERE reference LIKE 'WORKFLOW-TEST-%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Workflow Test Account'")
        .execute(&pool)
        .await
        .ok();

    // 1. Create bank account
    let account = BankAccount::create(
        &pool,
        "Workflow Test Account".to_string(),
        "DE89370400440532013100".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    assert_eq!(account.name, "Workflow Test Account");
    assert!(account.deleted_at.is_none());

    // 2. Create income transaction
    let income = Transaction::create(
        &pool,
        account.id,
        "Income".to_string(),
        "1000.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
        None,
        "Donation".to_string(),
        "WORKFLOW-TEST-INCOME-001".to_string(),
        Some("Monthly donation".to_string()),
    )
    .await
    .expect("Failed to create income transaction");

    assert_eq!(income.r#type, "Income");
    assert_eq!(income.amount, "1000.00");
    assert_eq!(income.version, 1);

    // 3. Create expense transaction
    let expense = Transaction::create(
        &pool,
        account.id,
        "Expense".to_string(),
        "250.50".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 20).unwrap(),
        None,
        "Supplies".to_string(),
        "WORKFLOW-TEST-EXPENSE-001".to_string(),
        Some("Office supplies".to_string()),
    )
    .await
    .expect("Failed to create expense transaction");

    assert_eq!(expense.r#type, "Expense");
    assert_eq!(expense.amount, "250.50");

    // 4. List transactions
    let transactions = Transaction::list_by_account(&pool, account.id, 10, 0)
        .await
        .expect("Failed to list transactions");

    assert_eq!(transactions.len(), 2);
    assert!(transactions.iter().any(|t| t.id == income.id));
    assert!(transactions.iter().any(|t| t.id == expense.id));

    // 5. Soft delete transaction
    let deleted = Transaction::soft_delete(&pool, expense.id)
        .await
        .expect("Failed to soft delete");

    assert!(deleted);

    // 6. Verify soft-deleted excluded from list
    let updated_transactions = Transaction::list_by_account(&pool, account.id, 10, 0)
        .await
        .expect("Failed to list transactions");

    assert_eq!(updated_transactions.len(), 1);
    assert!(updated_transactions.iter().all(|t| t.id != expense.id));

    // 7. Verify get_by_id returns None for soft-deleted
    let deleted_by_id = Transaction::get_by_id(&pool, expense.id)
        .await
        .expect("Failed to get transaction");

    assert!(deleted_by_id.is_none());

    // 8. Verify version-based concurrency (update with wrong version returns None)
    let updated_income = income
        .update(
            &pool,
            2, // Wrong version (current is 1)
            "1500.00".to_string(),
            NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
            "Donation".to_string(),
            "WORKFLOW-TEST-INCOME-001-UPDATED".to_string(),
            None,
        )
        .await
        .expect("Failed to update");

    assert!(updated_income.is_none());

    // 9. Update with correct version succeeds
    let correctly_updated = income
        .update(
            &pool,
            1, // Correct version
            "1500.00".to_string(),
            NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
            "Donation".to_string(),
            "WORKFLOW-TEST-INCOME-001-UPDATED".to_string(),
            None,
        )
        .await
        .expect("Failed to update");

    assert!(correctly_updated.is_some());
    let updated = correctly_updated.unwrap();
    assert_eq!(updated.amount, "1500.00");
    assert_eq!(updated.version, 2);

    // Clean up after test
    sqlx::query("DELETE FROM transactions WHERE reference LIKE 'WORKFLOW-TEST-%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Workflow Test Account'")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn test_soft_delete_filtering() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Soft Delete Test Account'")
        .execute(&pool)
        .await
        .ok();

    // Create account
    let account = BankAccount::create(
        &pool,
        "Soft Delete Test Account".to_string(),
        "DE89370400440532013101".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    // Verify it's in list
    let accounts = BankAccount::list(&pool)
        .await
        .expect("Failed to list accounts");

    assert!(accounts.iter().any(|a| a.id == account.id));

    // Soft delete it
    let deleted = BankAccount::soft_delete(&pool, account.id)
        .await
        .expect("Failed to soft delete");

    assert!(deleted);

    // Verify it's excluded from list
    let updated_accounts = BankAccount::list(&pool)
        .await
        .expect("Failed to list accounts");

    assert!(!updated_accounts.iter().any(|a| a.id == account.id));

    // Verify get_by_id with soft-deleted returns None
    let deleted_by_id = BankAccount::get_by_id(&pool, account.id)
        .await
        .expect("Failed to get account");

    assert!(deleted_by_id.is_none());

    // Clean up after test
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Soft Delete Test Account'")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn test_receipt_upload_and_download() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Set up storage in a temporary directory
    let temp_dir = PathBuf::from("/tmp/receipt_test");
    let _ = std::fs::create_dir_all(&temp_dir);
    let storage = LocalFileStorage::new(temp_dir.clone());

    // Clean up before test
    sqlx::query("DELETE FROM transactions WHERE reference = 'RECEIPT-TEST-001'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Receipt Test Account'")
        .execute(&pool)
        .await
        .ok();

    // Create transaction
    let account = BankAccount::create(
        &pool,
        "Receipt Test Account".to_string(),
        "DE89370400440532013102".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    let transaction = Transaction::create(
        &pool,
        account.id,
        "Expense".to_string(),
        "50.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
        None,
        "Supplies".to_string(),
        "RECEIPT-TEST-001".to_string(),
        None,
    )
    .await
    .expect("Failed to create transaction");

    // Save receipt to storage
    let test_content = b"Receipt PDF content";
    let receipt_reference = storage
        .save(transaction.id, test_content.to_vec(), "receipt.pdf")
        .await
        .expect("Failed to save receipt");

    assert!(receipt_reference.contains(&transaction.id.to_string()));
    assert!(receipt_reference.contains("receipt.pdf"));

    // Update transaction with receipt reference
    let updated_tx = transaction
        .set_receipt_reference(&pool, receipt_reference.clone())
        .await
        .expect("Failed to set receipt reference");

    assert_eq!(updated_tx.receipt_reference, Some(receipt_reference.clone()));

    // Retrieve receipt
    let retrieved_content = storage
        .get(&receipt_reference)
        .await
        .expect("Failed to retrieve receipt");

    assert_eq!(retrieved_content, test_content);

    // Delete receipt
    storage
        .delete(&receipt_reference)
        .await
        .expect("Failed to delete receipt");

    // Verify it's deleted
    let result = storage.get(&receipt_reference).await;
    assert!(result.is_err());

    // Clean up after test
    sqlx::query("DELETE FROM transactions WHERE reference = 'RECEIPT-TEST-001'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Receipt Test Account'")
        .execute(&pool)
        .await
        .ok();
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
#[ignore]
async fn test_reconciliation_matching() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM reconciliations WHERE file_name = 'reconciliation_test.csv'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM transactions WHERE reference LIKE 'RECONCILE-TEST-%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Reconciliation Test Account'")
        .execute(&pool)
        .await
        .ok();

    // Create account
    let account = BankAccount::create(
        &pool,
        "Reconciliation Test Account".to_string(),
        "DE89370400440532013103".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    // Create transactions with different amounts and dates
    let tx1 = Transaction::create(
        &pool,
        account.id,
        "Income".to_string(),
        "100.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 15).unwrap(),
        None,
        "Donation".to_string(),
        "RECONCILE-TEST-001".to_string(),
        None,
    )
    .await
    .expect("Failed to create transaction 1");

    let tx2 = Transaction::create(
        &pool,
        account.id,
        "Expense".to_string(),
        "50.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 16).unwrap(),
        None,
        "Supplies".to_string(),
        "RECONCILE-TEST-002".to_string(),
        None,
    )
    .await
    .expect("Failed to create transaction 2");

    let tx3 = Transaction::create(
        &pool,
        account.id,
        "Income".to_string(),
        "200.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 17).unwrap(),
        None,
        "Grant".to_string(),
        "RECONCILE-TEST-003".to_string(),
        None,
    )
    .await
    .expect("Failed to create transaction 3");

    // Create reconciliation
    let reconciliation = Reconciliation::create(
        &pool,
        account.id,
        NaiveDate::from_ymd_opt(2025, 4, 17).unwrap(),
        "reconciliation_test.csv".to_string(),
    )
    .await
    .expect("Failed to create reconciliation");

    assert_eq!(reconciliation.status, "pending");
    assert_eq!(reconciliation.matched_transaction_ids.len(), 0);

    // Update reconciliation with matched transactions
    let matched_ids = vec![tx1.id, tx2.id];
    let updated_reconciliation = Reconciliation::update_status(
        &pool,
        reconciliation.id,
        "completed".to_string(),
        matched_ids.clone(),
    )
    .await
    .expect("Failed to update reconciliation");

    assert_eq!(updated_reconciliation.status, "completed");
    assert_eq!(updated_reconciliation.matched_transaction_ids, matched_ids);

    // Verify unmatched transaction (tx3) is still available
    let tx3_retrieved = Transaction::get_by_id(&pool, tx3.id)
        .await
        .expect("Failed to retrieve transaction")
        .expect("Transaction should exist");

    assert!(!tx3_retrieved.reconciled);

    // Clean up after test
    sqlx::query("DELETE FROM reconciliations WHERE file_name = 'reconciliation_test.csv'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM transactions WHERE reference LIKE 'RECONCILE-TEST-%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Reconciliation Test Account'")
        .execute(&pool)
        .await
        .ok();
}

#[tokio::test]
#[ignore]
async fn test_hard_delete_requires_cleanup() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM transactions WHERE reference LIKE 'HARD-DELETE-TEST-%'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Hard Delete Test Account'")
        .execute(&pool)
        .await
        .ok();

    // Create account with transactions
    let account = BankAccount::create(
        &pool,
        "Hard Delete Test Account".to_string(),
        "DE89370400440532013104".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    let tx1 = Transaction::create(
        &pool,
        account.id,
        "Income".to_string(),
        "100.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
        None,
        "Donation".to_string(),
        "HARD-DELETE-TEST-001".to_string(),
        None,
    )
    .await
    .expect("Failed to create transaction 1");

    let tx2 = Transaction::create(
        &pool,
        account.id,
        "Expense".to_string(),
        "50.00".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
        None,
        "Supplies".to_string(),
        "HARD-DELETE-TEST-002".to_string(),
        None,
    )
    .await
    .expect("Failed to create transaction 2");

    // Soft delete all transactions
    Transaction::soft_delete(&pool, tx1.id)
        .await
        .expect("Failed to soft delete tx1");
    Transaction::soft_delete(&pool, tx2.id)
        .await
        .expect("Failed to soft delete tx2");

    // Hard delete account should succeed after cleanup
    let hard_deleted = BankAccount::hard_delete(&pool, account.id)
        .await
        .expect("Failed to hard delete account");

    assert!(hard_deleted);

    // Verify account is gone
    let deleted_account = BankAccount::get_by_id(&pool, account.id)
        .await
        .expect("Failed to get account");

    assert!(deleted_account.is_none());

    // Clean up remaining transactions after test
    sqlx::query("DELETE FROM transactions WHERE reference LIKE 'HARD-DELETE-TEST-%'")
        .execute(&pool)
        .await
        .ok();
}
