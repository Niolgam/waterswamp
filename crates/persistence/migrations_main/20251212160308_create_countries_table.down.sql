-- ============================================================================
-- SIGALM - Rollback: countries
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_countries ON countries;
DROP TABLE IF EXISTS countries CASCADE;
