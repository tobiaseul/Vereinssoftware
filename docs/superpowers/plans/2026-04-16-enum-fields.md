# Enum Custom Fields Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an `enum` (dropdown) field type with per-field option management, inline editing of field definitions, and removal of the standalone roles system.

**Architecture:** New DB migration adds `enum` to `field_type` and creates `field_definition_options`. Backend gains five new endpoints (update field, CRUD options). Frontend FieldsPage gains inline edit + expandable options panel. MemberForm renders enum fields as `<select>`. Roles are dropped from DB, API, and UI.

**Tech Stack:** Rust 1.77+, Axum 0.7, sqlx 0.7, PostgreSQL 15+, React 18, TypeScript 5, TanStack Query v5, Tailwind CSS v4

---

## File Structure

```
backend/
  migrations/
    002_enum_fields.sql          # new: alter type, new table, drop roles
  src/
    field_definitions/
      model.rs                   # modify: split into FieldDefinitionRow + FieldDefinition(+options), add FieldOption, new request types
      repository.rs              # modify: update list_fields, create_field, add update_field, get_options, add_option, update_option, delete_option
      handlers.rs                # modify: update existing, add update_field, add_option, update_option, delete_option
    main.rs                      # modify: new routes, remove roles routes and mod
    roles/                       # delete entire module

frontend/
  src/
    types/index.ts               # modify: add FieldOption, update FieldDefinition + FieldType
    api/fieldDefinitions.ts      # modify: add updateFieldDefinition, addFieldOption, updateFieldOption, deleteFieldOption
    pages/FieldsPage.tsx         # rewrite: inline edit, expandable options panel
    pages/RolesPage.tsx          # delete
    components/MemberForm.tsx    # modify: add enum case
    App.tsx                      # modify: remove roles route + nav link
```

---

### Task 1: Database migration

**Files:**
- Create: `backend/migrations/002_enum_fields.sql`

- [ ] **Step 1: Create the migration file**

```sql
-- backend/migrations/002_enum_fields.sql

-- Extend field_type enum with 'enum' value
ALTER TYPE field_type ADD VALUE IF NOT EXISTS 'enum';

-- Options for enum fields (CASCADE delete when field deleted)
CREATE TABLE field_definition_options (
    id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    field_definition_id  UUID NOT NULL REFERENCES field_definitions(id) ON DELETE CASCADE,
    value                TEXT NOT NULL,
    display_order        INTEGER NOT NULL DEFAULT 0,
    UNIQUE (field_definition_id, value)
);

-- Remove roles (replaced by enum custom fields)
DROP TABLE IF EXISTS member_roles;
DROP TABLE IF EXISTS roles;
```

- [ ] **Step 2: Run the migration**

```bash
cd /path/to/project
source .env
sqlx migrate run --database-url "$DATABASE_URL"
```

Expected output:
```
Applied 002_enum_fields.sql
```

- [ ] **Step 3: Verify schema**

```bash
psql "$DATABASE_URL" -c "\d field_definition_options"
psql "$DATABASE_URL" -c "\dT field_type"
```

Expected: `field_definition_options` table exists; `field_type` enum includes `enum`.

- [ ] **Step 4: Commit**

```bash
git add backend/migrations/002_enum_fields.sql
git commit -m "feat: add enum field type, field_definition_options table, drop roles"
```

---

### Task 2: Backend — model + repository

**Files:**
- Modify: `backend/src/field_definitions/model.rs`
- Modify: `backend/src/field_definitions/repository.rs`

- [ ] **Step 1: Write failing test**

Add to the bottom of `backend/src/field_definitions/repository.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::field_definitions::model::{CreateFieldRequest, CreateOptionRequest, UpdateFieldRequest, UpdateOptionRequest};

    #[sqlx::test(migrations = "../migrations")]
    async fn test_create_enum_field(pool: PgPool) {
        let req = CreateFieldRequest {
            name: "Status".into(), field_type: "enum".into(),
            required: Some(false), display_order: None,
        };
        let field = create_field(&pool, &req).await.unwrap();
        assert_eq!(field.field_type, "enum");
        assert_eq!(field.options.len(), 0);
    }

    #[sqlx::test(migrations = "../migrations")]
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

    #[sqlx::test(migrations = "../migrations")]
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

    #[sqlx::test(migrations = "../migrations")]
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
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cd backend && cargo test field_definitions 2>&1 | tail -20
```

