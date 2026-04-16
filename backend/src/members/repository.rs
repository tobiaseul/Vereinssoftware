// src/members/repository.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::{CreateMemberRequest, Member, MemberListQuery, UpdateMemberRequest};

pub async fn list_members(db: &PgPool, query: &MemberListQuery) -> Result<Vec<Member>, AppError> {
    let include_left = query.include_left.unwrap_or(false);
    let members = sqlx::query_as_unchecked!(
        Member,
        r#"SELECT id, version, first_name, last_name, email, phone, street, city,
                  postal_code, birth_date, membership_type,
                  joined_at, left_at, notes, custom_fields, created_at, updated_at
           FROM members
           WHERE ($1::bool OR left_at IS NULL)
             AND ($2::text IS NULL OR LOWER(first_name || ' ' || last_name) LIKE LOWER('%' || $2 || '%'))
             AND ($3::text IS NULL OR membership_type::text = $3)
           ORDER BY last_name, first_name"#,
        include_left, query.search, query.membership_type
    )
    .fetch_all(db)
    .await?;
    Ok(members)
}

pub async fn get_member(db: &PgPool, id: Uuid) -> Result<Option<Member>, AppError> {
    let member = sqlx::query_as_unchecked!(
        Member,
        r#"SELECT id, version, first_name, last_name, email, phone, street, city,
                  postal_code, birth_date, membership_type,
                  joined_at, left_at, notes, custom_fields, created_at, updated_at
           FROM members WHERE id = $1"#,
        id
    )
    .fetch_optional(db)
    .await?;
    Ok(member)
}

pub async fn create_member(db: &PgPool, req: &CreateMemberRequest) -> Result<Member, AppError> {
    let joined_at = req.joined_at.unwrap_or_else(|| chrono::Local::now().date_naive());
    let custom_fields = req.custom_fields.clone().unwrap_or(serde_json::json!({}));
    let member = sqlx::query_as_unchecked!(
        Member,
        r#"INSERT INTO members (first_name, last_name, email, phone, street, city,
               postal_code, birth_date, membership_type, joined_at, notes, custom_fields)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::membership_type,$10,$11,$12)
           RETURNING id, version, first_name, last_name, email, phone, street, city,
                     postal_code, birth_date, membership_type,
                     joined_at, left_at, notes, custom_fields, created_at, updated_at"#,
        req.first_name, req.last_name, req.email, req.phone, req.street, req.city,
        req.postal_code, req.birth_date, req.membership_type, joined_at, req.notes, custom_fields
    )
    .fetch_one(db)
    .await?;
    Ok(member)
}

pub async fn update_member(db: &PgPool, id: Uuid, req: &UpdateMemberRequest) -> Result<Member, AppError> {
    let custom_fields = req.custom_fields.clone().unwrap_or(serde_json::json!({}));
    let result = sqlx::query!(
        "SELECT version FROM members WHERE id = $1", id
    )
    .fetch_optional(db)
    .await?;

    let current = result.ok_or_else(|| AppError::NotFound("Member not found".into()))?;
    if current.version != req.version {
        return Err(AppError::Conflict {
            current_version: current.version,
            submitted_version: req.version,
        });
    }

    let member = sqlx::query_as_unchecked!(
        Member,
        r#"UPDATE members SET
               version = version + 1,
               first_name=$2, last_name=$3, email=$4, phone=$5,
               street=$6, city=$7, postal_code=$8, birth_date=$9,
               membership_type=$10::membership_type, joined_at=$11, left_at=$12,
               notes=$13, custom_fields=$14, updated_at=NOW()
           WHERE id=$1
           RETURNING id, version, first_name, last_name, email, phone, street, city,
                     postal_code, birth_date, membership_type,
                     joined_at, left_at, notes, custom_fields, created_at, updated_at"#,
        id, req.first_name, req.last_name, req.email, req.phone,
        req.street, req.city, req.postal_code, req.birth_date,
        req.membership_type, req.joined_at, req.left_at,
        req.notes, custom_fields
    )
    .fetch_one(db)
    .await?;
    Ok(member)
}

pub async fn soft_delete_member(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "UPDATE members SET left_at = CURRENT_DATE WHERE id = $1 AND left_at IS NULL",
        id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
