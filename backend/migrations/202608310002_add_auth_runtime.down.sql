DROP TABLE IF EXISTS auth_sessions;

DROP INDEX IF EXISTS users_username_idx;

ALTER TABLE users
    DROP CONSTRAINT IF EXISTS users_username_not_blank;

ALTER TABLE users
    DROP COLUMN IF EXISTS username;
