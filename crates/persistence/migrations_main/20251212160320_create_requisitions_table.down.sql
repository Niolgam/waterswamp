-- ============================================================================
-- SIGALM - Rollback: requisitions
-- ============================================================================

DROP TRIGGER IF EXISTS trg_generate_requisition_number ON requisitions;
DROP FUNCTION IF EXISTS fn_generate_requisition_number();
DROP TRIGGER IF EXISTS set_timestamp_requisitions ON requisitions;
DROP TABLE IF EXISTS requisitions CASCADE;
DROP TYPE IF EXISTS requisition_priority_enum;
DROP TYPE IF EXISTS requisition_status_enum;
