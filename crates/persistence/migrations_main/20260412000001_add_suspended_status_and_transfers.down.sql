-- ============================================================================
-- Rollback: DRS Features - SUSPENDED status + stock_transfers table
-- ============================================================================

DROP TABLE IF EXISTS stock_transfer_items CASCADE;
DROP TABLE IF EXISTS stock_transfers CASCADE;
DROP TYPE IF EXISTS stock_transfer_status_enum CASCADE;
DROP FUNCTION IF EXISTS fn_generate_transfer_number() CASCADE;

-- Note: PostgreSQL does not support removing enum values directly.
-- The SUSPENDED value from requisition_status_enum cannot be removed
-- without recreating the type. This is acceptable for a rollback scenario
-- where no SUSPENDED records exist yet.