Expected: compile errors (functions don't exist yet).

- [ ] **Step 3: Rewrite model.rs**

```rust
// backend/src/field_definitions/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// DB row — used internally for sqlx queries
#[derive(Debug, sqlx::FromRow)]
pub struct FieldDefinitionRow {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

/// API response — includes nested options
#[derive(Debug, Serialize)]
pub struct FieldDefinition {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub options: Vec<FieldOption>,
}

impl FieldDefinition {
    pub fn from_row(row: FieldDefinitionRow, options: Vec<FieldOption>) -> Self {
        Self {
            id: row.id,
            name: row.name,
            field_type: row.field_type,
            required: row.required,
            display_order: row.display_order,
            created_at: row.created_at,
            options,
        }
    }
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FieldOption {
    pub id: Uuid,
    pub field_definition_id: Uuid,
    pub value: String,
    pub display_order: i32,
}

#[derive(Deserialize)]
pub struct CreateFieldRequest {
    pub name: String,
    pub field_type: String,
    pub required: Option<bool>,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateFieldRequest {
    pub name: Option<String>,
    pub required: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateOptionRequest {
    pub value: String,
    pub display_order: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateOptionRequest {
    pub value: Option<String>,
    pub display_order: Option<i32>,
}
```

- [ ] **Step 4: Rewrite repository.rs**

```rust
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

    #[sqlx::test(migrations = "../migrations")]
    async fn test_create_enum_field(pool: PgPool) {
        let req = CreateFieldRequest {
            name: "Status".into(), field_type: "enum".into(),
            required: Some(false), display_order: None,
        };
        let field = create_field(&pool, &req).await.unwrap();
        assert_eq!(field.field_type, "enum");
        assert_eq!(field.options.len(), 0);
    }

    #[sqlx::test(migrations = "../migrations")]
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

    #[sqlx::test(migrations = "../migrations")]
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

    #[sqlx::test(migrations = "../migrations")]
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
```

- [ ] **Step 5: Run tests**

```bash
cd backend && cargo test field_definitions 2>&1 | tail -20
```

Expected: 4 tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/field_definitions/model.rs backend/src/field_definitions/repository.rs
git commit -m "feat: field_definitions model and repository with enum support and option CRUD"
```

---

### Task 3: Backend — handlers + routes

**Files:**
- Modify: `backend/src/field_definitions/handlers.rs`
- Modify: `backend/src/main.rs`
- Delete: `backend/src/roles/` (entire directory)

- [ ] **Step 1: Rewrite handlers.rs**

```rust
// backend/src/field_definitions/handlers.rs
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
    let row = repository::update_field(&s.db, id, &body).await?
        .ok_or_else(|| AppError::NotFound("Field not found".into()))?;
    let options = repository::get_options(&s.db, id).await?;
    Ok(Json(FieldDefinition::from_row(row, options)))
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
```

- [ ] **Step 2: Update main.rs**

Replace the full content of `backend/src/main.rs`:

```rust
mod admins;
mod auth;
mod config;
mod error;
mod field_definitions;
mod members;
mod state;
mod ws;

use axum::{routing::{delete, get, post, put}, Router};
use axum::http::{Method, header};
use tower_http::cors::CorsLayer;
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use crate::{config::Config, state::AppState};
use auth::handlers::{login, refresh, logout};
use ws::handler::ws_handler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    sqlx::migrate!("./migrations").run(&db).await.expect("migration failed");

    let (ws_tx, _) = broadcast::channel(100);
    let state = AppState { db, config: config.clone(), ws_tx };

    let cors = CorsLayer::new()
        .allow_origin(state.config.frontend_url.parse::<axum::http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true);

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ws", get(ws_handler))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/logout", post(logout))
        .route("/api/v1/admins", get(admins::handlers::list_admins).post(admins::handlers::create_admin))
        .route("/api/v1/admins/:id", delete(admins::handlers::delete_admin))
        .route("/api/v1/admins/:id/password", put(admins::handlers::change_password))
        .route("/api/v1/members", get(members::handlers::list_members).post(members::handlers::create_member))
        .route("/api/v1/members/export", get(members::handlers::export_members))
        .route("/api/v1/members/:id", get(members::handlers::get_member).put(members::handlers::update_member).delete(members::handlers::delete_member))
        .route("/api/v1/field-definitions", get(field_definitions::handlers::list_fields).post(field_definitions::handlers::create_field))
        .route("/api/v1/field-definitions/:id", put(field_definitions::handlers::update_field).delete(field_definitions::handlers::delete_field))
        .route("/api/v1/field-definitions/:id/options", post(field_definitions::handlers::add_option))
        .route("/api/v1/field-definitions/:id/options/:option_id", put(field_definitions::handlers::update_option).delete(field_definitions::handlers::delete_option))
        .layer(cors)
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 3: Delete the roles module**

