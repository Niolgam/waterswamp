-- ============================================================================
-- SIGALM - Rollback: warehouses
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_warehouses ON warehouses;
DROP TABLE IF EXISTS warehouses CASCADE;
DROP TYPE IF EXISTS warehouse_type_enum;
