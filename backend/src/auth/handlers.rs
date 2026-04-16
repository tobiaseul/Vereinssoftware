// src/auth/handlers.rs
use axum::{
    extract::State,
    http::{header, HeaderMap},
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
use super::tokens::{create_access_token, hash_refresh_token,
    generate_refresh_token, verify_password, AdminRole};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub admin_id: Uuid,
    pub role: AdminRole,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), AppError> {
    let admin = sqlx::query!(
        r#"SELECT id, password_hash, role as "role: String" FROM admins WHERE username = $1"#,
        body.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if !verify_password(&body.password, &admin.password_hash) {
        return Err(AppError::Unauthorized);
    }

    let role = match admin.role.as_str() {
        "SuperAdmin" => AdminRole::SuperAdmin,
        _ => AdminRole::Admin,
    };

    let access_token = create_access_token(
        admin.id,
        role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_expiry_seconds,
    );

    let refresh_token = generate_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::days(state.config.refresh_token_expiry_days);

    sqlx::query!(
        "INSERT INTO refresh_tokens (admin_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        admin.id, token_hash, expires_at
    )
    .execute(&state.db)
    .await?;

    let cookie = format!(
        "refresh_token={refresh_token}; HttpOnly; SameSite=Strict; Path=/auth/refresh; Max-Age={}",
        state.config.refresh_token_expiry_days * 86400
    );
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, Json(LoginResponse { access_token, admin_id: admin.id, role })))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LoginResponse>, AppError> {
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find(|p| p.trim().starts_with("refresh_token=")))
        .and_then(|p| p.trim().strip_prefix("refresh_token="))
        .ok_or(AppError::Unauthorized)?
        .to_owned();

    let token_hash = hash_refresh_token(&cookie);

    let row = sqlx::query!(
        r#"SELECT rt.admin_id, rt.expires_at, a.role as "role: String"
           FROM refresh_tokens rt
           JOIN admins a ON a.id = rt.admin_id
           WHERE rt.token_hash = $1"#,
        token_hash
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if row.expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    let role = match row.role.as_str() {
        "SuperAdmin" => AdminRole::SuperAdmin,
        _ => AdminRole::Admin,
    };

    let access_token = create_access_token(
        row.admin_id,
        role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_expiry_seconds,
    );

    Ok(Json(LoginResponse { access_token, admin_id: row.admin_id, role }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(cookie) = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find(|p| p.trim().starts_with("refresh_token=")))
        .and_then(|p| p.trim().strip_prefix("refresh_token="))
    {
        let hash = hash_refresh_token(cookie);
        sqlx::query!("DELETE FROM refresh_tokens WHERE token_hash = $1", hash)
            .execute(&state.db)
            .await?;
    }
    Ok(Json(serde_json::json!({"ok": true})))
}