```bash
rm -rf backend/src/roles
```

- [ ] **Step 4: Verify compilation**

```bash
cd backend && cargo check 2>&1 | tail -10
```

Expected: `Finished` with no errors (warnings are ok).

- [ ] **Step 5: Run all tests**

```bash
cd backend && cargo test 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/field_definitions/handlers.rs backend/src/main.rs
git rm -r backend/src/roles
git commit -m "feat: field definitions handlers with update + option CRUD; remove roles"
```

---

### Task 4: Frontend — types + API client

**Files:**
- Modify: `frontend/src/types/index.ts`
- Modify: `frontend/src/api/fieldDefinitions.ts`

- [ ] **Step 1: Update types/index.ts**

Replace the file content:

```ts
// frontend/src/types/index.ts
export type AdminRole = 'Admin' | 'SuperAdmin'
export type MembershipType = 'Aktiv' | 'Passiv' | 'Ehrenmitglied'
export type FieldType = 'text' | 'number' | 'date' | 'boolean' | 'enum'

export interface Admin {
  id: string
  username: string
  role: AdminRole
  created_at: string
}

export interface Member {
  id: string
  version: number
  first_name: string
  last_name: string
  email: string | null
  phone: string | null
  street: string | null
  city: string | null
  postal_code: string | null
  birth_date: string | null
  membership_type: MembershipType
  joined_at: string
  left_at: string | null
  notes: string | null
  custom_fields: Record<string, unknown>
  created_at: string
  updated_at: string
}

export interface FieldOption {
  id: string
  field_definition_id: string
  value: string
  display_order: number
}

export interface FieldDefinition {
  id: string
  name: string
  field_type: FieldType
  required: boolean
  display_order: number
  options: FieldOption[]
  created_at: string
}

export interface Role {
  id: string
  name: string
  created_at: string
}

export interface ConflictError {
  code: 'CONFLICT'
  message: string
  details: {
    current_version: number
    submitted_version: number
  }
}

export interface AuthState {
  access_token: string
  admin_id: string
  role: AdminRole
}
```

- [ ] **Step 2: Update api/fieldDefinitions.ts**

Replace the file content:

