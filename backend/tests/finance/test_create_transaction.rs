use chrono::NaiveDate;
use sqlx::postgres::PgPoolOptions;
use vereinssoftware_backend::finance::{BankAccount, Transaction};

// Note: These tests require a test database. Configure via DATABASE_URL environment variable.
// Run with: cargo test --test test_create_transaction -- --ignored

#[tokio::test]
#[ignore]
async fn test_transaction_create() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM transactions WHERE reference = 'TEST-TRANSACTION-001'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank for Transaction'")
        .execute(&pool)
        .await
        .ok();

    // Create a bank account first
    let account = BankAccount::create(
        &pool,
        "Test Bank for Transaction".to_string(),
        "DE89370400440532013001".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    // Create a transaction
    let transaction = Transaction::create(
        &pool,
        account.id,
        "Income".to_string(),
        "100.50".to_string(),
        NaiveDate::from_ymd_opt(2025, 4, 19).unwrap(),
        None,
        "Donation".to_string(),
        "TEST-TRANSACTION-001".to_string(),
        Some("Test transaction description".to_string()),
    )
    .await
    .expect("Failed to create transaction");

    // Verify the created transaction has correct values
    assert_eq!(transaction.bank_account_id, account.id);
    assert_eq!(transaction.r#type, "Income");
    assert_eq!(transaction.amount, "100.50");
    assert_eq!(transaction.category, "Donation");
    assert_eq!(transaction.reference, "TEST-TRANSACTION-001");
    assert_eq!(transaction.description, Some("Test transaction description".to_string()));
    assert_eq!(transaction.version, 1);
    assert!(!transaction.reconciled);
    assert!(transaction.deleted_at.is_none());
    assert!(transaction.member_id.is_none());

    // Clean up after test
    sqlx::query("DELETE FROM transactions WHERE reference = 'TEST-TRANSACTION-001'")
        .execute(&pool)
        .await
        .ok();
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank for Transaction'")
        .execute(&pool)
        .await
        .ok();
}
