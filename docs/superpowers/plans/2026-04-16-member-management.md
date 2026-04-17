# Member Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a multi-user admin tool for club member management with auth, dynamic fields, and optimistic concurrency.

**Architecture:** Rust/Axum REST API + WebSocket presence server backed by PostgreSQL. React/TypeScript SPA consuming the API. Optimistic concurrency via row `version` integers; conflicts return HTTP 409 with both versions for frontend resolution.

**Tech Stack:** Rust 1.77+, Axum 0.7, sqlx 0.7, PostgreSQL 15+, React 18, TypeScript 5, Vite, TanStack Query v5, Axios, Tailwind CSS, Vitest

---

## File Structure

```
/
├── .env.example
├── .gitignore
├── backend/
│   ├── Cargo.toml
│   ├── migrations/
│   │   ├── 001_initial.sql
│   │   └── 002_seed_superadmin.sql
│   └── src/
│       ├── main.rs                  # app entry, router assembly
│       ├── config.rs                # env-based config
│       ├── error.rs                 # AppError → HTTP response
│       ├── state.rs                 # AppState (db pool, config, ws broadcast)
│       ├── auth/
│       │   ├── mod.rs
│       │   ├── tokens.rs            # JWT + refresh token logic
│       │   ├── middleware.rs        # JWT extractor / SuperAdmin guard
│       │   └── handlers.rs          # login, refresh, logout
│       ├── admins/
│       │   ├── mod.rs
│       │   ├── model.rs
│       │   ├── repository.rs
│       │   └── handlers.rs
│       ├── members/
│       │   ├── mod.rs
│       │   ├── model.rs
│       │   ├── repository.rs
│       │   └── handlers.rs          # list, get, create, update, delete, export
│       ├── roles/
│       │   ├── mod.rs
│       │   ├── model.rs
│       │   ├── repository.rs
│       │   └── handlers.rs
│       ├── field_definitions/
│       │   ├── mod.rs
│       │   ├── model.rs
│       │   ├── repository.rs
│       │   └── handlers.rs
│       └── ws/
│           ├── mod.rs
│           └── handler.rs           # WebSocket presence
└── frontend/
    ├── package.json
    ├── vite.config.ts
    ├── tailwind.config.ts
    ├── index.html
    └── src/
        ├── main.tsx
        ├── App.tsx                  # router + QueryClientProvider
        ├── types/index.ts           # shared TS types
        ├── api/
        │   ├── client.ts            # axios instance + token refresh interceptor
        │   ├── auth.ts
        │   ├── members.ts
        │   ├── roles.ts
        │   ├── fieldDefinitions.ts
        │   └── admins.ts
        ├── hooks/
        │   ├── useAuth.ts
        │   └── usePresence.ts
        ├── components/
        │   ├── ProtectedRoute.tsx
        │   ├── SuperAdminRoute.tsx
        │   ├── MemberForm.tsx
        │   ├── ConflictDialog.tsx
        │   └── PresenceIndicator.tsx
        └── pages/
            ├── LoginPage.tsx
            ├── MembersPage.tsx
            ├── MemberDetailPage.tsx
            ├── MemberNewPage.tsx
            ├── RolesPage.tsx
            ├── FieldsPage.tsx
            └── AdminsPage.tsx
```

---

### Task 1: Workspace & environment setup

**Files:**
- Create: `.gitignore`
- Create: `.env.example`

- [ ] **Step 1: Create .gitignore**

```
/backend/target
/frontend/node_modules
/frontend/dist
.env
*.env.local
```

- [ ] **Step 2: Create .env.example**

```
DATABASE_URL=postgres://postgres:password@localhost:5432/vereinssoftware
JWT_SECRET=change-me-to-a-long-random-string
JWT_EXPIRY_SECONDS=900
REFRESH_TOKEN_EXPIRY_DAYS=7
BACKEND_PORT=3000
FRONTEND_URL=http://localhost:5173
```

- [ ] **Step 3: Copy to .env and fill in values**

```bash
cp .env.example .env
```

- [ ] **Step 4: Create the database**

```bash
createdb vereinssoftware
```

- [ ] **Step 5: Commit**

```bash
git init
git add .gitignore .env.example
git commit -m "chore: initial workspace setup"
```

---

### Task 2: Rust backend init

**Files:**
- Create: `backend/Cargo.toml`
- Create: `backend/src/main.rs`

- [ ] **Step 1: Initialize Rust project**

```bash
cd backend && cargo init --name vereinssoftware-backend
```

- [ ] **Step 2: Set Cargo.toml dependencies**

```toml
[package]
name = "vereinssoftware-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.7", features = ["ws", "macros"] }
tokio = { version = "1", features = ["full"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "json"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
jsonwebtoken = "9"
argon2 = "0.5"
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tower-http = { version = "0.5", features = ["cors"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
dotenvy = "0.15"
csv = "1"
rand = "0.8"

[dev-dependencies]
axum-test = "14"
```

- [ ] **Step 3: Write minimal main.rs health check**

```rust
// src/main.rs
use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new().route("/health", get(|| async { "ok" }));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 4: Verify it compiles and responds**

```bash
cd backend && cargo run &
curl http://localhost:3000/health
# Expected: ok
kill %1
```

- [ ] **Step 5: Commit**

```bash
git add backend/
git commit -m "chore: init Rust backend with health check"
```

---

### Task 3: Database migrations

**Files:**
- Create: `backend/migrations/001_initial.sql`

- [ ] **Step 1: Install sqlx-cli**

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

- [ ] **Step 2: Write migration 001**

```sql
-- backend/migrations/001_initial.sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TYPE admin_role AS ENUM ('Admin', 'SuperAdmin');
CREATE TYPE membership_type AS ENUM ('Aktiv', 'Passiv', 'Ehrenmitglied');
CREATE TYPE field_type AS ENUM ('text', 'number', 'date', 'boolean');