```ts
// frontend/src/api/fieldDefinitions.ts
import { client } from './client'
import type { FieldDefinition, FieldOption, FieldType } from '../types'

export const getFieldDefinitions = () =>
  client.get<FieldDefinition[]>('/api/v1/field-definitions').then(r => r.data)

export const createFieldDefinition = (data: { name: string; field_type: FieldType; required?: boolean; display_order?: number }) =>
  client.post<FieldDefinition>('/api/v1/field-definitions', data).then(r => r.data)

export const updateFieldDefinition = (id: string, data: { name?: string; required?: boolean }) =>
  client.put<FieldDefinition>(`/api/v1/field-definitions/${id}`, data).then(r => r.data)

export const deleteFieldDefinition = (id: string) =>
  client.delete(`/api/v1/field-definitions/${id}`).then(r => r.data)

export const addFieldOption = (fieldId: string, data: { value: string; display_order?: number }) =>
  client.post<FieldOption>(`/api/v1/field-definitions/${fieldId}/options`, data).then(r => r.data)

export const updateFieldOption = (fieldId: string, optionId: string, data: { value?: string; display_order?: number }) =>
  client.put<FieldOption>(`/api/v1/field-definitions/${fieldId}/options/${optionId}`, data).then(r => r.data)

export const deleteFieldOption = (fieldId: string, optionId: string) =>
  client.delete(`/api/v1/field-definitions/${fieldId}/options/${optionId}`).then(r => r.data)
```

- [ ] **Step 3: Run TypeScript check**

```bash
cd frontend && npx tsc --noEmit 2>&1 | head -30
```

