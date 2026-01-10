-- ============================================================================
-- SIGALM - Rollback: suppliers
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_suppliers ON suppliers;
DROP TABLE IF EXISTS suppliers CASCADE;
DROP TYPE IF EXISTS supplier_type_enum;
