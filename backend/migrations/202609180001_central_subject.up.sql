ALTER TABLE users ADD COLUMN central_sub text;
DROP INDEX IF EXISTS users_email_idx;
CREATE UNIQUE INDEX users_legacy_email_idx ON users (lower(email)) WHERE central_sub IS NULL;
CREATE UNIQUE INDEX users_central_sub_idx ON users (central_sub) WHERE central_sub IS NOT NULL;
