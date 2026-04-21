mod admins;
mod auth;
mod config;
mod error;
mod field_definitions;
mod finance;
mod members;
mod services;
mod state;
mod storage;
mod ws;

use axum::{routing::{delete, get, post, put}, Router};
use axum::http::{Method, header};
use tower_http::cors::CorsLayer;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use crate::{config::Config, state::AppState};
use auth::handlers::{login, refresh, logout};
use ws::handler::ws_handler;
use finance::handlers::{
    list_accounts, create_account, get_account, update_account, delete_account,
    list_transactions, create_transaction, get_transaction, update_transaction, delete_transaction,
    upload_receipt, download_receipt, start_reconciliation, confirm_reconciliation,
    assign_finance_role, remove_finance_role, list_admin_finance_roles,
    list_categories, create_category, delete_category, list_all_transactions,
};
use std::sync::Arc;
use storage::LocalFileStorage;
use services::reconciliation::ManualReconciliationService;
use std::path::PathBuf;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations").run(&db).await.expect("migration failed");

    let (ws_tx, _) = broadcast::channel(100);

    // Initialize file storage with receipts directory
    let storage_path = PathBuf::from("./data");
    let file_storage = Arc::new(LocalFileStorage::new(storage_path));

    // Initialize reconciliation service
    let reconciliation_service = Arc::new(ManualReconciliationService);

    let state = AppState {
        db,
        config: config.clone(),
        ws_tx,
        file_storage,
        reconciliation_service,
    };

    let cors = CorsLayer::new()
        .allow_origin(state.config.frontend_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/api/v1/admins", get(admins::handlers::list_admins).post(admins::handlers::create_admin))
        .route("/api/v1/admins/:id", delete(admins::handlers::delete_admin))
        .route("/api/v1/admins/:id/password", put(admins::handlers::change_password))
        .route("/api/v1/members", get(members::handlers::list_members).post(members::handlers::create_member))
        .route("/api/v1/members/export", get(members::handlers::export_members))
        .route("/api/v1/members/:id", get(members::handlers::get_member).put(members::handlers::update_member).delete(members::handlers::delete_member))
        .route("/api/v1/field-definitions", get(field_definitions::handlers::list_fields).post(field_definitions::handlers::create_field))
        .route("/api/v1/field-definitions/:id", put(field_definitions::handlers::update_field).delete(field_definitions::handlers::delete_field))
        .route("/api/v1/field-definitions/:id/options", post(field_definitions::handlers::add_option))
        .route("/api/v1/field-definitions/:id/options/:option_id", put(field_definitions::handlers::update_option).delete(field_definitions::handlers::delete_option))
        .route("/api/v1/finance/accounts", get(list_accounts).post(create_account))
        .route("/api/v1/finance/accounts/:id", get(get_account).put(update_account).delete(delete_account))
        .route("/api/v1/finance/accounts/:id/transactions", get(list_transactions).post(create_transaction))
        .route("/api/v1/finance/accounts/:id/reconciliation", post(start_reconciliation))
        .route("/api/v1/finance/accounts/:id/reconciliation/:rec-id", put(confirm_reconciliation))
        .route("/api/v1/finance/transactions", get(list_all_transactions))
        .route("/api/v1/finance/transactions/:id", get(get_transaction).put(update_transaction).delete(delete_transaction))
        .route("/api/v1/finance/transactions/:id/receipt", post(upload_receipt))
        .route("/api/v1/finance/transactions/:id/receipt/:ref", get(download_receipt))
        .route("/api/v1/finance/admins/:id/roles", get(list_admin_finance_roles).post(assign_finance_role))
        .route("/api/v1/finance/admins/:id/roles/:role", delete(remove_finance_role))
        .route("/api/v1/finance/categories", get(list_categories).post(create_category))
        .route("/api/v1/finance/categories/:id", delete(delete_category))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
