-- ============================================================================
-- SIGALM - Rollback: invoice_items
-- ============================================================================

DROP TRIGGER IF EXISTS trg_update_invoice_totals ON invoice_items;
DROP FUNCTION IF EXISTS fn_update_invoice_totals();
DROP TABLE IF EXISTS invoice_items CASCADE;
