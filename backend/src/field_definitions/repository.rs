use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::{CreateFieldRequest, FieldDefinition};

pub async fn list_fields(db: &PgPool) -> Result<Vec<FieldDefinition>, AppError> {
    Ok(sqlx::query_as!(
        FieldDefinition,
        r#"SELECT id, name, field_type as "field_type: String", required, display_order, created_at
           FROM field_definitions ORDER BY display_order, name"#
    ).fetch_all(db).await?)
}

pub async fn create_field(db: &PgPool, req: &CreateFieldRequest) -> Result<FieldDefinition, AppError> {
    let valid_types = ["text", "number", "date", "boolean"];
    if !valid_types.contains(&req.field_type.as_str()) {
        return Err(AppError::Validation(vec![
            ("field_type".into(), "must be text, number, date, or boolean".into())
        ]));
    }
    Ok(sqlx::query_as_unchecked!(
        FieldDefinition,
        r#"INSERT INTO field_definitions (name, field_type, required, display_order)
           VALUES ($1, $2::field_type, $3, $4)
           RETURNING id, name, field_type as "field_type: String", required, display_order, created_at"#,
        req.name, req.field_type,
        req.required.unwrap_or(false),
        req.display_order.unwrap_or(0)
    ).fetch_one(db).await?)
}

pub async fn delete_field(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query!("DELETE FROM field_definitions WHERE id = $1", id)
        .execute(db).await?.rows_affected() > 0)
}
