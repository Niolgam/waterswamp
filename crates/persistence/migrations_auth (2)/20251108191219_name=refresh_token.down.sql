-- Add down migration script here

DROP TRIGGER IF EXISTS set_timestamp_refresh_tokens ON refresh_tokens;
DROP FUNCTION IF EXISTS trigger_set_timestamp_refresh_tokens();
DROP TABLE IF EXISTS refresh_tokens;
