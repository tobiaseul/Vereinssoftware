use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{auth::middleware::AuthClaims, error::AppError, state::AppState};
use super::{
    model::{CreateFieldRequest, CreateOptionRequest, FieldDefinition, FieldOption,
            UpdateFieldRequest, UpdateOptionRequest},
    repository,
};

pub async fn list_fields(State(s): State<AppState>, _: AuthClaims) -> Result<Json<Vec<FieldDefinition>>, AppError> {
    Ok(Json(repository::list_fields(&s.db).await?))
}

pub async fn create_field(State(s): State<AppState>, _: AuthClaims, Json(body): Json<CreateFieldRequest>) -> Result<Json<FieldDefinition>, AppError> {
    Ok(Json(repository::create_field(&s.db, &body).await?))
}

pub async fn update_field(
    State(s): State<AppState>, _: AuthClaims,
    Path(id): Path<Uuid>, Json(body): Json<UpdateFieldRequest>,
) -> Result<Json<FieldDefinition>, AppError> {
    let field = repository::update_field(&s.db, id, &body).await?
        .ok_or_else(|| AppError::NotFound("Field not found".into()))?;
    Ok(Json(field))
}

pub async fn delete_field(State(s): State<AppState>, _: AuthClaims, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    if !repository::delete_field(&s.db, id).await? {
        return Err(AppError::NotFound("Field not found".into()));
    }
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn add_option(
    State(s): State<AppState>, _: AuthClaims,
    Path(field_id): Path<Uuid>, Json(body): Json<CreateOptionRequest>,
) -> Result<Json<FieldOption>, AppError> {
    Ok(Json(repository::add_option(&s.db, field_id, &body).await?))
}

pub async fn update_option(
    State(s): State<AppState>, _: AuthClaims,
    Path((field_id, option_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<UpdateOptionRequest>,
) -> Result<Json<FieldOption>, AppError> {
    repository::update_option(&s.db, field_id, option_id, &body).await?
        .ok_or_else(|| AppError::NotFound("Option not found".into()))
        .map(Json)
}

pub async fn delete_option(
    State(s): State<AppState>, _: AuthClaims,
    Path((field_id, option_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !repository::delete_option(&s.db, field_id, option_id).await? {
        return Err(AppError::NotFound("Option not found".into()));
    }
    Ok(Json(serde_json::json!({"ok": true})))
}
