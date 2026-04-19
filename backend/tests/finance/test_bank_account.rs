use sqlx::postgres::PgPoolOptions;
use vereinssoftware_backend::finance::BankAccount;

// Note: These tests require a test database. Configure via DATABASE_URL environment variable.
// Run with: cargo test --test test_bank_account -- --ignored

#[tokio::test]
#[ignore]
async fn test_bank_account_create() {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Clean up before test
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank Account'")
        .execute(&pool)
        .await
        .ok();

    // Use the actual BankAccount::create method
    let account = BankAccount::create(
        &pool,
        "Test Bank Account".to_string(),
        "DE89370400440532013000".to_string(),
        "Test Bank".to_string(),
    )
    .await
    .expect("Failed to create bank account");

    // Verify the created account has correct values
    assert_eq!(account.name, "Test Bank Account");
    assert_eq!(account.iban, "DE89370400440532013000");
    assert_eq!(account.bank_name, "Test Bank");
    assert!(account.deleted_at.is_none());

    // Clean up after test
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank Account'")
        .execute(&pool)
        .await
        .ok();
}
