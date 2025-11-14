-- Add down migration script here
DROP INDEX IF EXISTS idx_users_email_lower_unique;
ALTER TABLE users DROP CONSTRAINT IF EXISTS users_email_unique;
ALTER TABLE users DROP COLUMN IF EXISTS email;
