use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use vereinssoftware_backend::finance::{BankAccount, Reconciliation};

// Note: These tests require a test database. Configure via DATABASE_URL environment variable.
// Run with: cargo test --test test_reconciliation -- --ignored

#[tokio::test]
#[ignore]
async fn test_reconciliation_create() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM reconciliations WHERE file_name = 'test_statement.csv'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank for Reconciliation'")
        .execute(&pool)
        .await
        .ok();

    // Create a bank account first
    let account = BankAccount::create(
        &pool,
        "Test Bank for Reconciliation".to_string(),
        "DE89370400440532013002".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    // Create a reconciliation
    let reconciliation = Reconciliation::create(
        &pool,
        account.id,
        NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
        "test_statement.csv".to_string(),
    )
    .await
    .expect("Failed to create reconciliation");

    // Verify the created reconciliation has correct values
    assert_eq!(reconciliation.bank_account_id, account.id);
    assert_eq!(reconciliation.statement_date, NaiveDate::from_ymd_opt(2025, 4, 19).unwrap());
    assert_eq!(reconciliation.file_name, "test_statement.csv");
    assert_eq!(reconciliation.status, "pending");
    assert_eq!(reconciliation.matched_transaction_ids, Vec::new());

    // Clean up after test
    sqlx::query("DELETE FROM reconciliations WHERE file_name = 'test_statement.csv'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank for Reconciliation'")
        .execute(&pool)
        .await
        .ok();
}
