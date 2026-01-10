-- ============================================================================
-- SIGALM - Rollback: warehouse_stocks
-- ============================================================================

DROP FUNCTION IF EXISTS fn_get_available_quantity(UUID, UUID);
DROP TRIGGER IF EXISTS set_timestamp_warehouse_stocks ON warehouse_stocks;
DROP TABLE IF EXISTS warehouse_stocks CASCADE;
