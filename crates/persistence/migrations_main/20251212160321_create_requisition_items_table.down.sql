-- ============================================================================
-- SIGALM - Rollback: requisition_items
-- ============================================================================

DROP TRIGGER IF EXISTS trg_capture_requisition_item_value ON requisition_items;
DROP FUNCTION IF EXISTS fn_capture_requisition_item_value();
DROP TRIGGER IF EXISTS trg_update_requisition_total ON requisition_items;
DROP FUNCTION IF EXISTS fn_update_requisition_total();
DROP TABLE IF EXISTS requisition_items CASCADE;
