//! Usage: cargo run --bin seed -- --username admin --password secret --role SuperAdmin
use sqlx::postgres::PgPoolOptions;
use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().collect();
    let username = args.iter().position(|a| a == "--username")
        .map(|i| args[i + 1].clone()).unwrap_or("admin".into());
    let password = args.iter().position(|a| a == "--password")
        .map(|i| args[i + 1].clone()).expect("--password required");
    let role = args.iter().position(|a| a == "--role")
        .map(|i| args[i + 1].clone()).unwrap_or("SuperAdmin".into());

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt).unwrap().to_string();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let pool = PgPoolOptions::new().connect(&db_url).await.unwrap();

    sqlx::query_unchecked!(
        "INSERT INTO admins (username, password_hash, role) VALUES ($1, $2, $3::admin_role) ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash",
        username, hash, role
    )
    .execute(&pool)
    .await
    .unwrap();

    println!("Admin '{username}' ({role}) created/updated.");
}
