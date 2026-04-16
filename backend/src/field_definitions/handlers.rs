use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{auth::middleware::AuthClaims, error::AppError, state::AppState};
use super::{model::{CreateFieldRequest, FieldDefinition}, repository};

pub async fn list_fields(State(s): State<AppState>, _: AuthClaims) -> Result<Json<Vec<FieldDefinition>>, AppError> {
    Ok(Json(repository::list_fields(&s.db).await?))
}

pub async fn create_field(State(s): State<AppState>, _: AuthClaims, Json(body): Json<CreateFieldRequest>) -> Result<Json<FieldDefinition>, AppError> {
    Ok(Json(repository::create_field(&s.db, &body).await?))
}

pub async fn delete_field(State(s): State<AppState>, _: AuthClaims, Path(id): Path<Uuid>) -> Result<Json<serde_json::Value>, AppError> {
    if !repository::delete_field(&s.db, id).await? { return Err(AppError::NotFound("Field not found".into())); }
    Ok(Json(serde_json::json!({"ok": true})))
}
