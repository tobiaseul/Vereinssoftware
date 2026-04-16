mod admins;
mod auth;
mod config;
mod error;
mod field_definitions;
mod members;
mod state;
mod ws;

use axum::{routing::{delete, get, post, put}, Router};
use axum::http::{Method, header};
use tower_http::cors::CorsLayer;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use crate::{config::Config, state::AppState};
use auth::handlers::{login, refresh, logout};
use ws::handler::ws_handler;

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
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