Expected: errors only in FieldsPage (stub doesn't know about new types yet) — no errors in types or api files.

- [ ] **Step 4: Commit**

```bash
git add frontend/src/types/index.ts frontend/src/api/fieldDefinitions.ts
git commit -m "feat: add FieldOption type, enum to FieldType, option CRUD API functions"
```

---

### Task 5: Frontend — FieldsPage rewrite

**Files:**
- Modify: `frontend/src/pages/FieldsPage.tsx`

- [ ] **Step 1: Rewrite FieldsPage.tsx**

```tsx
// frontend/src/pages/FieldsPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import {
  getFieldDefinitions, createFieldDefinition, updateFieldDefinition, deleteFieldDefinition,
  addFieldOption, updateFieldOption, deleteFieldOption,
} from '../api/fieldDefinitions'
import type { FieldDefinition, FieldType } from '../types'

function OptionsList({ field }: { field: FieldDefinition }) {
  const qc = useQueryClient()
  const [newValue, setNewValue] = useState('')
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editValue, setEditValue] = useState('')
  const [error, setError] = useState<string | null>(null)

  const add = useMutation({
    mutationFn: () => addFieldOption(field.id, { value: newValue }),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setNewValue(''); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to add option'),
  })

  const update = useMutation({
    mutationFn: ({ optionId, value }: { optionId: string; value: string }) =>
      updateFieldOption(field.id, optionId, { value }),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setEditingId(null); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to update option'),
  })

  const remove = useMutation({
    mutationFn: (optionId: string) => deleteFieldOption(field.id, optionId),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to delete option'),
  })

  return (
    <div className="mt-2 ml-4 pl-4 border-l border-gray-200 space-y-1">
      {error && <p className="text-red-600 text-xs">{error}</p>}
      {field.options.map(opt => (
        <div key={opt.id} className="flex items-center gap-2">
          {editingId === opt.id ? (
            <>
              <input className="border rounded px-2 py-1 text-sm flex-1"
                value={editValue} onChange={e => setEditValue(e.target.value)} />
              <button onClick={() => update.mutate({ optionId: opt.id, value: editValue })}
                disabled={!editValue || update.isPending}
                className="text-green-600 text-sm hover:text-green-800 disabled:opacity-50">Save</button>
              <button onClick={() => setEditingId(null)}
                className="text-gray-500 text-sm hover:text-gray-700">Cancel</button>
            </>
          ) : (
            <>
              <span className="text-sm flex-1">{opt.value}</span>
              <button onClick={() => { setEditingId(opt.id); setEditValue(opt.value) }}
                className="text-blue-600 text-sm hover:text-blue-800">Edit</button>
              <button onClick={() => { if (confirm(`Remove option "${opt.value}"?`)) remove.mutate(opt.id) }}
                className="text-red-600 text-sm hover:text-red-800">Remove</button>
            </>
          )}
        </div>
      ))}
      <div className="flex gap-2 mt-2">
        <input className="border rounded px-2 py-1 text-sm flex-1" placeholder="New option..."
          value={newValue} onChange={e => setNewValue(e.target.value)}
          onKeyDown={e => { if (e.key === 'Enter' && newValue) add.mutate() }} />
        <button onClick={() => add.mutate()} disabled={!newValue || add.isPending}
          className="px-3 py-1 bg-blue-600 text-white text-sm rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>
    </div>
  )
}

export function FieldsPage() {
  const qc = useQueryClient()
  const [form, setForm] = useState({ name: '', field_type: 'text' as FieldType, required: false })
  const [error, setError] = useState<string | null>(null)
  const [deletingId, setDeletingId] = useState<string | null>(null)
  const [editingId, setEditingId] = useState<string | null>(null)
  const [editForm, setEditForm] = useState({ name: '', required: false })
  const [expandedId, setExpandedId] = useState<string | null>(null)

  const { data: fields = [] } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })

  const add = useMutation({
    mutationFn: () => createFieldDefinition(form),
    onSuccess: (newField) => {
      qc.invalidateQueries({ queryKey: ['field-definitions'] })
      setForm({ name: '', field_type: 'text', required: false })
      setError(null)
      if (newField.field_type === 'enum') setExpandedId(newField.id)
    },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to add field'),
  })

  const update = useMutation({
    mutationFn: ({ id, ...data }: { id: string; name: string; required: boolean }) =>
      updateFieldDefinition(id, data),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setEditingId(null); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to update field'),
  })

  const remove = useMutation({
    mutationFn: (id: string) => deleteFieldDefinition(id),
    onMutate: (id) => setDeletingId(id),
    onSettled: () => setDeletingId(null),
    onSuccess: () => { qc.invalidateQueries({ queryKey: ['field-definitions'] }); setError(null) },
    onError: (err) => setError(err instanceof Error ? err.message : 'Failed to delete field'),
  })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Custom Fields</h1>
      {error && <p className="text-red-600 text-sm mb-2">{error}</p>}

      <div className="flex gap-2 mb-4">
        <input className="border rounded px-3 py-2 flex-1" placeholder="Field name..."
          value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
        <select className="border rounded px-3 py-2" value={form.field_type}
          onChange={e => setForm(f => ({ ...f, field_type: e.target.value as FieldType }))}>
          <option value="text">Text</option>
          <option value="number">Number</option>
          <option value="date">Date</option>
          <option value="boolean">Boolean</option>
          <option value="enum">Dropdown</option>
        </select>
        <label className="flex items-center gap-1 text-sm">
          <input type="checkbox" checked={form.required}
            onChange={e => setForm(f => ({ ...f, required: e.target.checked }))} /> Required
        </label>
        <button onClick={() => add.mutate()} disabled={!form.name || add.isPending}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>

      <ul className="space-y-2">
        {fields.map(f => (
          <li key={f.id} className="border rounded px-3 py-2">
            {editingId === f.id ? (
              <div className="flex gap-2 items-center">
                <input className="border rounded px-2 py-1 flex-1" value={editForm.name}
                  onChange={e => setEditForm(ef => ({ ...ef, name: e.target.value }))} />
                <label className="flex items-center gap-1 text-sm">
                  <input type="checkbox" checked={editForm.required}
                    onChange={e => setEditForm(ef => ({ ...ef, required: e.target.checked }))} /> Required
                </label>
                <button onClick={() => update.mutate({ id: f.id, ...editForm })}
                  disabled={!editForm.name || update.isPending}
                  className="px-2 py-1 bg-green-600 text-white text-sm rounded hover:bg-green-700 disabled:opacity-50">Save</button>
                <button onClick={() => setEditingId(null)}
                  className="text-sm text-gray-500 hover:text-gray-700">Cancel</button>
              </div>
            ) : (
              <div className="flex justify-between items-center">
                <div className="flex items-center gap-2">
                  {f.field_type === 'enum' && (
                    <button onClick={() => setExpandedId(expandedId === f.id ? null : f.id)}
                      className="text-gray-400 hover:text-gray-600 text-xs w-4">
                      {expandedId === f.id ? '▼' : '▶'}
                    </button>
                  )}
                  <span>
                    {f.name}
                    <span className="text-gray-500 text-sm ml-1">
                      ({f.field_type === 'enum' ? `dropdown, ${f.options.length} options` : f.field_type}
                      {f.required ? ', required' : ''})
                    </span>
                  </span>
                </div>
                <div className="flex gap-2">
                  <button onClick={() => { setEditingId(f.id); setEditForm({ name: f.name, required: f.required }) }}
                    className="text-blue-600 hover:text-blue-800 text-sm">Edit</button>
                  <button disabled={deletingId === f.id}
                    onClick={() => { if (confirm(`Remove "${f.name}" and all its options?`)) remove.mutate(f.id) }}
                    className="text-red-600 hover:text-red-800 text-sm disabled:opacity-50">Remove</button>
                </div>
              </div>
            )}
            {expandedId === f.id && f.field_type === 'enum' && <OptionsList field={f} />}
          </li>
        ))}
      </ul>
    </div>
  )
}
```

- [ ] **Step 2: Run TypeScript check**

```bash
cd frontend && npx tsc --noEmit 2>&1 | head -20
```

Expected: no errors in FieldsPage.tsx.

- [ ] **Step 3: Commit**

```bash
git add frontend/src/pages/FieldsPage.tsx
git commit -m "feat: FieldsPage with inline edit, enum field type, and expandable options panel"
```

---

### Task 6: Frontend — MemberForm enum case + App.tsx cleanup

**Files:**
- Modify: `frontend/src/components/MemberForm.tsx`
- Modify: `frontend/src/App.tsx`
- Delete: `frontend/src/pages/RolesPage.tsx`

- [ ] **Step 1: Update MemberForm.tsx custom fields section**

Replace only the custom fields rendering block (lines 58–79). The full updated file:

```tsx
// frontend/src/components/MemberForm.tsx
import { useQuery } from '@tanstack/react-query'
import { getFieldDefinitions } from '../api/fieldDefinitions'
import type { Member, MembershipType } from '../types'

interface Props {
  value: Partial<Member>
  onChange: (updated: Partial<Member>) => void
  disabled?: boolean
}

export function MemberForm({ value, onChange, disabled }: Props) {
  const { data: fields = [] } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })

  const set = (key: keyof Member, val: unknown) => onChange({ ...value, [key]: val })
  const cf = value.custom_fields as Record<string, unknown> ?? {}
  const setCf = (name: string, val: unknown) => set('custom_fields', { ...cf, [name]: val })

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium mb-1">First Name *</label>
          <input required disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.first_name ?? ''} onChange={e => set('first_name', e.target.value)} />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Last Name *</label>
          <input required disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.last_name ?? ''} onChange={e => set('last_name', e.target.value)} />
        </div>
      </div>
      <div className="grid grid-cols-2 gap-4">
        <div>
          <label className="block text-sm font-medium mb-1">Email</label>
          <input type="email" disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.email ?? ''} onChange={e => set('email', e.target.value || null)} />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Phone</label>
          <input disabled={disabled} className="border rounded w-full px-3 py-2"
            value={value.phone ?? ''} onChange={e => set('phone', e.target.value || null)} />
        </div>
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Membership Type *</label>
        <select required disabled={disabled} className="border rounded px-3 py-2"
          value={value.membership_type ?? ''} onChange={e => set('membership_type', e.target.value as MembershipType)}>
          <option value="">Select...</option>
          <option value="Aktiv">Aktiv</option>
          <option value="Passiv">Passiv</option>
          <option value="Ehrenmitglied">Ehrenmitglied</option>
        </select>
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Notes</label>
        <textarea disabled={disabled} className="border rounded w-full px-3 py-2" rows={3}
          value={value.notes ?? ''} onChange={e => set('notes', e.target.value || null)} />
      </div>

      {fields.length > 0 && (
        <div className="border-t pt-4">
          <h3 className="font-medium mb-3">Custom Fields</h3>
          <div className="space-y-3">
            {fields.map(f => (
              <div key={f.id}>
                <label className="block text-sm font-medium mb-1">{f.name}{f.required && ' *'}</label>
                {f.field_type === 'enum' ? (
                  <select disabled={disabled} className="border rounded px-3 py-2 w-full"
                    value={cf[f.name] as string ?? ''}
                    onChange={e => setCf(f.name, e.target.value || null)}>
                    <option value="">Select...</option>
                    {f.options.map(opt => (
                      <option key={opt.id} value={opt.value}>{opt.value}</option>
                    ))}
                  </select>
                ) : f.field_type === 'boolean' ? (
                  <input type="checkbox" disabled={disabled}
                    checked={!!cf[f.name]}
                    onChange={e => setCf(f.name, e.target.checked)} />
                ) : (
                  <input
                    type={f.field_type === 'number' ? 'number' : f.field_type === 'date' ? 'date' : 'text'}
                    disabled={disabled} className="border rounded w-full px-3 py-2"
                    value={cf[f.name] as string ?? ''}
                    onChange={e => setCf(f.name, e.target.value || null)} />
                )}
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Update App.tsx — remove roles**

```tsx
// frontend/src/App.tsx
import { BrowserRouter, Routes, Route, Navigate, Link } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AuthProvider, useAuth } from './hooks/useAuth'
import { ProtectedRoute } from './components/ProtectedRoute'
import { SuperAdminRoute } from './components/SuperAdminRoute'
import { LoginPage } from './pages/LoginPage'
import { MembersPage } from './pages/MembersPage'
import { MemberDetailPage } from './pages/MemberDetailPage'
import { MemberNewPage } from './pages/MemberNewPage'
import { FieldsPage } from './pages/FieldsPage'
import { AdminsPage } from './pages/AdminsPage'

