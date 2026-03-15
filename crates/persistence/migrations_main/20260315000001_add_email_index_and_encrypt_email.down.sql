DROP INDEX IF EXISTS users_email_index_unique;
ALTER TABLE users DROP COLUMN IF EXISTS email_index;
