ALTER TABLE users
    ADD COLUMN IF NOT EXISTS username text;

UPDATE users
SET username = concat(
    lower(regexp_replace(split_part(email, '@', 1), '[^a-z0-9_-]+', '-', 'g')),
    '-',
    left(replace(id::text, '-', ''), 8)
)
WHERE username IS NULL OR btrim(username) = '';

UPDATE users
SET username = concat('user-', replace(id::text, '-', ''))
WHERE username IS NULL OR btrim(username) = '' OR username = '-';

ALTER TABLE users
    ALTER COLUMN username SET NOT NULL;

DO $$
BEGIN
    ALTER TABLE users
        ADD CONSTRAINT users_username_not_blank CHECK (btrim(username) <> '');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE UNIQUE INDEX IF NOT EXISTS users_username_idx ON users (lower(username));

CREATE TABLE IF NOT EXISTS auth_sessions (
    id uuid PRIMARY KEY,
    user_id uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    access_token_hash text NOT NULL,
    refresh_token_hash text NOT NULL,
    expires_at timestamptz NOT NULL,
    refresh_expires_at timestamptz NOT NULL,
    revoked_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    last_used_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT auth_sessions_access_hash_not_blank CHECK (btrim(access_token_hash) <> ''),
    CONSTRAINT auth_sessions_refresh_hash_not_blank CHECK (btrim(refresh_token_hash) <> ''),
    CONSTRAINT auth_sessions_expiry_check CHECK (refresh_expires_at > expires_at)
);

CREATE UNIQUE INDEX IF NOT EXISTS auth_sessions_access_hash_idx ON auth_sessions (access_token_hash);
CREATE UNIQUE INDEX IF NOT EXISTS auth_sessions_refresh_hash_idx ON auth_sessions (refresh_token_hash);
CREATE INDEX IF NOT EXISTS auth_sessions_user_active_idx
    ON auth_sessions (user_id, expires_at DESC)
    WHERE revoked_at IS NULL;
