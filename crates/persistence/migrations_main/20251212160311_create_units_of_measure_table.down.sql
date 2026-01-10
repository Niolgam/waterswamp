-- ============================================================================
-- SIGALM - Rollback: units_of_measure
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_units_of_measure ON units_of_measure;
DROP TABLE IF EXISTS units_of_measure CASCADE;