const queryClient = new QueryClient()

function Nav() {
  const { auth, logout } = useAuth()
  if (!auth) return null
  return (
    <nav className="bg-gray-800 text-white px-6 py-3 flex gap-6 items-center">
      <Link to="/members" className="hover:text-blue-300">Members</Link>
      <Link to="/settings/fields" className="hover:text-blue-300">Fields</Link>
      {auth.role === 'SuperAdmin' && <Link to="/settings/admins" className="hover:text-blue-300">Admins</Link>}
      <button onClick={logout} className="ml-auto text-sm hover:text-red-300">Logout</button>
    </nav>
  )
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <BrowserRouter>
          <Nav />
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/members" element={<ProtectedRoute><MembersPage /></ProtectedRoute>} />
            <Route path="/members/new" element={<ProtectedRoute><MemberNewPage /></ProtectedRoute>} />
            <Route path="/members/:id" element={<ProtectedRoute><MemberDetailPage /></ProtectedRoute>} />
            <Route path="/settings/fields" element={<ProtectedRoute><FieldsPage /></ProtectedRoute>} />
            <Route path="/settings/admins" element={<SuperAdminRoute><AdminsPage /></SuperAdminRoute>} />
            <Route path="*" element={<Navigate to="/members" replace />} />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  )
}
```

- [ ] **Step 3: Delete RolesPage.tsx**

```bash
rm frontend/src/pages/RolesPage.tsx
```

- [ ] **Step 4: Run TypeScript check**

```bash
cd frontend && npx tsc --noEmit 2>&1 | head -20
```

Expected: no errors.

- [ ] **Step 5: Run all frontend tests**

```bash
cd frontend && npx vitest run 2>&1 | tail -10
```

Expected: all tests pass.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/components/MemberForm.tsx frontend/src/App.tsx
git rm frontend/src/pages/RolesPage.tsx
git commit -m "feat: enum dropdown in MemberForm, remove Roles from nav and router"
```
