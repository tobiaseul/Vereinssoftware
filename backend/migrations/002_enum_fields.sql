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
