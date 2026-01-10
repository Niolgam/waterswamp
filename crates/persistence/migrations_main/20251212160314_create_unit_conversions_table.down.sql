-- ============================================================================
-- SIGALM - Rollback: unit_conversions
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_unit_conversions ON unit_conversions;
DROP TABLE IF EXISTS unit_conversions CASCADE;
