DROP TABLE IF EXISTS email_verification_tokens;
DROP INDEX IF EXISTS idx_users_email_verified;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified_at;
ALTER TABLE users DROP COLUMN IF EXISTS email_verified;
