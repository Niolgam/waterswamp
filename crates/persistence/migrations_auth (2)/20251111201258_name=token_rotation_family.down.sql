-- Add down migration script here
DROP INDEX IF EXISTS idx_refresh_tokens_family_id;

ALTER TABLE refresh_tokens
DROP COLUMN IF EXISTS parent_token_hash,
DROP COLUMN IF EXISTS family_id;
