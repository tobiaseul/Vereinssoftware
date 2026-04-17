# Enum Custom Fields — Design Spec

**Date:** 2026-04-16
**Scope:** Extend custom fields with an enum (dropdown) type; replace roles with enum fields
**Status:** Approved

---

## Context

Phase 1 shipped with custom fields of type `text | number | date | boolean`. Roles were a separate entity (own table, own page). This spec adds:

1. An `enum` field type whose allowed values are managed per-field
2. Inline editing of existing field definitions (name + required flag)
3. Clean removal of the roles concept (replaced by an enum custom field)

---

## Data Model

New migration (`002_enum_fields.sql`):

```sql
-- Extend field_type enum
ALTER TYPE field_type ADD VALUE 'enum';

-- Options for enum fields
CREATE TABLE field_definition_options (
  id                   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  field_definition_id  UUID NOT NULL REFERENCES field_definitions(id) ON DELETE CASCADE,
  value                TEXT NOT NULL,
  display_order        INTEGER NOT NULL DEFAULT 0,
  UNIQUE (field_definition_id, value)
);

-- Remove roles (no production data yet)
DROP TABLE member_roles;
DROP TABLE roles;
```

---

## API

### Field Definitions

| Method | Path | Description |
|---|---|---|
| GET | `/api/v1/field-definitions` | List all fields, each with `options: []` (populated for enum fields) |
| POST | `/api/v1/field-definitions` | Create field (type `enum` now valid) |
| PUT | `/api/v1/field-definitions/:id` | Edit name + required (type is immutable after creation) |
| DELETE | `/api/v1/field-definitions/:id` | Delete field and its options (cascade) |

### Field Options

| Method | Path | Description |
|---|---|---|
| POST | `/api/v1/field-definitions/:id/options` | Add option `{ value: string, display_order?: int }` |
| PUT | `/api/v1/field-definitions/:id/options/:option_id` | Edit option value or display_order |
| DELETE | `/api/v1/field-definitions/:id/options/:option_id` | Delete option |

### Removed

- `GET /api/v1/roles`
- `POST /api/v1/roles`
- `DELETE /api/v1/roles/:id`

### Response shape

`GET /api/v1/field-definitions` returns:

```json
[
  {
    "id": "...",
    "name": "Funktion",
    "field_type": "enum",
    "required": false,
    "display_order": 0,
    "options": [
      { "id": "...", "value": "Vorstand", "display_order": 0 },
      { "id": "...", "value": "Kassierer", "display_order": 1 }
    ]
  }
]
```

Non-enum fields always return `"options": []`.

---

## Frontend

### Types

```ts
export interface FieldOption {
  id: string
  value: string
  display_order: number
}

// FieldDefinition gains:
options: FieldOption[]
```

### FieldsPage

- Each field row has an **Edit** button → opens inline form to edit name + required (type shown read-only)
- Enum fields have an **Options** toggle → expands inline list of options with edit + delete per option, and an "Add option" input at the bottom
- Add field form includes `Enum` in the type dropdown

### MemberForm

- Custom fields with `field_type === 'enum'` render as `<select>` populated from `field.options`
- All other types unchanged

### Navigation / routing

- Remove Roles link from settings sidebar
- Delete `RolesPage.tsx`
- Route `/settings/roles` removed from router

### New API functions

```ts
// src/api/fieldDefinitions.ts additions
updateFieldDefinition(id, { name, required })
addFieldOption(fieldId, { value, display_order? })
updateFieldOption(fieldId, optionId, { value?, display_order? })
deleteFieldOption(fieldId, optionId)
```

---

## Out of Scope

- Reordering options via drag-and-drop (display_order set manually via number input if needed)
- Migrating existing roles data (none exists)
- Changing a field's type after creation