CREATE TABLE admins (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username    TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role        admin_role NOT NULL DEFAULT 'Admin',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE refresh_tokens (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    admin_id    UUID NOT NULL REFERENCES admins(id) ON DELETE CASCADE,
    token_hash  TEXT NOT NULL UNIQUE,
    expires_at  TIMESTAMPTZ NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE members (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version         INTEGER NOT NULL DEFAULT 1,
    first_name      TEXT NOT NULL,
    last_name       TEXT NOT NULL,
    email           TEXT,
    phone           TEXT,
    street          TEXT,
    city            TEXT,
    postal_code     TEXT,
    birth_date      DATE,
    membership_type membership_type NOT NULL,
    joined_at       DATE NOT NULL DEFAULT CURRENT_DATE,
    left_at         DATE,
    notes           TEXT,
    custom_fields   JSONB NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE roles (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name       TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE member_roles (
    member_id   UUID NOT NULL REFERENCES members(id) ON DELETE CASCADE,
    role_id     UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (member_id, role_id)
);

CREATE TABLE field_definitions (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          TEXT NOT NULL UNIQUE,
    field_type    field_type NOT NULL,
    required      BOOLEAN NOT NULL DEFAULT FALSE,
    display_order INTEGER NOT NULL DEFAULT 0,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

- [ ] **Step 3: Run migration**

```bash
cd backend && sqlx migrate run --database-url $DATABASE_URL
```

Expected: `Applied 1/migrations/001_initial.sql`

- [ ] **Step 4: Commit**

```bash
git add backend/migrations/
git commit -m "feat: database schema migration"
```

---

### Task 4: Config & error types

**Files:**
- Create: `backend/src/config.rs`
- Create: `backend/src/error.rs`
- Create: `backend/src/state.rs`

- [ ] **Step 1: Write config.rs**

```rust
// src/config.rs
#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiry_seconds: u64,
    pub refresh_token_expiry_days: i64,
    pub port: u16,
    pub frontend_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: std::env::var("DATABASE_URL").expect("DATABASE_URL required"),
            jwt_secret: std::env::var("JWT_SECRET").expect("JWT_SECRET required"),
            jwt_expiry_seconds: std::env::var("JWT_EXPIRY_SECONDS")
                .unwrap_or("900".into()).parse().unwrap(),
            refresh_token_expiry_days: std::env::var("REFRESH_TOKEN_EXPIRY_DAYS")
                .unwrap_or("7".into()).parse().unwrap(),
            port: std::env::var("BACKEND_PORT")
                .unwrap_or("3000".into()).parse().unwrap(),
            frontend_url: std::env::var("FRONTEND_URL")
                .unwrap_or("http://localhost:5173".into()),
        }
    }
}
```

- [ ] **Step 2: Write error.rs**

```rust
// src/error.rs
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

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError::Internal(e.into())
    }
}
```

- [ ] **Step 3: Add anyhow to Cargo.toml**

```toml
anyhow = "1"
```

- [ ] **Step 4: Write state.rs**

```rust
// src/state.rs
use sqlx::PgPool;
use tokio::sync::broadcast;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub ws_tx: broadcast::Sender<String>,
}
```

- [ ] **Step 5: Update main.rs to connect DB and build state**

```rust
// src/main.rs
mod config;
mod error;
mod state;

use axum::{routing::get, Router};
use sqlx::postgres::PgPoolOptions;
use tokio::sync::broadcast;
use crate::{config::Config, state::AppState};

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

    let app = Router::new()
        .route("/health", get(|| async { "ok" }))
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}");
    axum::serve(listener, app).await.unwrap();
}
```

- [ ] **Step 6: Verify compile**

```bash
cd backend && cargo build
```

Expected: compiles without errors.

- [ ] **Step 7: Commit**

```bash
git add backend/src/
git commit -m "feat: config, error types, app state"
```

---

### Task 5: Auth — tokens & password hashing

**Files:**
- Create: `backend/src/auth/mod.rs`
- Create: `backend/src/auth/tokens.rs`

- [ ] **Step 1: Write failing test for JWT round-trip**

```rust
// src/auth/tokens.rs
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AdminRole { Admin, SuperAdmin }

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: AdminRole,
    pub exp: i64,
}

pub fn create_access_token(admin_id: Uuid, role: AdminRole, secret: &str, expiry_seconds: u64) -> String {
    let exp = (Utc::now() + Duration::seconds(expiry_seconds as i64)).timestamp();
    let claims = Claims { sub: admin_id, role, exp };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap()
}

pub fn validate_access_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let data = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_bytes()), &Validation::default())?;
    Ok(data.claims)
}

pub fn hash_password(password: &str) -> String {
    use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default().hash_password(password.as_bytes(), &salt).unwrap().to_string()
}

pub fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{password_hash::{PasswordHash, PasswordVerifier}, Argon2};
    let parsed = PasswordHash::new(hash).unwrap();
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

pub fn generate_refresh_token() -> String {
    use rand::Rng;
    rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect()
}

pub fn hash_refresh_token(token: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    // Use SHA-256 in production; here we use a simple approach for the type system
    // Actually use SHA2:
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwt_round_trip() {
        let id = Uuid::new_v4();
        let token = create_access_token(id, AdminRole::SuperAdmin, "secret", 900);
        let claims = validate_access_token(&token, "secret").unwrap();
        assert_eq!(claims.sub, id);
        assert_eq!(claims.role, AdminRole::SuperAdmin);
    }

    #[test]
    fn test_invalid_jwt_rejected() {
        let result = validate_access_token("invalid.token.here", "secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_hash_and_verify() {
        let hash = hash_password("hunter2");
        assert!(verify_password("hunter2", &hash));
        assert!(!verify_password("wrong", &hash));
    }
}
```

- [ ] **Step 2: Add sha2 to Cargo.toml**

```toml
sha2 = "0.10"
```

- [ ] **Step 3: Write auth/mod.rs**

```rust
// src/auth/mod.rs
pub mod tokens;
pub mod middleware;
pub mod handlers;
```

- [ ] **Step 4: Add auth mod to main.rs**

```rust
mod auth;
```

- [ ] **Step 5: Run tests**

```bash
cd backend && cargo test auth::tokens
```

Expected: 3 tests pass.

- [ ] **Step 6: Commit**

```bash
git add backend/src/auth/ backend/Cargo.toml
git commit -m "feat: auth tokens and password hashing"
```

---

### Task 6: Auth — handlers (login, refresh, logout)

**Files:**
- Create: `backend/src/auth/handlers.rs`

- [ ] **Step 1: Write the handlers**

```rust
// src/auth/handlers.rs
use axum::{
    extract::State,
    http::{header, HeaderMap},
    Json,
};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::{error::AppError, state::AppState};
use super::tokens::{create_access_token, hash_refresh_token,
    generate_refresh_token, validate_access_token, verify_password, AdminRole};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub admin_id: Uuid,
    pub role: AdminRole,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<(HeaderMap, Json<LoginResponse>), AppError> {
    let admin = sqlx::query!(
        "SELECT id, password_hash, role as \"role: String\" FROM admins WHERE username = $1",
        body.username
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::Unauthorized)?;

    if !verify_password(&body.password, &admin.password_hash) {
        return Err(AppError::Unauthorized);
    }

    let role = match admin.role.as_str() {
        "SuperAdmin" => AdminRole::SuperAdmin,
        _ => AdminRole::Admin,
    };

    let access_token = create_access_token(
        admin.id,
        role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_expiry_seconds,
    );

    let refresh_token = generate_refresh_token();
    let token_hash = hash_refresh_token(&refresh_token);
    let expires_at = Utc::now() + Duration::days(state.config.refresh_token_expiry_days);

    sqlx::query!(
        "INSERT INTO refresh_tokens (admin_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        admin.id, token_hash, expires_at
    )
    .execute(&state.db)
    .await?;

    let cookie = format!(
        "refresh_token={refresh_token}; HttpOnly; SameSite=Strict; Path=/auth/refresh; Max-Age={}",
        state.config.refresh_token_expiry_days * 86400
    );
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, Json(LoginResponse { access_token, admin_id: admin.id, role })))
}

pub async fn refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<LoginResponse>, AppError> {
    let cookie = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find(|p| p.trim().starts_with("refresh_token=")))
        .and_then(|p| p.trim().strip_prefix("refresh_token="))
        .ok_or(AppError::Unauthorized)?
        .to_owned();

    let token_hash = hash_refresh_token(&cookie);

    let row = sqlx::query!(
        r#"SELECT rt.admin_id, rt.expires_at, a.role as "role: String"
           FROM refresh_tokens rt
           JOIN admins a ON a.id = rt.admin_id
           WHERE rt.token_hash = $1"#,
        token_hash
    )
    .fetch_optional(&state.db)
    .await?
    .ok_or(AppError::Unauthorized)?;

    if row.expires_at < Utc::now() {
        return Err(AppError::Unauthorized);
    }

    let role = match row.role.as_str() {
        "SuperAdmin" => AdminRole::SuperAdmin,
        _ => AdminRole::Admin,
    };

    let access_token = create_access_token(
        row.admin_id,
        role.clone(),
        &state.config.jwt_secret,
        state.config.jwt_expiry_seconds,
    );

    Ok(Json(LoginResponse { access_token, admin_id: row.admin_id, role }))
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(cookie) = headers
        .get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(';').find(|p| p.trim().starts_with("refresh_token=")))
        .and_then(|p| p.trim().strip_prefix("refresh_token="))
    {
        let hash = hash_refresh_token(cookie);
        sqlx::query!("DELETE FROM refresh_tokens WHERE token_hash = $1", hash)
            .execute(&state.db)
            .await?;
    }
    Ok(Json(serde_json::json!({"ok": true})))
}
```

- [ ] **Step 2: Register auth routes in main.rs**

```rust
// In main.rs, add after imports:
use axum::routing::post;
use auth::handlers::{login, refresh, logout};

// In main(), replace app definition:
let app = Router::new()
    .route("/health", get(|| async { "ok" }))
    .route("/auth/login", post(login))
    .route("/auth/refresh", post(refresh))
    .route("/auth/logout", post(logout))
    .with_state(state);
```

- [ ] **Step 3: Build to verify**

```bash
cd backend && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add backend/src/
git commit -m "feat: auth login/refresh/logout handlers"
```

---

### Task 7: Auth middleware (JWT extractor + SuperAdmin guard)

**Files:**
- Create: `backend/src/auth/middleware.rs`

- [ ] **Step 1: Write middleware**

```rust
// src/auth/middleware.rs
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use crate::{error::AppError, state::AppState};
use super::tokens::{validate_access_token, AdminRole, Claims};

pub struct AuthClaims(pub Claims);
pub struct SuperAdminClaims(pub Claims);

#[async_trait]
impl FromRequestParts<AppState> for AuthClaims {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Unauthorized)?;

        validate_access_token(token, &state.config.jwt_secret)
            .map(AuthClaims)
            .map_err(|_| AppError::Unauthorized)
    }
}

#[async_trait]
impl FromRequestParts<AppState> for SuperAdminClaims {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let AuthClaims(claims) = AuthClaims::from_request_parts(parts, state).await?;
        if claims.role != AdminRole::SuperAdmin {
            return Err(AppError::Forbidden);
        }
        Ok(SuperAdminClaims(claims))
    }
}
```

- [ ] **Step 2: Build to verify**

```bash
cd backend && cargo build
```

- [ ] **Step 3: Commit**

```bash
git add backend/src/auth/middleware.rs
git commit -m "feat: JWT extractor and SuperAdmin guard middleware"
```

---

### Task 8: Admin management (CRUD, SuperAdmin only)

**Files:**
- Create: `backend/src/admins/mod.rs`
- Create: `backend/src/admins/model.rs`
- Create: `backend/src/admins/repository.rs`
- Create: `backend/src/admins/handlers.rs`

- [ ] **Step 1: Write model.rs**

```rust
// src/admins/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::auth::tokens::AdminRole;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Admin {
    pub id: Uuid,
    pub username: String,
    pub role: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateAdminRequest {
    pub username: String,
    pub password: String,
    pub role: Option<String>, // "Admin" | "SuperAdmin", defaults to "Admin"
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub password: String,
}
```

- [ ] **Step 2: Write repository.rs**

```rust
// src/admins/repository.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::Admin;

pub async fn list_admins(db: &PgPool) -> Result<Vec<Admin>, AppError> {
    let admins = sqlx::query_as!(
        Admin,
        "SELECT id, username, role as \"role: String\", created_at FROM admins ORDER BY created_at"
    )
    .fetch_all(db)
    .await?;
    Ok(admins)
}

pub async fn create_admin(db: &PgPool, username: &str, password_hash: &str, role: &str) -> Result<Admin, AppError> {
    let admin = sqlx::query_as!(
        Admin,
        r#"INSERT INTO admins (username, password_hash, role)
           VALUES ($1, $2, $3::admin_role)
           RETURNING id, username, role as "role: String", created_at"#,
        username, password_hash, role
    )
    .fetch_one(db)
    .await?;
    Ok(admin)
}

pub async fn delete_admin(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    let result = sqlx::query!("DELETE FROM admins WHERE id = $1", id)
        .execute(db)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_password(db: &PgPool, id: Uuid, password_hash: &str) -> Result<bool, AppError> {
    let result = sqlx::query!(
        "UPDATE admins SET password_hash = $1 WHERE id = $2",
        password_hash, id
    )
    .execute(db)
    .await?;
    Ok(result.rows_affected() > 0)
}
```

- [ ] **Step 3: Write handlers.rs**

```rust
// src/admins/handlers.rs
use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{auth::{middleware::SuperAdminClaims, tokens::hash_password}, error::AppError, state::AppState};
use super::{model::{Admin, ChangePasswordRequest, CreateAdminRequest}, repository};

pub async fn list_admins(
    State(state): State<AppState>,
    _: SuperAdminClaims,
) -> Result<Json<Vec<Admin>>, AppError> {
    Ok(Json(repository::list_admins(&state.db).await?))
}

pub async fn create_admin(
    State(state): State<AppState>,
    _: SuperAdminClaims,
    Json(body): Json<CreateAdminRequest>,
) -> Result<Json<Admin>, AppError> {
    let role = body.role.unwrap_or_else(|| "Admin".into());
    if role != "Admin" && role != "SuperAdmin" {
        return Err(AppError::Validation(vec![("role".into(), "must be Admin or SuperAdmin".into())]));
    }
    let hash = hash_password(&body.password);
    Ok(Json(repository::create_admin(&state.db, &body.username, &hash, &role).await?))
}

pub async fn delete_admin(
    State(state): State<AppState>,
    _: SuperAdminClaims,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let found = repository::delete_admin(&state.db, id).await?;
    if !found { return Err(AppError::NotFound("Admin not found".into())); }
    Ok(Json(serde_json::json!({"ok": true})))
}

pub async fn change_password(
    State(state): State<AppState>,
    _: SuperAdminClaims,
    Path(id): Path<Uuid>,
    Json(body): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let hash = hash_password(&body.password);
    let found = repository::update_password(&state.db, id, &hash).await?;
    if !found { return Err(AppError::NotFound("Admin not found".into())); }
    Ok(Json(serde_json::json!({"ok": true})))
}
```

- [ ] **Step 4: Write mod.rs**

```rust
// src/admins/mod.rs
pub mod model;
pub mod repository;
pub mod handlers;
```

- [ ] **Step 5: Register routes in main.rs**

```rust
mod admins;
use axum::routing::{delete, put};
use admins::handlers::{list_admins, create_admin, delete_admin, change_password};

// In app router:
.route("/api/v1/admins", get(list_admins).post(create_admin))
.route("/api/v1/admins/:id", delete(delete_admin))
.route("/api/v1/admins/:id/password", put(change_password))
```

- [ ] **Step 6: Build**

```bash
cd backend && cargo build
```

- [ ] **Step 7: Commit**

```bash
git add backend/src/admins/ backend/src/main.rs
git commit -m "feat: admin management endpoints (SuperAdmin only)"
```

---

### Task 9: Members — model & repository

**Files:**
- Create: `backend/src/members/mod.rs`
- Create: `backend/src/members/model.rs`
- Create: `backend/src/members/repository.rs`

- [ ] **Step 1: Write model.rs**

```rust
// src/members/model.rs
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Member {
    pub id: Uuid,
    pub version: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub membership_type: String,
    pub joined_at: NaiveDate,
    pub left_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub custom_fields: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateMemberRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub membership_type: String,
    pub joined_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub custom_fields: Option<Value>,
}

#[derive(Deserialize)]
pub struct UpdateMemberRequest {
    pub version: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub city: Option<String>,
    pub postal_code: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub membership_type: String,
    pub joined_at: NaiveDate,
    pub left_at: Option<NaiveDate>,
    pub notes: Option<String>,
    pub custom_fields: Option<Value>,
}

#[derive(Deserialize)]
pub struct MemberListQuery {
    pub search: Option<String>,
    pub membership_type: Option<String>,
    pub include_left: Option<bool>,
}
```

- [ ] **Step 2: Write repository.rs**

```rust
// src/members/repository.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::{CreateMemberRequest, Member, MemberListQuery, UpdateMemberRequest};

pub async fn list_members(db: &PgPool, query: &MemberListQuery) -> Result<Vec<Member>, AppError> {
    let include_left = query.include_left.unwrap_or(false);
    let members = sqlx::query_as!(
        Member,
        r#"SELECT id, version, first_name, last_name, email, phone, street, city,
                  postal_code, birth_date, membership_type as "membership_type: String",
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
    let member = sqlx::query_as!(
        Member,
        r#"SELECT id, version, first_name, last_name, email, phone, street, city,
                  postal_code, birth_date, membership_type as "membership_type: String",
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
    let member = sqlx::query_as!(
        Member,
        r#"INSERT INTO members (first_name, last_name, email, phone, street, city,
               postal_code, birth_date, membership_type, joined_at, notes, custom_fields)
           VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9::membership_type,$10,$11,$12)
           RETURNING id, version, first_name, last_name, email, phone, street, city,
                     postal_code, birth_date, membership_type as "membership_type: String",
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
    // Attempt update only if version matches
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

    let member = sqlx::query_as!(
        Member,
        r#"UPDATE members SET
               version = version + 1,
               first_name=$2, last_name=$3, email=$4, phone=$5,
               street=$6, city=$7, postal_code=$8, birth_date=$9,
               membership_type=$10::membership_type, joined_at=$11, left_at=$12,
               notes=$13, custom_fields=$14, updated_at=NOW()
           WHERE id=$1
           RETURNING id, version, first_name, last_name, email, phone, street, city,
                     postal_code, birth_date, membership_type as "membership_type: String",
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
```

- [ ] **Step 3: Write mod.rs**

```rust
// src/members/mod.rs
pub mod model;
pub mod repository;
pub mod handlers;
```

- [ ] **Step 4: Build**

```bash
cd backend && cargo build
```

- [ ] **Step 5: Commit**

```bash
git add backend/src/members/
git commit -m "feat: member model and repository"
```

---

### Task 10: Members — handlers & CSV export

**Files:**
- Create: `backend/src/members/handlers.rs`

- [ ] **Step 1: Write handlers**

```rust
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
    // Broadcast presence event
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
```

- [ ] **Step 2: Register member routes in main.rs**

```rust
mod members;
use members::handlers::{list_members, get_member, create_member, update_member, delete_member, export_members};

// In router:
.route("/api/v1/members", get(list_members).post(create_member))
.route("/api/v1/members/export", get(export_members))
.route("/api/v1/members/:id", get(get_member).put(update_member).delete(delete_member))
```

- [ ] **Step 3: Build**

```bash
cd backend && cargo build
```

- [ ] **Step 4: Commit**

```bash
git add backend/src/members/handlers.rs backend/src/main.rs
git commit -m "feat: member CRUD handlers and CSV export"
```

---

### Task 11: Roles & Field Definitions

**Files:**
- Create: `backend/src/roles/` (mod.rs, model.rs, repository.rs, handlers.rs)
- Create: `backend/src/field_definitions/` (mod.rs, model.rs, repository.rs, handlers.rs)

- [ ] **Step 1: Write roles/model.rs**

```rust
// src/roles/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
}
```

- [ ] **Step 2: Write roles/repository.rs**

```rust
// src/roles/repository.rs
use sqlx::PgPool;
use uuid::Uuid;
use crate::error::AppError;
use super::model::Role;

pub async fn list_roles(db: &PgPool) -> Result<Vec<Role>, AppError> {
    Ok(sqlx::query_as!(Role, "SELECT id, name, created_at FROM roles ORDER BY name")
        .fetch_all(db).await?)
}

pub async fn create_role(db: &PgPool, name: &str) -> Result<Role, AppError> {
    Ok(sqlx::query_as!(Role,
        "INSERT INTO roles (name) VALUES ($1) RETURNING id, name, created_at", name)
        .fetch_one(db).await?)
}

pub async fn delete_role(db: &PgPool, id: Uuid) -> Result<bool, AppError> {
    Ok(sqlx::query!("DELETE FROM roles WHERE id = $1", id)
        .execute(db).await?.rows_affected() > 0)
}
```

- [ ] **Step 3: Write roles/handlers.rs**

```rust
// src/roles/handlers.rs
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
```

- [ ] **Step 4: Write roles/mod.rs**

```rust
pub mod model;
pub mod repository;
pub mod handlers;
```

- [ ] **Step 5: Write field_definitions/model.rs**

```rust
// src/field_definitions/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct FieldDefinition {
    pub id: Uuid,
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateFieldRequest {
    pub name: String,
    pub field_type: String, // "text" | "number" | "date" | "boolean"
    pub required: Option<bool>,
    pub display_order: Option<i32>,
}
```

- [ ] **Step 6: Write field_definitions/repository.rs**

```rust
// src/field_definitions/repository.rs
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
    Ok(sqlx::query_as!(
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
```

- [ ] **Step 7: Write field_definitions/handlers.rs**

```rust
// src/field_definitions/handlers.rs
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
```

- [ ] **Step 8: Write field_definitions/mod.rs**

```rust
pub mod model;
pub mod repository;
pub mod handlers;
```

- [ ] **Step 9: Register routes in main.rs**

```rust
mod roles;
mod field_definitions;
use roles::handlers::{list_roles, create_role, delete_role};
use field_definitions::handlers::{list_fields, create_field, delete_field};

// In router:
.route("/api/v1/roles", get(list_roles).post(create_role))
.route("/api/v1/roles/:id", axum::routing::delete(delete_role))
.route("/api/v1/field-definitions", get(list_fields).post(create_field))
.route("/api/v1/field-definitions/:id", axum::routing::delete(delete_field))
```

- [ ] **Step 10: Build**

```bash
cd backend && cargo build
```

- [ ] **Step 11: Commit**

```bash
git add backend/src/roles/ backend/src/field_definitions/ backend/src/main.rs
git commit -m "feat: roles and field definitions endpoints"
```

---

### Task 12: WebSocket presence

**Files:**
- Create: `backend/src/ws/mod.rs`
- Create: `backend/src/ws/handler.rs`

- [ ] **Step 1: Write ws/handler.rs**

```rust
// src/ws/handler.rs
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::Response,
};
use serde::Deserialize;
use crate::{auth::tokens::validate_access_token, state::AppState};

#[derive(Deserialize)]
pub struct WsQuery {
    pub token: String,
}

pub async fn ws_handler(
    State(state): State<AppState>,
    Query(query): Query<WsQuery>,
    ws: WebSocketUpgrade,
) -> Response {
    // Validate token before upgrading
    if validate_access_token(&query.token, &state.config.jwt_secret).is_err() {
        return axum::response::IntoResponse::into_response((
            axum::http::StatusCode::UNAUTHORIZED,
            "Invalid token",
        ));
    }
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut rx = state.ws_tx.subscribe();
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Ok(event) => {
                        if socket.send(Message::Text(event)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        // Broadcast presence events from clients (e.g. "viewing member X")
                        let _ = state.ws_tx.send(text);
                    }
                    _ => break,
                }
            }
        }
    }
}
```

- [ ] **Step 2: Write ws/mod.rs**

```rust
pub mod handler;
```

- [ ] **Step 3: Register WS route in main.rs**

```rust
mod ws;
use ws::handler::ws_handler;

// In router:
.route("/ws", get(ws_handler))
```

- [ ] **Step 4: Build**

```bash
cd backend && cargo build
```

- [ ] **Step 5: Commit**

```bash
git add backend/src/ws/ backend/src/main.rs
git commit -m "feat: WebSocket presence handler"
```

---

### Task 13: CORS & finalize router + seed script

**Files:**
- Modify: `backend/src/main.rs`
- Create: `backend/src/bin/seed.rs`

- [ ] **Step 1: Add CORS to main.rs**

```rust
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;

// In main(), before .with_state():
let cors = CorsLayer::new()
    .allow_origin(state.config.frontend_url.parse::<axum::http::HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
    .allow_headers(Any)
    .allow_credentials(true);

let app = Router::new()
    // ... all routes ...
    .layer(cors)
    .with_state(state);
```

- [ ] **Step 2: Write seed binary**

```rust
// src/bin/seed.rs
//! Usage: cargo run --bin seed -- --username admin --password secret --role SuperAdmin
use sqlx::postgres::PgPoolOptions;
use argon2::{password_hash::{rand_core::OsRng, PasswordHasher, SaltString}, Argon2};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().collect();
    let username = args.iter().position(|a| a == "--username")
        .map(|i| args[i + 1].clone()).unwrap_or("admin".into());
    let password = args.iter().position(|a| a == "--password")
        .map(|i| args[i + 1].clone()).expect("--password required");
    let role = args.iter().position(|a| a == "--role")
        .map(|i| args[i + 1].clone()).unwrap_or("SuperAdmin".into());

    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default().hash_password(password.as_bytes(), &salt).unwrap().to_string();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let pool = PgPoolOptions::new().connect(&db_url).await.unwrap();

    sqlx::query!(
        "INSERT INTO admins (username, password_hash, role) VALUES ($1, $2, $3::admin_role) ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash",
        username, hash, role
    )
    .execute(&pool)
    .await
    .unwrap();

    println!("Admin '{username}' ({role}) created/updated.");
}
```

- [ ] **Step 3: Seed the first SuperAdmin**

```bash
cd backend && cargo run --bin seed -- --username admin --password changeme --role SuperAdmin
```

Expected: `Admin 'admin' (SuperAdmin) created/updated.`

- [ ] **Step 4: Build and smoke test**

```bash
cd backend && cargo run &
sleep 2
curl -s -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"changeme"}' | jq .
# Expected: access_token, admin_id, role fields
kill %1
```

- [ ] **Step 5: Commit**

```bash
git add backend/src/
git commit -m "feat: CORS config and SuperAdmin seed script"
```

---

### Task 14: Frontend init

**Files:**
- Create: `frontend/` (Vite project)

- [ ] **Step 1: Scaffold frontend**

```bash
cd /Users/tobi/Documents/Coding/Vereinssoftware
npm create vite@latest frontend -- --template react-ts
cd frontend && npm install
```

- [ ] **Step 2: Install dependencies**

```bash
npm install @tanstack/react-query axios react-router-dom
npm install -D tailwindcss postcss autoprefixer vitest @testing-library/react @testing-library/jest-dom @vitejs/plugin-react jsdom
npx tailwindcss init -p
```

- [ ] **Step 3: Configure tailwind.config.ts**

```ts
// frontend/tailwind.config.ts
export default {
  content: ["./index.html", "./src/**/*.{ts,tsx}"],
  theme: { extend: {} },
  plugins: [],
}
```

- [ ] **Step 4: Add Tailwind to src/index.css**

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

- [ ] **Step 5: Configure vitest in vite.config.ts**

```ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: { proxy: { '/api': 'http://localhost:3000', '/auth': 'http://localhost:3000', '/ws': { target: 'ws://localhost:3000', ws: true } } },
  test: { environment: 'jsdom', setupFiles: ['./src/test-setup.ts'] },
})
```

- [ ] **Step 6: Create test-setup.ts**

```ts
// src/test-setup.ts
import '@testing-library/jest-dom'
```

- [ ] **Step 7: Verify dev server starts**

```bash
cd frontend && npm run dev &
curl -s http://localhost:5173 | head -5
kill %1
```

- [ ] **Step 8: Commit**

```bash
git add frontend/
git commit -m "chore: init React/TypeScript frontend with Vite"
```

---

### Task 15: Types & API client

**Files:**
- Create: `frontend/src/types/index.ts`
- Create: `frontend/src/api/client.ts`
- Create: `frontend/src/api/auth.ts`
- Create: `frontend/src/api/members.ts`
- Create: `frontend/src/api/roles.ts`
- Create: `frontend/src/api/fieldDefinitions.ts`
- Create: `frontend/src/api/admins.ts`

- [ ] **Step 1: Write types/index.ts**

```ts
// src/types/index.ts
export type AdminRole = 'Admin' | 'SuperAdmin'
export type MembershipType = 'Aktiv' | 'Passiv' | 'Ehrenmitglied'
export type FieldType = 'text' | 'number' | 'date' | 'boolean'

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

export interface Role {
  id: string
  name: string
  created_at: string
}

export interface FieldDefinition {
  id: string
  name: string
  field_type: FieldType
  required: boolean
  display_order: number
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

- [ ] **Step 2: Write api/client.ts with token refresh interceptor**

```ts
// src/api/client.ts
import axios, { AxiosError } from 'axios'

export const client = axios.create({ withCredentials: true })

// Attach token from memory
let accessToken: string | null = null
export const setAccessToken = (t: string | null) => { accessToken = t }
export const getAccessToken = () => accessToken

client.interceptors.request.use(config => {
  if (accessToken) config.headers.Authorization = `Bearer ${accessToken}`
  return config
})

let refreshPromise: Promise<string> | null = null

client.interceptors.response.use(
  r => r,
  async (error: AxiosError) => {
    const original = error.config!
    if (error.response?.status === 401 && !(original as any)._retry) {
      ;(original as any)._retry = true
      if (!refreshPromise) {
        refreshPromise = axios.post<{ access_token: string }>('/auth/refresh', {}, { withCredentials: true })
          .then(r => { accessToken = r.data.access_token; return accessToken })
          .finally(() => { refreshPromise = null })
      }
      try {
        const token = await refreshPromise
        original.headers!.Authorization = `Bearer ${token}`
        return client(original)
      } catch {
        accessToken = null
        window.location.href = '/login'
      }
    }
    return Promise.reject(error)
  }
)
```

- [ ] **Step 3: Write api/auth.ts**

```ts
// src/api/auth.ts
import axios from 'axios'
import { setAccessToken } from './client'
import type { AuthState } from '../types'

export async function login(username: string, password: string): Promise<AuthState> {
  const { data } = await axios.post<AuthState>('/auth/login', { username, password }, { withCredentials: true })
  setAccessToken(data.access_token)
  return data
}

export async function logout(): Promise<void> {
  await axios.post('/auth/logout', {}, { withCredentials: true })
  setAccessToken(null)
}
```

- [ ] **Step 4: Write api/members.ts**

```ts
// src/api/members.ts
import { client } from './client'
import type { Member } from '../types'

export interface MemberListParams { search?: string; membership_type?: string; include_left?: boolean }
export interface UpdateMemberPayload extends Omit<Member, 'id' | 'created_at' | 'updated_at'> {}
export interface CreateMemberPayload extends Omit<Member, 'id' | 'version' | 'created_at' | 'updated_at' | 'left_at'> {}

export const getMembers = (params?: MemberListParams) =>
  client.get<Member[]>('/api/v1/members', { params }).then(r => r.data)

export const getMember = (id: string) =>
  client.get<Member>(`/api/v1/members/${id}`).then(r => r.data)

export const createMember = (data: CreateMemberPayload) =>
  client.post<Member>('/api/v1/members', data).then(r => r.data)

export const updateMember = (id: string, data: UpdateMemberPayload) =>
  client.put<Member>(`/api/v1/members/${id}`, data).then(r => r.data)

export const deleteMember = (id: string) =>
  client.delete(`/api/v1/members/${id}`).then(r => r.data)

export const exportMembers = (params?: MemberListParams) => {
  window.open(`/api/v1/members/export?${new URLSearchParams(params as any).toString()}`)
}
```

- [ ] **Step 5: Write api/roles.ts, api/fieldDefinitions.ts, api/admins.ts**

```ts
// src/api/roles.ts
import { client } from './client'
import type { Role } from '../types'
export const getRoles = () => client.get<Role[]>('/api/v1/roles').then(r => r.data)
export const createRole = (name: string) => client.post<Role>('/api/v1/roles', { name }).then(r => r.data)
export const deleteRole = (id: string) => client.delete(`/api/v1/roles/${id}`).then(r => r.data)
```

```ts
// src/api/fieldDefinitions.ts
import { client } from './client'
import type { FieldDefinition, FieldType } from '../types'
export const getFieldDefinitions = () => client.get<FieldDefinition[]>('/api/v1/field-definitions').then(r => r.data)
export const createFieldDefinition = (data: { name: string; field_type: FieldType; required?: boolean; display_order?: number }) =>
  client.post<FieldDefinition>('/api/v1/field-definitions', data).then(r => r.data)
export const deleteFieldDefinition = (id: string) => client.delete(`/api/v1/field-definitions/${id}`).then(r => r.data)
```

```ts
// src/api/admins.ts
import { client } from './client'
import type { Admin, AdminRole } from '../types'
export const getAdmins = () => client.get<Admin[]>('/api/v1/admins').then(r => r.data)
export const createAdmin = (data: { username: string; password: string; role?: AdminRole }) =>
  client.post<Admin>('/api/v1/admins', data).then(r => r.data)
export const deleteAdmin = (id: string) => client.delete(`/api/v1/admins/${id}`).then(r => r.data)
export const changePassword = (id: string, password: string) =>
  client.put(`/api/v1/admins/${id}/password`, { password }).then(r => r.data)
```

- [ ] **Step 6: Commit**

```bash
git add frontend/src/types/ frontend/src/api/
git commit -m "feat: TypeScript types and API client layer"
```

---

### Task 16: Auth UI (login page, useAuth hook, routing)

**Files:**
- Create: `frontend/src/hooks/useAuth.ts`
- Create: `frontend/src/components/ProtectedRoute.tsx`
- Create: `frontend/src/components/SuperAdminRoute.tsx`
- Create: `frontend/src/pages/LoginPage.tsx`
- Create: `frontend/src/App.tsx`
- Create: `frontend/src/main.tsx`

- [ ] **Step 1: Write useAuth.ts**

```ts
// src/hooks/useAuth.ts
import { createContext, useContext, useState, useEffect, ReactNode, createElement } from 'react'
import { login as apiLogin, logout as apiLogout } from '../api/auth'
import { setAccessToken } from '../api/client'
import type { AdminRole, AuthState } from '../types'

interface AuthCtx {
  auth: AuthState | null
  login: (username: string, password: string) => Promise<void>
  logout: () => Promise<void>
  isLoading: boolean
}

const AuthContext = createContext<AuthCtx>(null!)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [auth, setAuth] = useState<AuthState | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    // Try silent refresh on load
    import('axios').then(({ default: axios }) =>
      axios.post<AuthState>('/auth/refresh', {}, { withCredentials: true })
        .then(r => { setAccessToken(r.data.access_token); setAuth(r.data) })
        .catch(() => {})
        .finally(() => setIsLoading(false))
    )
  }, [])

  const login = async (username: string, password: string) => {
    const data = await apiLogin(username, password)
    setAuth(data)
  }

  const logout = async () => {
    await apiLogout()
    setAuth(null)
  }

  return createElement(AuthContext.Provider, { value: { auth, login, logout, isLoading } }, children)
}

export const useAuth = () => useContext(AuthContext)
```

- [ ] **Step 2: Write ProtectedRoute.tsx and SuperAdminRoute.tsx**

```tsx
// src/components/ProtectedRoute.tsx
import { Navigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { auth, isLoading } = useAuth()
  if (isLoading) return <div className="p-8 text-center">Loading...</div>
  if (!auth) return <Navigate to="/login" replace />
  return <>{children}</>
}
```

```tsx
// src/components/SuperAdminRoute.tsx
import { Navigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export function SuperAdminRoute({ children }: { children: React.ReactNode }) {
  const { auth, isLoading } = useAuth()
  if (isLoading) return null
  if (!auth) return <Navigate to="/login" replace />
  if (auth.role !== 'SuperAdmin') return <Navigate to="/members" replace />
  return <>{children}</>
}
```

- [ ] **Step 3: Write LoginPage.tsx**

```tsx
// src/pages/LoginPage.tsx
import { useState, FormEvent } from 'react'
import { useNavigate } from 'react-router-dom'
import { useAuth } from '../hooks/useAuth'

export function LoginPage() {
  const { login } = useAuth()
  const navigate = useNavigate()
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      await login(username, password)
      navigate('/members')
    } catch {
      setError('Invalid username or password')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <form onSubmit={handleSubmit} className="bg-white p-8 rounded-lg shadow w-full max-w-sm space-y-4">
        <h1 className="text-2xl font-bold">Vereinssoftware</h1>
        {error && <p className="text-red-600 text-sm">{error}</p>}
        <div>
          <label className="block text-sm font-medium mb-1">Username</label>
          <input className="border rounded w-full px-3 py-2" value={username} onChange={e => setUsername(e.target.value)} required />
        </div>
        <div>
          <label className="block text-sm font-medium mb-1">Password</label>
          <input type="password" className="border rounded w-full px-3 py-2" value={password} onChange={e => setPassword(e.target.value)} required />
        </div>
        <button type="submit" disabled={loading} className="w-full bg-blue-600 text-white py-2 rounded hover:bg-blue-700 disabled:opacity-50">
          {loading ? 'Logging in...' : 'Login'}
        </button>
      </form>
    </div>
  )
}
```

- [ ] **Step 4: Write App.tsx**

```tsx
// src/App.tsx
import { BrowserRouter, Routes, Route, Navigate, Link } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { AuthProvider, useAuth } from './hooks/useAuth'
import { ProtectedRoute } from './components/ProtectedRoute'
import { SuperAdminRoute } from './components/SuperAdminRoute'
import { LoginPage } from './pages/LoginPage'
import { MembersPage } from './pages/MembersPage'
import { MemberDetailPage } from './pages/MemberDetailPage'
import { MemberNewPage } from './pages/MemberNewPage'
import { RolesPage } from './pages/RolesPage'
import { FieldsPage } from './pages/FieldsPage'
import { AdminsPage } from './pages/AdminsPage'

const queryClient = new QueryClient()

function Nav() {
  const { auth, logout } = useAuth()
  if (!auth) return null
  return (
    <nav className="bg-gray-800 text-white px-6 py-3 flex gap-6 items-center">
      <Link to="/members" className="hover:text-blue-300">Members</Link>
      <Link to="/settings/roles" className="hover:text-blue-300">Roles</Link>
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
            <Route path="/settings/roles" element={<ProtectedRoute><RolesPage /></ProtectedRoute>} />
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

- [ ] **Step 5: Update main.tsx**

```tsx
// src/main.tsx
import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App'
import './index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode><App /></React.StrictMode>
)
```

- [ ] **Step 6: Write failing test for LoginPage**

```tsx
// src/pages/LoginPage.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react'
import { MemoryRouter } from 'react-router-dom'
import { describe, it, expect, vi } from 'vitest'

vi.mock('../hooks/useAuth', () => ({
  useAuth: () => ({
    login: vi.fn().mockRejectedValueOnce(new Error('Unauthorized')),
  }),
}))

import { LoginPage } from './LoginPage'

describe('LoginPage', () => {
  it('shows error on failed login', async () => {
    render(<MemoryRouter><LoginPage /></MemoryRouter>)
    fireEvent.change(screen.getByLabelText(/username/i), { target: { value: 'admin' } })
    fireEvent.change(screen.getByLabelText(/password/i), { target: { value: 'wrong' } })
    fireEvent.click(screen.getByRole('button', { name: /login/i }))
    await waitFor(() => expect(screen.getByText(/invalid username or password/i)).toBeInTheDocument())
  })
})
```

- [ ] **Step 7: Run tests**

```bash
cd frontend && npx vitest run src/pages/LoginPage.test.tsx
```

Expected: 1 test passes.

- [ ] **Step 8: Commit**

```bash
git add frontend/src/
git commit -m "feat: auth UI — login page, useAuth hook, routing"
```

---

### Task 17: Members list page

**Files:**
- Create: `frontend/src/pages/MembersPage.tsx`

- [ ] **Step 1: Write MembersPage.tsx**

```tsx
// src/pages/MembersPage.tsx
import { useState } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { getMembers, exportMembers } from '../api/members'

export function MembersPage() {
  const navigate = useNavigate()
  const [search, setSearch] = useState('')
  const [membershipType, setMembershipType] = useState('')
  const [includeLeft, setIncludeLeft] = useState(false)

  const { data: members = [], isLoading } = useQuery({
    queryKey: ['members', search, membershipType, includeLeft],
    queryFn: () => getMembers({ search: search || undefined, membership_type: membershipType || undefined, include_left: includeLeft }),
  })

  return (
    <div className="p-6 max-w-5xl mx-auto">
      <div className="flex justify-between items-center mb-4">
        <h1 className="text-2xl font-bold">Members ({members.length})</h1>
        <div className="flex gap-2">
          <button onClick={() => exportMembers({ search: search || undefined, membership_type: membershipType || undefined })}
            className="px-3 py-2 border rounded hover:bg-gray-50 text-sm">Export CSV</button>
          <button onClick={() => navigate('/members/new')}
            className="px-3 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 text-sm">+ New Member</button>
        </div>
      </div>

      <div className="flex gap-3 mb-4">
        <input placeholder="Search name..." value={search} onChange={e => setSearch(e.target.value)}
          className="border rounded px-3 py-2 flex-1" />
        <select value={membershipType} onChange={e => setMembershipType(e.target.value)} className="border rounded px-3 py-2">
          <option value="">All types</option>
          <option value="Aktiv">Aktiv</option>
          <option value="Passiv">Passiv</option>
          <option value="Ehrenmitglied">Ehrenmitglied</option>
        </select>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={includeLeft} onChange={e => setIncludeLeft(e.target.checked)} />
          Include former
        </label>
      </div>

      {isLoading ? <p>Loading...</p> : (
        <table className="w-full border-collapse">
          <thead>
            <tr className="bg-gray-100 text-left text-sm">
              <th className="px-3 py-2 border">Name</th>
              <th className="px-3 py-2 border">Email</th>
              <th className="px-3 py-2 border">Type</th>
              <th className="px-3 py-2 border">Joined</th>
              <th className="px-3 py-2 border">Status</th>
            </tr>
          </thead>
          <tbody>
            {members.map(m => (
              <tr key={m.id} onClick={() => navigate(`/members/${m.id}`)}
                className="hover:bg-blue-50 cursor-pointer">
                <td className="px-3 py-2 border">{m.last_name}, {m.first_name}</td>
                <td className="px-3 py-2 border text-gray-600">{m.email ?? '—'}</td>
                <td className="px-3 py-2 border">{m.membership_type}</td>
                <td className="px-3 py-2 border">{m.joined_at}</td>
                <td className="px-3 py-2 border">{m.left_at ? <span className="text-red-500">Left</span> : <span className="text-green-600">Active</span>}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Commit**

```bash
git add frontend/src/pages/MembersPage.tsx
git commit -m "feat: member list page with search and filters"
```

---

### Task 18: MemberForm + detail and new pages

**Files:**
- Create: `frontend/src/components/MemberForm.tsx`
- Create: `frontend/src/pages/MemberDetailPage.tsx`
- Create: `frontend/src/pages/MemberNewPage.tsx`

- [ ] **Step 1: Write MemberForm.tsx**

```tsx
// src/components/MemberForm.tsx
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
                {f.field_type === 'boolean' ? (
                  <input type="checkbox" disabled={disabled}
                    checked={!!(value.custom_fields as any)?.[f.name]}
                    onChange={e => set('custom_fields', { ...(value.custom_fields as any), [f.name]: e.target.checked })} />
                ) : (
                  <input type={f.field_type === 'number' ? 'number' : f.field_type === 'date' ? 'date' : 'text'}
                    disabled={disabled} className="border rounded w-full px-3 py-2"
                    value={(value.custom_fields as any)?.[f.name] ?? ''}
                    onChange={e => set('custom_fields', { ...(value.custom_fields as any), [f.name]: e.target.value || null })} />
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

- [ ] **Step 2: Write MemberDetailPage.tsx**

```tsx
// src/pages/MemberDetailPage.tsx
import { useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getMember, updateMember, deleteMember } from '../api/members'
import { MemberForm } from '../components/MemberForm'
import { ConflictDialog } from '../components/ConflictDialog'
import { PresenceIndicator } from '../components/PresenceIndicator'
import type { Member, ConflictError } from '../types'
import { AxiosError } from 'axios'

export function MemberDetailPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [draft, setDraft] = useState<Partial<Member> | null>(null)
  const [conflict, setConflict] = useState<{ serverMember: Member; myDraft: Partial<Member> } | null>(null)

  const { data: member, isLoading } = useQuery({
    queryKey: ['member', id],
    queryFn: () => getMember(id!),
    onSuccess: (m) => { if (!draft) setDraft(m) },
  })

  const { mutate: save, isLoading: isSaving } = useMutation({
    mutationFn: (data: Partial<Member>) => updateMember(id!, data as any),
    onSuccess: (updated) => { qc.setQueryData(['member', id], updated); setDraft(updated) },
    onError: async (err: AxiosError) => {
      if (err.response?.status === 409) {
        const serverMember = await getMember(id!)
        setConflict({ serverMember, myDraft: draft! })
      }
    },
  })

  const { mutate: remove } = useMutation({
    mutationFn: () => deleteMember(id!),
    onSuccess: () => { qc.invalidateQueries(['members']); navigate('/members') },
  })

  if (isLoading || !draft || !member) return <div className="p-6">Loading...</div>

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <div className="flex justify-between items-center mb-6">
        <h1 className="text-2xl font-bold">{member.first_name} {member.last_name}</h1>
        <PresenceIndicator memberId={id!} />
      </div>

      <MemberForm value={draft} onChange={setDraft} />

      <div className="flex gap-3 mt-6">
        <button onClick={() => save(draft)} disabled={isSaving}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">
          {isSaving ? 'Saving...' : 'Save'}
        </button>
        <button onClick={() => navigate('/members')} className="px-4 py-2 border rounded hover:bg-gray-50">
          Cancel
        </button>
        <button onClick={() => { if (confirm('Mark as left?')) remove() }}
          className="ml-auto px-4 py-2 text-red-600 border border-red-300 rounded hover:bg-red-50">
          Mark as Left
        </button>
      </div>

      {conflict && (
        <ConflictDialog
          myDraft={conflict.myDraft}
          serverMember={conflict.serverMember}
          onResolve={(resolved) => { setConflict(null); setDraft(resolved); save(resolved) }}
          onDiscard={() => { setConflict(null); setDraft(conflict.serverMember) }}
        />
      )}
    </div>
  )
}
```

- [ ] **Step 3: Write MemberNewPage.tsx**

```tsx
// src/pages/MemberNewPage.tsx
import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useMutation, useQueryClient } from '@tanstack/react-query'
import { createMember } from '../api/members'
import { MemberForm } from '../components/MemberForm'
import type { Member } from '../types'

export function MemberNewPage() {
  const navigate = useNavigate()
  const qc = useQueryClient()
  const [draft, setDraft] = useState<Partial<Member>>({})

  const { mutate, isLoading } = useMutation({
    mutationFn: () => createMember(draft as any),
    onSuccess: (m) => { qc.invalidateQueries(['members']); navigate(`/members/${m.id}`) },
  })

  return (
    <div className="p-6 max-w-2xl mx-auto">
      <h1 className="text-2xl font-bold mb-6">New Member</h1>
      <MemberForm value={draft} onChange={setDraft} />
      <div className="flex gap-3 mt-6">
        <button onClick={() => mutate()} disabled={isLoading}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">
          {isLoading ? 'Creating...' : 'Create Member'}
        </button>
        <button onClick={() => navigate('/members')} className="px-4 py-2 border rounded hover:bg-gray-50">
          Cancel
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/
git commit -m "feat: member form, detail page, and new member page"
```

---

### Task 19: Conflict dialog & presence indicator

**Files:**
- Create: `frontend/src/components/ConflictDialog.tsx`
- Create: `frontend/src/hooks/usePresence.ts`
- Create: `frontend/src/components/PresenceIndicator.tsx`

- [ ] **Step 1: Write ConflictDialog.tsx**

```tsx
// src/components/ConflictDialog.tsx
import type { Member } from '../types'

interface Props {
  myDraft: Partial<Member>
  serverMember: Member
  onResolve: (resolved: Partial<Member>) => void
  onDiscard: () => void
}

const FIELDS: (keyof Member)[] = ['first_name', 'last_name', 'email', 'phone', 'membership_type', 'notes']

export function ConflictDialog({ myDraft, serverMember, onResolve, onDiscard }: Props) {
  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl p-6 max-w-2xl w-full mx-4">
        <h2 className="text-xl font-bold mb-2 text-red-600">Edit Conflict</h2>
        <p className="text-sm text-gray-600 mb-4">
          Another admin saved changes while you were editing. Review the differences and choose which version to keep.
        </p>
        <table className="w-full text-sm border-collapse mb-4">
          <thead>
            <tr className="bg-gray-100">
              <th className="px-3 py-2 border text-left">Field</th>
              <th className="px-3 py-2 border text-left">Your version</th>
              <th className="px-3 py-2 border text-left">Server version</th>
            </tr>
          </thead>
          <tbody>
            {FIELDS.filter(f => myDraft[f] !== serverMember[f]).map(f => (
              <tr key={f}>
                <td className="px-3 py-2 border font-medium">{f}</td>
                <td className="px-3 py-2 border bg-yellow-50">{String(myDraft[f] ?? '—')}</td>
                <td className="px-3 py-2 border bg-green-50">{String(serverMember[f] ?? '—')}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <div className="flex gap-3">
          <button onClick={() => onResolve({ ...serverMember, ...myDraft, version: serverMember.version })}
            className="px-4 py-2 bg-yellow-500 text-white rounded hover:bg-yellow-600">
            Keep my changes
          </button>
          <button onClick={onDiscard}
            className="px-4 py-2 bg-green-600 text-white rounded hover:bg-green-700">
            Use server version
          </button>
        </div>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Write usePresence.ts**

```ts
// src/hooks/usePresence.ts
import { useEffect, useState, useRef } from 'react'
import { getAccessToken } from '../api/client'

export function usePresence(memberId: string) {
  const [viewers, setViewers] = useState<string[]>([])
  const wsRef = useRef<WebSocket | null>(null)

  useEffect(() => {
    const token = getAccessToken()
    if (!token) return

    const ws = new WebSocket(`ws://localhost:3000/ws?token=${token}`)
    wsRef.current = ws

    ws.onopen = () => {
      ws.send(JSON.stringify({ type: 'viewing', member_id: memberId }))
    }

    ws.onmessage = (e) => {
      try {
        const event = JSON.parse(e.data)
        if (event.type === 'viewing' && event.member_id === memberId) {
          setViewers(prev => [...new Set([...prev, event.username ?? 'Someone'])])
        }
        if (event.type === 'left' && event.member_id === memberId) {
          setViewers(prev => prev.filter(v => v !== event.username))
        }
      } catch {}
    }

    return () => {
      ws.send(JSON.stringify({ type: 'left', member_id: memberId }))
      ws.close()
    }
  }, [memberId])

  return viewers
}
```

- [ ] **Step 3: Write PresenceIndicator.tsx**

```tsx
// src/components/PresenceIndicator.tsx
import { usePresence } from '../hooks/usePresence'

export function PresenceIndicator({ memberId }: { memberId: string }) {
  const viewers = usePresence(memberId)
  if (viewers.length === 0) return null

  return (
    <div className="flex items-center gap-1 text-sm text-gray-500">
      <span className="w-2 h-2 rounded-full bg-green-400 inline-block" />
      {viewers.length === 1 ? `${viewers[0]} is also viewing` : `${viewers.length} others viewing`}
    </div>
  )
}
```

- [ ] **Step 4: Write failing test for ConflictDialog**

```tsx
// src/components/ConflictDialog.test.tsx
import { render, screen, fireEvent } from '@testing-library/react'
import { describe, it, expect, vi } from 'vitest'
import { ConflictDialog } from './ConflictDialog'
import type { Member } from '../types'

const base: Member = {
  id: '1', version: 2, first_name: 'Anna', last_name: 'Mueller',
  email: 'server@example.com', phone: null, street: null, city: null,
  postal_code: null, birth_date: null, membership_type: 'Aktiv',
  joined_at: '2024-01-01', left_at: null, notes: null,
  custom_fields: {}, created_at: '', updated_at: '',
}

describe('ConflictDialog', () => {
  it('shows conflicting fields and calls onDiscard', () => {
    const onDiscard = vi.fn()
    render(
      <ConflictDialog
        myDraft={{ ...base, email: 'mine@example.com' }}
        serverMember={base}
        onResolve={vi.fn()}
        onDiscard={onDiscard}
      />
    )
    expect(screen.getByText('mine@example.com')).toBeInTheDocument()
    expect(screen.getByText('server@example.com')).toBeInTheDocument()
    fireEvent.click(screen.getByText(/use server version/i))
    expect(onDiscard).toHaveBeenCalled()
  })
})
```

- [ ] **Step 5: Run test**

```bash
cd frontend && npx vitest run src/components/ConflictDialog.test.tsx
```

Expected: 1 test passes.

- [ ] **Step 6: Commit**

```bash
git add frontend/src/
git commit -m "feat: conflict dialog and presence indicator"
```

---

### Task 20: Settings pages (Roles, Fields, Admins)

**Files:**
- Create: `frontend/src/pages/RolesPage.tsx`
- Create: `frontend/src/pages/FieldsPage.tsx`
- Create: `frontend/src/pages/AdminsPage.tsx`

- [ ] **Step 1: Write RolesPage.tsx**

```tsx
// src/pages/RolesPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getRoles, createRole, deleteRole } from '../api/roles'

export function RolesPage() {
  const qc = useQueryClient()
  const [name, setName] = useState('')
  const { data: roles = [] } = useQuery({ queryKey: ['roles'], queryFn: getRoles })
  const add = useMutation({ mutationFn: () => createRole(name), onSuccess: () => { qc.invalidateQueries(['roles']); setName('') } })
  const remove = useMutation({ mutationFn: deleteRole, onSuccess: () => qc.invalidateQueries(['roles']) })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Roles</h1>
      <div className="flex gap-2 mb-4">
        <input className="border rounded px-3 py-2 flex-1" placeholder="New role name..." value={name} onChange={e => setName(e.target.value)} />
        <button onClick={() => add.mutate()} disabled={!name || add.isLoading} className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>
      <ul className="space-y-2">
        {roles.map(r => (
          <li key={r.id} className="flex justify-between items-center border rounded px-3 py-2">
            <span>{r.name}</span>
            <button onClick={() => remove.mutate(r.id)} className="text-red-600 hover:text-red-800 text-sm">Remove</button>
          </li>
        ))}
      </ul>
    </div>
  )
}
```

- [ ] **Step 2: Write FieldsPage.tsx**

```tsx
// src/pages/FieldsPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getFieldDefinitions, createFieldDefinition, deleteFieldDefinition } from '../api/fieldDefinitions'
import type { FieldType } from '../types'

export function FieldsPage() {
  const qc = useQueryClient()
  const [form, setForm] = useState({ name: '', field_type: 'text' as FieldType, required: false })
  const { data: fields = [] } = useQuery({ queryKey: ['field-definitions'], queryFn: getFieldDefinitions })
  const add = useMutation({ mutationFn: () => createFieldDefinition(form), onSuccess: () => { qc.invalidateQueries(['field-definitions']); setForm({ name: '', field_type: 'text', required: false }) } })
  const remove = useMutation({ mutationFn: deleteFieldDefinition, onSuccess: () => qc.invalidateQueries(['field-definitions']) })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Custom Fields</h1>
      <div className="flex gap-2 mb-4">
        <input className="border rounded px-3 py-2 flex-1" placeholder="Field name..." value={form.name} onChange={e => setForm(f => ({ ...f, name: e.target.value }))} />
        <select className="border rounded px-3 py-2" value={form.field_type} onChange={e => setForm(f => ({ ...f, field_type: e.target.value as FieldType }))}>
          <option value="text">Text</option>
          <option value="number">Number</option>
          <option value="date">Date</option>
          <option value="boolean">Boolean</option>
        </select>
        <label className="flex items-center gap-1 text-sm">
          <input type="checkbox" checked={form.required} onChange={e => setForm(f => ({ ...f, required: e.target.checked }))} /> Required
        </label>
        <button onClick={() => add.mutate()} disabled={!form.name || add.isLoading} className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add</button>
      </div>
      <ul className="space-y-2">
        {fields.map(f => (
          <li key={f.id} className="flex justify-between items-center border rounded px-3 py-2">
            <span>{f.name} <span className="text-gray-500 text-sm">({f.field_type}{f.required ? ', required' : ''})</span></span>
            <button onClick={() => remove.mutate(f.id)} className="text-red-600 hover:text-red-800 text-sm">Remove</button>
          </li>
        ))}
      </ul>
    </div>
  )
}
```

- [ ] **Step 3: Write AdminsPage.tsx**

```tsx
// src/pages/AdminsPage.tsx
import { useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { getAdmins, createAdmin, deleteAdmin } from '../api/admins'
import { useAuth } from '../hooks/useAuth'
import type { AdminRole } from '../types'

export function AdminsPage() {
  const { auth } = useAuth()
  const qc = useQueryClient()
  const [form, setForm] = useState({ username: '', password: '', role: 'Admin' as AdminRole })
  const { data: admins = [] } = useQuery({ queryKey: ['admins'], queryFn: getAdmins })
  const add = useMutation({ mutationFn: () => createAdmin(form), onSuccess: () => { qc.invalidateQueries(['admins']); setForm({ username: '', password: '', role: 'Admin' }) } })
  const remove = useMutation({ mutationFn: deleteAdmin, onSuccess: () => qc.invalidateQueries(['admins']) })

  return (
    <div className="p-6 max-w-lg mx-auto">
      <h1 className="text-2xl font-bold mb-4">Admin Users</h1>
      <div className="grid grid-cols-2 gap-2 mb-4">
        <input className="border rounded px-3 py-2" placeholder="Username" value={form.username} onChange={e => setForm(f => ({ ...f, username: e.target.value }))} />
        <input type="password" className="border rounded px-3 py-2" placeholder="Password" value={form.password} onChange={e => setForm(f => ({ ...f, password: e.target.value }))} />
        <select className="border rounded px-3 py-2" value={form.role} onChange={e => setForm(f => ({ ...f, role: e.target.value as AdminRole }))}>
          <option value="Admin">Admin</option>
          <option value="SuperAdmin">SuperAdmin</option>
        </select>
        <button onClick={() => add.mutate()} disabled={!form.username || !form.password || add.isLoading}
          className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 disabled:opacity-50">Add Admin</button>
      </div>
      <ul className="space-y-2">
        {admins.map(a => (
          <li key={a.id} className="flex justify-between items-center border rounded px-3 py-2">
            <div>
              <span className="font-medium">{a.username}</span>
              <span className="ml-2 text-sm text-gray-500">{a.role}</span>
              {a.id === auth?.admin_id && <span className="ml-2 text-xs text-blue-500">(you)</span>}
            </div>
            {a.id !== auth?.admin_id && (
              <button onClick={() => { if (confirm(`Remove ${a.username}?`)) remove.mutate(a.id) }}
                className="text-red-600 hover:text-red-800 text-sm">Remove</button>
            )}
          </li>
        ))}
      </ul>
    </div>
  )
}
```

- [ ] **Step 4: Commit**

```bash
git add frontend/src/pages/
git commit -m "feat: roles, fields, and admin management pages"
```

---

### Task 21: End-to-end smoke test

- [ ] **Step 1: Start backend**

```bash
cd backend && cargo run &
```

- [ ] **Step 2: Start frontend**

```bash
cd frontend && npm run dev &
```

- [ ] **Step 3: Open browser and verify**

Navigate to `http://localhost:5173`.

Verify the golden path:
1. Login with seeded admin credentials → redirects to /members
2. Create a new member → appears in list
3. Open member → edit a field → save → version increments
4. Open same member in second browser tab → edit a different field → save → first tab gets 409 conflict dialog when saving
5. Resolve conflict → member updated
6. Go to Settings → Roles → add a role → appears in list
7. Go to Settings → Fields → add a custom field → appears in member form
8. Go to Settings → Admins (SuperAdmin only) → add an Admin user

- [ ] **Step 4: Run all frontend tests**

```bash
cd frontend && npx vitest run
```

Expected: all tests pass.

- [ ] **Step 5: Final commit**

```bash
git add .
git commit -m "feat: member management system complete"
```
