// src/finance/finance_auth.rs
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Check if an admin has a specific finance role
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `admin_id` - ID of the admin to check
/// * `required_role` - Name of the finance role to verify
///
/// # Returns
/// * `Ok(true)` if admin has the specified role
/// * `Ok(false)` if admin does not have the role
/// * `Err(sqlx::Error)` on database error
pub async fn has_finance_role(
    pool: &PgPool,
    admin_id: Uuid,
    required_role: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        "SELECT 1
         FROM admin_finance_roles afr
         INNER JOIN finance_roles fr ON afr.finance_role_id = fr.id
         WHERE afr.admin_id = $1 AND fr.name = $2
         LIMIT 1"
    )
    .bind(admin_id)
    .bind(required_role)
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}

/// Require Treasurer role for an admin
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `admin_id` - ID of the admin to check
///
/// # Returns
/// * `Ok(())` if admin has Treasurer role
/// * `Err(Response)` with 403 Forbidden response if not authorized
pub async fn require_treasurer(pool: &PgPool, admin_id: Uuid) -> Result<(), Response> {
    match has_finance_role(pool, admin_id, "Treasurer").await {
        Ok(true) => Ok(()),
        Ok(false) => Err(create_forbidden_response("Treasurer role required")),
        Err(_) => Err(create_forbidden_response("Unable to verify permissions")),
    }
}

/// Require Finance Officer or Treasurer role for an admin
///
/// Finance Officer and Treasurer are at the same level for this check.
/// The role hierarchy is: Treasurer > Finance Officer > Admin (view-only)
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `admin_id` - ID of the admin to check
///
/// # Returns
/// * `Ok(())` if admin has Finance Officer or Treasurer role
/// * `Err(Response)` with 403 Forbidden response if neither role present
pub async fn require_finance_officer(pool: &PgPool, admin_id: Uuid) -> Result<(), Response> {
    match has_finance_role(pool, admin_id, "Finance Officer").await {
        Ok(true) => return Ok(()),
        Err(_) => return Err(create_forbidden_response("Unable to verify permissions")),
        Ok(false) => {}
    }

    match has_finance_role(pool, admin_id, "Treasurer").await {
        Ok(true) => Ok(()),
        Ok(false) => Err(create_forbidden_response(
            "Finance Officer or Treasurer role required",
        )),
        Err(_) => Err(create_forbidden_response("Unable to verify permissions")),
    }
}

/// Create a standardized 403 Forbidden response
fn create_forbidden_response(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        axum::Json(json!({
            "code": "FORBIDDEN",
            "message": message
        })),
    )
        .into_response()
}
