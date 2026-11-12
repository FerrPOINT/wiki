-- Explicit native grants have a separate provenance from historical SSO shadow roles.
-- No historical unknown admin row is promoted by this migration.
CREATE TABLE wiki_confirmed_global_roles (
    user_id uuid PRIMARY KEY REFERENCES users(id) ON DELETE RESTRICT,
    role text NOT NULL CHECK(role='admin'),
    granted_by uuid NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    granted_at timestamptz NOT NULL DEFAULT now()
);
