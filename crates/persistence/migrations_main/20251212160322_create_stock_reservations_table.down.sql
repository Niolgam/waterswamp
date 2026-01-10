-- ============================================================================
-- SIGALM - Rollback: stock_reservations
-- ============================================================================

DROP TRIGGER IF EXISTS trg_handle_requisition_reservation ON requisitions;
DROP FUNCTION IF EXISTS fn_manage_stock_reservation();
DROP TABLE IF EXISTS stock_reservations CASCADE;
