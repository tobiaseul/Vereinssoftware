use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{auth::middleware::AuthClaims, error::AppError, state::AppState};
use super::{model::{CreateRoleRequest, Role}, repository};

pub async fn list_roles(State(state): State<AppState>, _: AuthClaims) -> Result<Json<Vec<Role>>, AppError> {
    Ok(Json(repository::list_roles(&state.db).await?))
}

pub async fn create_role(
    State(state): State<AppState>, _: AuthClaims,
    Json(body): Json<CreateRoleRequest>,
) -> Result<Json<Role>, AppError> {
    Ok(Json(repository::create_role(&state.db, &body.name).await?))
}

pub async fn delete_role(
    State(state): State<AppState>, _: AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if !repository::delete_role(&state.db, id).await? {
        return Err(AppError::NotFound("Role not found".into()));
    }
    Ok(Json(serde_json::json!({"ok": true})))
}
