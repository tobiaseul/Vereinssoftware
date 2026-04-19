use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

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

    let account = sqlx::query_as_unchecked!(
        (Uuid, String, String, String),
        r#"INSERT INTO bank_accounts (name, iban, bank_name)
           VALUES ($1, $2, $3)
           RETURNING id, name, iban, bank_name"#,
        "Test Bank Account",
        "DE89370400440532013000",
        "Test Bank"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to create bank account");

    assert_eq!(account.1, "Test Bank Account");
    assert_eq!(account.2, "DE89370400440532013000");
    assert_eq!(account.3, "Test Bank");

    // Clean up after test
    sqlx::query("DELETE FROM bank_accounts WHERE name = 'Test Bank Account'")
        .execute(&pool)
        .await
        .ok();
}
