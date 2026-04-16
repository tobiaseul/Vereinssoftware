# Vue 3 Frontend Rewrite — Design Spec

**Date:** 2026-04-16  
**Scope:** Replace React frontend with Vue 3 + Element Plus + AG Grid  
**Status:** Approved

---

## Context

The existing React + Tailwind frontend is replaced in full. The Rust/Axum backend, PostgreSQL database, and all API contracts are unchanged. Feature parity is the goal — no new features.

---

## Stack

| Layer | Technology |
|---|---|
| Framework | Vue 3 (Composition API, `<script setup>`) + TypeScript |
| Build | Vite (preserved from React project) |
| Routing | Vue Router 4 |
| Server state | @tanstack/vue-query |
| HTTP | Axios (same client + interceptors) |
| Components | Element Plus |
| Tables | AG Grid Community + ag-grid-vue3 |
| Auth state | Pinia |
| Testing | Vitest + @vue/test-utils |

---

## Migration approach

Delete `frontend/` entirely. Scaffold a fresh Vue 3 + Vite + TypeScript project. Carry over unchanged:

- `src/api/` — all API modules (plain TypeScript, no React dependencies)
- `src/types/index.ts` — unchanged
- `vite.config.ts` proxy config (`/api`, `/auth`, `/ws` → localhost:3000)

---

## File Structure

```
frontend/src/
  api/                      ← carried over unchanged
    client.ts
    auth.ts
    members.ts
    admins.ts
    fieldDefinitions.ts
  types/
    index.ts                ← carried over unchanged
  stores/
    auth.ts                 ← Pinia store: auth state, login, logout, silent refresh
  composables/
    usePresence.ts          ← WebSocket presence (Vue composable)
  router/
    index.ts                ← Vue Router with navigation guards
  views/
    LoginView.vue
    MembersView.vue         ← AG Grid members table
    MemberDetailView.vue    ← Element Plus form, conflict dialog, presence
    MemberNewView.vue       ← Element Plus form
    FieldsView.vue          ← inline edit + expandable enum options
    AdminsView.vue          ← admin CRUD
  components/
    MemberForm.vue          ← shared form used by Detail + New views
    ConflictDialog.vue      ← ElDialog with side-by-side diff
    PresenceIndicator.vue   ← uses usePresence composable
  App.vue                   ← ElMenu nav bar
  main.ts                   ← app bootstrap
```

---

## Auth + Routing

**Pinia `useAuthStore`** (`stores/auth.ts`):
- State: `auth: AuthState | null`, `isLoading: boolean`
- Actions: `login(username, password)`, `logout()`, `silentRefresh()` (called on app mount)
- Axios interceptors: attach Bearer token on requests; on 401, attempt refresh, retry, else redirect to `/login`

**Vue Router navigation guards** (`router/index.ts`):
```
router.beforeEach:
  - route.meta.requiresAuth && !auth  → redirect /login
  - route.meta.requiresSuperAdmin && role !== SuperAdmin  → redirect /members
```

**Routes:**
| Path | View | Guard |
|---|---|---|
| `/login` | LoginView | — |
| `/members` | MembersView | auth |
| `/members/new` | MemberNewView | auth |
| `/members/:id` | MemberDetailView | auth |
| `/settings/fields` | FieldsView | auth |
| `/settings/admins` | AdminsView | SuperAdmin |
| `*` | redirect `/members` | — |

**Nav bar** (`App.vue`): Element Plus `ElMenu` horizontal. Links: Members, Fields, Admins (SuperAdmin only), Logout button on the right.

---

## Members List (AG Grid)

`MembersView.vue` uses AG Grid Community:

**Column definitions:**
| Field | Header | Notes |
|---|---|---|
| `first_name` + `last_name` | Name | `valueGetter` combining both |
| `membership_type` | Type | — |
| `joined_at` | Joined | date format |
| `left_at` | Status | "Active" if null, "Left" if set |

**Behaviour:**
- `domLayout: 'autoHeight'`
- `rowSelection: 'single'` — click row → navigate to `/members/:id`
- Search input above grid → AG Grid quick filter
- "New Member" button → navigate to `/members/new`
- "Export CSV" button → `GET /api/v1/members/export` download

---

## Other Views

All other views use Element Plus components (`ElForm`, `ElInput`, `ElSelect`, `ElButton`, `ElDialog`, `ElTable`):

- **MemberForm.vue** — same fields as React version; enum custom fields render as `ElSelect`; used by both MemberDetailView and MemberNewView
- **ConflictDialog.vue** — `ElDialog` with a table showing only differing fields; "Keep mine" / "Use server" actions
- **PresenceIndicator.vue** — small badge showing who else is viewing the member
- **FieldsView.vue** — `ElTable` for field list; inline edit row; expandable enum options section per field
- **AdminsView.vue** — `ElTable` for admin list; form to add new admin; self-deletion prevented

---

## Optimistic Concurrency

Identical to the React implementation:
- `PUT /members/:id` includes `version`
- On 409: `ConflictDialog` opens with `myDraft` and `serverMember`
- Resolve: merge chosen fields with `serverMember.version`

---

## Testing

**Vitest + @vue/test-utils.** Two test targets matching existing React coverage:

1. `LoginView.test.ts` — submitting the login form calls `login()` with correct credentials
2. `ConflictDialog.test.ts` — renders differing fields, "Use server" calls onDiscard

---

## Out of Scope

- No backend changes
- No new features
- No migration of existing test infrastructure (old React tests deleted with the React project)
