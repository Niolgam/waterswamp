-- Reverte a migration idempotency_keys

DROP INDEX IF EXISTS idx_idempotency_keys_expires_at;
DROP INDEX IF EXISTS idx_idempotency_keys_request_id;
DROP TABLE IF EXISTS idempotency_keys;
