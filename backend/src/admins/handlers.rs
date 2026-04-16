// src/admins/handlers.rs
use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{auth::{middleware::SuperAdminClaims, tokens::hash_password}, error::AppError, state::AppState};
use super::{model::{Admin, ChangePasswordRequest, CreateAdminRequest}, repository};

pub async fn list_admins(
    State(state): State<AppState>,
    _: SuperAdminClaims,
) -> Result<Json<Vec<Admin>>, AppError> {
    Ok(Json(repository::list_admins(&state.db).await?))
}

pub async fn create_admin(
    State(state): State<AppState>,
    _: SuperAdminClaims,
    Json(body): Json<CreateAdminRequest>,
) -> Result<Json<Admin>, AppError> {
    let role = body.role.unwrap_or_else(|| "Admin".into());
    if role != "Admin" && role != "SuperAdmin" {
        return Err(AppError::Validation(vec![("role".into(), "must be Admin or SuperAdmin".into())]));
    }
    let hash = hash_password(&body.password);
    Ok(Json(repository::create_admin(&state.db, &body.username, &hash, &role).await?))
}

pub async fn delete_admin(
    State(state): State<AppState>,
    _: SuperAdminClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let found = repository::delete_admin(&state.db, id).await?;
    if !found { return Err(AppError::NotFound("Admin not found".into())); }
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn change_password(
    State(state): State<AppState>,
    _: SuperAdminClaims,
    Path(id): Path<Uuid>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let hash = hash_password(&body.password);
    let found = repository::update_password(&state.db, id, &hash).await?;
    if !found { return Err(AppError::NotFound("Admin not found".into())); }
    Ok(Json(serde_json::json!({"ok": true})))
}
