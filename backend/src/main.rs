mod auth;
mod config;
mod error;
mod state;

use axum::{routing::{get, post}, Router};
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
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
