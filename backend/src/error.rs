use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug)]
pub enum AppError {
    NotFound(String),
    Conflict { current_version: i32, submitted_version: i32 },
    Unauthorized,
    Forbidden,
    Validation(Vec<(String, String)>),
    Internal(anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                json!({"code": "NOT_FOUND", "message": msg}),
            ),
            AppError::Conflict { current_version, submitted_version } => (
                StatusCode::CONFLICT,
                json!({
                    "code": "CONFLICT",
                    "message": "Resource was modified by another user",
                    "details": {
                        "current_version": current_version,
                        "submitted_version": submitted_version
                    }
                }),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                json!({"code": "UNAUTHORIZED", "message": "Authentication required"}),
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                json!({"code": "FORBIDDEN", "message": "Insufficient permissions"}),
            ),
            AppError::Validation(errors) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({"code": "VALIDATION_ERROR", "message": "Validation failed", "details": errors}),
            ),
            AppError::Internal(e) => {
                tracing::error!("internal error: {e:?}");
                (StatusCode::INTERNAL_SERVER_ERROR, json!({"code": "INTERNAL_ERROR", "message": "Internal server error"}))
            }
        };
        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Internal(e)
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Internal(e.into())
    }
}
