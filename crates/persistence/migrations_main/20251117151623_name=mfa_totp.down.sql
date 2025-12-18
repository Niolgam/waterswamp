DROP TABLE IF EXISTS mfa_backup_code_usage;
DROP TABLE IF EXISTS mfa_setup_tokens;

DROP INDEX IF EXISTS idx_users_mfa_enabled;
ALTER TABLE users DROP COLUMN IF EXISTS mfa_backup_codes;
ALTER TABLE users DROP COLUMN IF EXISTS mfa_secret;
ALTER TABLE users DROP COLUMN IF EXISTS mfa_enabled;
