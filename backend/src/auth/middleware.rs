// src/auth/middleware.rs
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::{error::AppError, state::AppState};
use super::tokens::{validate_access_token, AdminRole, Claims};

pub struct AuthClaims(pub Claims);
pub struct SuperAdminClaims(pub Claims);

#[async_trait]
impl FromRequestParts<AppState> for AuthClaims {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        validate_access_token(token, &state.config.jwt_secret)
            .map(AuthClaims)
            .map_err(|_| AppError::Unauthorized)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for SuperAdminClaims {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let AuthClaims(claims) = AuthClaims::from_request_parts(parts, state).await?;
        if claims.role != AdminRole::SuperAdmin {
            return Err(AppError::Forbidden);
        }
        Ok(SuperAdminClaims(claims))
    }
}
