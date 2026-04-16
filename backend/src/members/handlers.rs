// src/members/handlers.rs
use axum::{
    extract::{Path, Query, State},
    http::header,
    response::Response,
    Json,
};
use uuid::Uuid;
use crate::{auth::middleware::AuthClaims, error::AppError, state::AppState};
use super::{model::{CreateMemberRequest, MemberListQuery, UpdateMemberRequest}, repository};

pub async fn list_members(
    State(state): State<AppState>,
    _: AuthClaims,
    Query(query): Query<MemberListQuery>,
) -> Result<Json<Vec<super::model::Member>>, AppError> {
    Ok(Json(repository::list_members(&state.db, &query).await?))
}

pub async fn get_member(
    State(state): State<AppState>,
    _: AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<super::model::Member>, AppError> {
    repository::get_member(&state.db, id)
        .await?
        .map(Json)
        .ok_or_else(|| AppError::NotFound("Member not found".into()))
}

pub async fn create_member(
    State(state): State<AppState>,
    _: AuthClaims,
    Json(body): Json<CreateMemberRequest>,
) -> Result<Json<super::model::Member>, AppError> {
    if body.first_name.trim().is_empty() || body.last_name.trim().is_empty() {
        return Err(AppError::Validation(vec![
            ("first_name".into(), "required".into()),
        ]));
    }
    let member = repository::create_member(&state.db, &body).await?;
    let _ = state.ws_tx.send(format!(r#"{{"type":"member_created","id":"{}"}}"#, member.id));
    Ok(Json(member))
}

pub async fn update_member(
    State(state): State<AppState>,
    _: AuthClaims,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateMemberRequest>,
) -> Result<Json<super::model::Member>, AppError> {
    let member = repository::update_member(&state.db, id, &body).await?;
    let _ = state.ws_tx.send(format!(r#"{{"type":"member_updated","id":"{}"}}"#, member.id));
    Ok(Json(member))
}

pub async fn delete_member(
    State(state): State<AppState>,
    _: AuthClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let found = repository::soft_delete_member(&state.db, id).await?;
    if !found { return Err(AppError::NotFound("Member not found or already left".into())); }
    let _ = state.ws_tx.send(format!(r#"{{"type":"member_deleted","id":"{}"}}"#, id));
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn export_members(
    State(state): State<AppState>,
    _: AuthClaims,
    Query(query): Query<MemberListQuery>,
) -> Result<Response, AppError> {
    let members = repository::list_members(&state.db, &query).await?;
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record(["id","first_name","last_name","email","phone","membership_type","joined_at","left_at"]).unwrap();
    for m in &members {
        wtr.write_record([
            m.id.to_string(),
            m.first_name.clone(),
            m.last_name.clone(),
            m.email.clone().unwrap_or_default(),
            m.phone.clone().unwrap_or_default(),
            m.membership_type.clone(),
            m.joined_at.to_string(),
            m.left_at.map(|d| d.to_string()).unwrap_or_default(),
        ]).unwrap();
    }
    let csv_bytes = wtr.into_inner().unwrap();

    Ok(Response::builder()
        .header(header::CONTENT_TYPE, "text/csv")
        .header(header::CONTENT_DISPOSITION, "attachment; filename=\"members.csv\"")
        .body(axum::body::Body::from(csv_bytes))
        .unwrap())
}
