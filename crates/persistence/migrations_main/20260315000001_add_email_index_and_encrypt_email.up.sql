-- Add email_index column for HMAC blind-index lookups on encrypted email.
-- The application layer will:
--   1. Encrypt the email column value with AES-256-GCM (WS_FIELD_ENCRYPTION_KEY).
--   2. Store HMAC-SHA256(lowercase(email), key) in email_index for exact lookups.
-- Existing rows retain plaintext email until a backfill job is run.
-- All new writes go through the encrypted path immediately.

ALTER TABLE users ADD COLUMN IF NOT EXISTS email_index TEXT;

-- Partial index: only non-null values need to be unique.
CREATE UNIQUE INDEX IF NOT EXISTS users_email_index_unique
    ON users (email_index)
    WHERE email_index IS NOT NULL;

COMMENT ON COLUMN users.email_index IS
    'HMAC-SHA256 blind index of lower(email). Used for encrypted-email lookups.';
