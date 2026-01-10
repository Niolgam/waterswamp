-- ============================================================================
-- SIGALM - Rollback: states
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_states ON states;
DROP TABLE IF EXISTS states CASCADE;
