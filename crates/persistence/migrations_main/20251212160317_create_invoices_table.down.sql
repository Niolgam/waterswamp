-- ============================================================================
-- SIGALM - Rollback: invoices
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_invoices ON invoices;
DROP TABLE IF EXISTS invoices CASCADE;
DROP TYPE IF EXISTS invoice_status_enum;
