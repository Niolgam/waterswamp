-- ============================================================================
-- Rollback: Drop Sessions Table and Related Objects
-- ============================================================================

-- Drop functions first
DROP FUNCTION IF EXISTS fn_touch_session(VARCHAR, INTEGER);
DROP FUNCTION IF EXISTS fn_revoke_user_sessions(UUID, VARCHAR);
DROP FUNCTION IF EXISTS fn_cleanup_expired_sessions();

-- Drop indexes (will be dropped with tables, but explicit for clarity)
DROP INDEX IF EXISTS idx_session_keys_active;
DROP INDEX IF EXISTS idx_sessions_ip;
DROP INDEX IF EXISTS idx_sessions_expires_at;
DROP INDEX IF EXISTS idx_sessions_user_active;
DROP INDEX IF EXISTS idx_sessions_token_hash;

-- Drop tables
DROP TABLE IF EXISTS sessions;
DROP TABLE IF EXISTS session_keys;
