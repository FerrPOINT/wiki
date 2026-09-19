DO $$ BEGIN
    IF EXISTS (SELECT 1 FROM users WHERE central_sub IS NOT NULL) THEN
        RAISE EXCEPTION 'cannot remove central subject while central profiles exist';
    END IF;
END $$;
DROP INDEX IF EXISTS users_central_sub_idx;
DROP INDEX IF EXISTS users_legacy_email_idx;
ALTER TABLE users DROP COLUMN IF EXISTS central_sub;
CREATE UNIQUE INDEX users_email_idx ON users (lower(email));
