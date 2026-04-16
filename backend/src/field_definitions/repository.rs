// backend/src/field_definitions/repository.rs
use std::collections::HashMap;
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::{
    CreateFieldRequest, CreateOptionRequest, FieldDefinition, FieldDefinitionRow,
    FieldOption, UpdateFieldRequest, UpdateOptionRequest,
};

pub async fn list_fields(db: &PgPool) -> Result<Vec<FieldDefinition>, AppError> {
    let rows = sqlx::query_as!(
        FieldDefinitionRow,
        r#"SELECT id, name, field_type as "field_type: String", required, display_order, created_at
           FROM field_definitions ORDER BY display_order, name"#
    ).fetch_all(db).await?;

    let options = sqlx::query_as!(
        FieldOption,
        "SELECT id, field_definition_id, value, display_order
         FROM field_definition_options ORDER BY display_order, value"
    ).fetch_all(db).await?;

    let mut by_field: HashMap<Uuid, Vec<FieldOption>> = HashMap::new();
    for opt in options {
        by_field.entry(opt.field_definition_id).or_default().push(opt);
    }

    Ok(rows.into_iter().map(|row| {
        let opts = by_field.remove(&row.id).unwrap_or_default();
        FieldDefinition::from_row(row, opts)
    }).collect())
}

pub async fn create_field(db: &PgPool, req: &CreateFieldRequest) -> Result<FieldDefinition, AppError> {
    let valid = ["text", "number", "date", "boolean", "enum"];
    if !valid.contains(&req.field_type.as_str()) {
        return Err(AppError::Validation(vec![
            ("field_type".into(), "must be text, number, date, boolean, or enum".into())
        ]));
    }
    let row = sqlx::query_as_unchecked!(
        FieldDefinitionRow,
        r#"INSERT INTO field_definitions (name, field_type, required, display_order)
           VALUES ($1, $2::field_type, $3, $4)
           RETURNING id, name, field_type as "field_type: String", required, display_order, created_at"#,
        req.name, req.field_type,
        req.required.unwrap_or(false),
        req.display_order.unwrap_or(0)
    ).fetch_one(db).await?;
    Ok(FieldDefinition::from_row(row, vec![]))
}

pub async fn update_field(db: &PgPool, id: Uuid, req: &UpdateFieldRequest) -> Result<Option<FieldDefinitionRow>, AppError> {
    Ok(sqlx::query_as_unchecked!(
        FieldDefinitionRow,
        r#"UPDATE field_definitions
           SET name = COALESCE($2, name), required = COALESCE($3, required)
           WHERE id = $1
           RETURNING id, name, field_type as "field_type: String", required, display_order, created_at"#,
        id, req.name, req.required
    ).fetch_optional(db).await?)
}

pub async fn delete_field(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query!("DELETE FROM field_definitions WHERE id = $1", id)
        .execute(db).await?.rows_affected() > 0)
}

pub async fn get_options(db: &PgPool, field_id: Uuid) -> Result<Vec<FieldOption>, AppError> {
    Ok(sqlx::query_as!(
        FieldOption,
        "SELECT id, field_definition_id, value, display_order
         FROM field_definition_options WHERE field_definition_id = $1 ORDER BY display_order, value",
        field_id
    ).fetch_all(db).await?)
}

pub async fn add_option(db: &PgPool, field_id: Uuid, req: &CreateOptionRequest) -> Result<FieldOption, AppError> {
    Ok(sqlx::query_as!(
        FieldOption,
        "INSERT INTO field_definition_options (field_definition_id, value, display_order)
         VALUES ($1, $2, $3)
         RETURNING id, field_definition_id, value, display_order",
        field_id, req.value, req.display_order.unwrap_or(0)
    ).fetch_one(db).await?)
}

pub async fn update_option(db: &PgPool, field_id: Uuid, option_id: Uuid, req: &UpdateOptionRequest) -> Result<Option<FieldOption>, AppError> {
    Ok(sqlx::query_as!(
        FieldOption,
        "UPDATE field_definition_options
         SET value = COALESCE($3, value), display_order = COALESCE($4, display_order)
         WHERE id = $1 AND field_definition_id = $2
         RETURNING id, field_definition_id, value, display_order",
        option_id, field_id, req.value, req.display_order
    ).fetch_optional(db).await?)
}

pub async fn delete_option(db: &PgPool, field_id: Uuid, option_id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query!(
        "DELETE FROM field_definition_options WHERE id = $1 AND field_definition_id = $2",
        option_id, field_id
    ).execute(db).await?.rows_affected() > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_definitions::model::{CreateFieldRequest, CreateOptionRequest, UpdateFieldRequest, UpdateOptionRequest};

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn test_create_enum_field(pool: PgPool) {
        let req = CreateFieldRequest {
            name: "Status".into(), field_type: "enum".into(),
            required: Some(false), display_order: None,
        };
        let field = create_field(&pool, &req).await.unwrap();
        assert_eq!(field.field_type, "enum");
        assert_eq!(field.options.len(), 0);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn test_add_and_list_option(pool: PgPool) {
        let field = create_field(&pool, &CreateFieldRequest {
            name: "Funktion".into(), field_type: "enum".into(),
            required: None, display_order: None,
        }).await.unwrap();

        let opt = add_option(&pool, field.id, &CreateOptionRequest {
            value: "Vorstand".into(), display_order: None,
        }).await.unwrap();
        assert_eq!(opt.value, "Vorstand");
        assert_eq!(opt.field_definition_id, field.id);

        let fields = list_fields(&pool).await.unwrap();
        let found = fields.iter().find(|f| f.id == field.id).unwrap();
        assert_eq!(found.options.len(), 1);
        assert_eq!(found.options[0].value, "Vorstand");
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn test_update_field(pool: PgPool) {
        let field = create_field(&pool, &CreateFieldRequest {
            name: "Old".into(), field_type: "text".into(),
            required: None, display_order: None,
        }).await.unwrap();

        let updated = update_field(&pool, field.id, &UpdateFieldRequest {
            name: Some("New".into()), required: Some(true),
        }).await.unwrap().unwrap();
        assert_eq!(updated.name, "New");
        assert!(updated.required);
    }

    #[sqlx::test(migrator = "MIGRATOR")]
    async fn test_update_and_delete_option(pool: PgPool) {
        let field = create_field(&pool, &CreateFieldRequest {
            name: "F".into(), field_type: "enum".into(),
            required: None, display_order: None,
        }).await.unwrap();
        let opt = add_option(&pool, field.id, &CreateOptionRequest {
            value: "A".into(), display_order: None,
        }).await.unwrap();

        let updated = update_option(&pool, field.id, opt.id, &UpdateOptionRequest {
            value: Some("B".into()), display_order: None,
        }).await.unwrap().unwrap();
        assert_eq!(updated.value, "B");

        let deleted = delete_option(&pool, field.id, opt.id).await.unwrap();
        assert!(deleted);
    }
}
