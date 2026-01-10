-- ============================================================================
-- SIGALM - Rollback: cities
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_cities ON cities;
DROP TABLE IF EXISTS cities CASCADE;
