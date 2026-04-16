mod admins;
mod auth;
mod config;
mod error;
mod members;
mod state;

use axum::{routing::{delete, get, post, put}, Router};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use crate::{config::Config, state::AppState};
use auth::handlers::{login, refresh, logout};

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
    let state = AppState { db, config: config.clone(), ws_tx };

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/api/v1/admins", get(admins::handlers::list_admins).post(admins::handlers::create_admin))
        .route("/api/v1/admins/:id", delete(admins::handlers::delete_admin))
        .route("/api/v1/admins/:id/password", put(admins::handlers::change_password))
        .route("/api/v1/members", get(members::handlers::list_members).post(members::handlers::create_member))
        .route("/api/v1/members/export", get(members::handlers::export_members))
        .route("/api/v1/members/:id", get(members::handlers::get_member).put(members::handlers::update_member).delete(members::handlers::delete_member))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
