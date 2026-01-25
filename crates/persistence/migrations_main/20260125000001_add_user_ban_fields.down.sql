-- Remove ban fields from users table
DROP INDEX IF EXISTS idx_users_is_banned;

ALTER TABLE users
    DROP COLUMN IF EXISTS banned_reason,
    DROP COLUMN IF EXISTS banned_at,
    DROP COLUMN IF EXISTS is_banned;
