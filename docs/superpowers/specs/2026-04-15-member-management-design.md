# Member Management — Design Spec

**Date:** 2026-04-15  
**Scope:** Mitgliederverwaltung module (Phase 1 of Vereinssoftware)  
**Status:** Approved

---

## Context

Software for a mixed sports and cultural association (~100 members). Phase 1 covers member management only. Financial management (Finanzverwaltung) is a separate subsequent module.

**Constraints:**
- 3–4 concurrent admin users
- Admin-only tool (member self-service is a future extension)
- Future Flutter-based member portal will consume the same API

---

## Stack

| Layer | Technology |
|---|---|
| Backend | Rust + Axum |
| Frontend | React + TypeScript |
| Database | PostgreSQL |
| Real-time | WebSockets (presence only) |

---

## Architecture

```
React (TypeScript)  ←→  Axum (Rust)  ←→  PostgreSQL
                    ←→  WebSocket (presence notifications)
```

- All data operations go through the REST API
- WebSocket carries lightweight presence events only ("user X is viewing member Y")
- Every database row has a `version` integer for optimistic concurrency control
- Conflict (HTTP 409) returns both versions; the frontend presents a resolution dialog

---

## Data Model

```
Admin
  id, username, password_hash (argon2)
  role: enum (Admin | SuperAdmin)
  created_at

Member
  id, version, created_at, updated_at
  first_name, last_name
  email, phone
  street, city, postal_code
  birth_date
  membership_type: enum (Aktiv | Passiv | Ehrenmitglied)
  joined_at, left_at (nullable — soft delete)
  notes
  custom_fields: JSONB

FieldDefinition
  id, name
  field_type: enum (text | number | date | boolean)
  required: bool
  display_order: int

Role
  id, name

MemberRole
  member_id, role_id, assigned_at
```

**Notes:**
- `membership_type` is the member's classification; `Role` is what they do in the club (e.g. Vorstand, Kassierer). A Passiv member can hold a Role.
- Admins can freely add/remove `Role` and `FieldDefinition` records without schema changes.
- Deletion is soft — `left_at` is set rather than removing the row.

---

## Authentication

- `POST /auth/login` — username + password → JWT access token (15 min) + refresh token (httpOnly cookie, 7 days)
- `POST /auth/refresh` — issues new access token from valid refresh token
- `POST /auth/logout` — invalidates refresh token server-side
- All `/api/v1/*` routes require valid JWT
- WebSocket connections authenticate via JWT on connect
- Passwords hashed with argon2
- First SuperAdmin account created via a one-time DB seed / setup script (bootstrap)

---

## API

Auth routes (unauthenticated):

| Method | Path | Description |
|---|---|---|
| POST | `/auth/login` | Login |
| POST | `/auth/refresh` | Refresh access token |
| POST | `/auth/logout` | Logout |

Protected routes (base path `/api/v1`, JWT required):

| Method | Path | Description |
|---|---|---|
| GET | `/members` | List members (filterable by membership_type, name search) |
| POST | `/members` | Create member |
| GET | `/members/:id` | Get member |
| PUT | `/members/:id` | Update member (body must include current `version`) |
| DELETE | `/members/:id` | Soft-delete member |
| GET | `/members/export` | Export member list as CSV |
| GET/POST | `/roles` | List / create roles |
| DELETE | `/roles/:id` | Delete role |
| GET/POST | `/field-definitions` | List / create custom field definitions |
| DELETE | `/field-definitions/:id` | Delete custom field definition |
| GET/POST | `/admins` | List / create admin accounts (SuperAdmin only) |
| DELETE | `/admins/:id` | Remove admin account (SuperAdmin only) |
| PUT | `/admins/:id/password` | Change admin password (SuperAdmin only) |
| WS | `/ws` | Presence notifications |

**Conflict handling:** `PUT /members/:id` returns HTTP 409 if the submitted `version` does not match the current DB version. Response body includes both the submitted and current versions.

**Error format:**
```json
{ "code": "CONFLICT", "message": "...", "details": { ... } }
```

**Validation errors:** HTTP 422 with per-field messages.

---

## Frontend

### Routes

```
/login              — login page (redirect here if unauthenticated)
/members            — member list with search and filters
/members/new        — create member
/members/:id        — member detail / edit
/settings/roles     — manage roles
/settings/fields    — manage custom field definitions
/settings/admins    — manage admin accounts (SuperAdmin only)
```

### Key behaviors

- **Optimistic updates** — changes apply in UI immediately, roll back on error
- **Presence indicator** — avatar shown on member detail when another admin has it open
- **Conflict dialog** — on 409, shows side-by-side diff; admin picks which fields to keep
- **Export** — CSV download button on member list
- **Silent token refresh** — access token refreshed automatically before expiry

### State management

React Query for all server state (no Redux). Handles caching, background refetch, and mutation lifecycle.

---

## Error Handling

| Scenario | Behavior |
|---|---|
| Network error | Toast notification, React Query retries |
| Validation error (422) | Inline per-field error messages |
| Conflict (409) | Blocking conflict resolution dialog |
| Auth expired | Silent refresh; if refresh also fails → redirect to /login |
| Server error (5xx) | Toast notification |

---

## Testing

**Backend:**
- Integration tests using `sqlx` test transactions — each test gets a clean DB state, rolled back after
- Auth middleware tested separately with valid/invalid/expired tokens

**Frontend:**
- Vitest + React Testing Library
- Key test targets: member form, conflict dialog, auth redirect behavior

---

## Out of Scope (Phase 1)

- Member self-service portal
- Financial management (separate module)
- Email/notification system
- Multi-language support
- Fine-grained permissions beyond Admin / SuperAdmin distinction
