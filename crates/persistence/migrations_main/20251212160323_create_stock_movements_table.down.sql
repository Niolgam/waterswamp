-- ============================================================================
-- SIGALM - Rollback: stock_movements
-- ============================================================================

DROP FUNCTION IF EXISTS fn_create_entry_from_invoice_item(UUID, UUID);
DROP TRIGGER IF EXISTS trg_stock_movement_process ON stock_movements;
DROP FUNCTION IF EXISTS fn_process_stock_movement();
DROP TABLE IF EXISTS stock_movements CASCADE;
DROP TYPE IF EXISTS stock_movement_type_enum;
