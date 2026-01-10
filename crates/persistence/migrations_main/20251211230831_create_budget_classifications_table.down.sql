-- ============================================================================
-- SIGALM - Rollback: budget_classifications
-- ============================================================================

DROP TRIGGER IF EXISTS trg_budget_hierarchy_auto ON budget_classifications;
DROP FUNCTION IF EXISTS fn_calculate_budget_hierarchy();
DROP TRIGGER IF EXISTS set_timestamp_budget_classifications ON budget_classifications;
DROP TABLE IF EXISTS budget_classifications CASCADE;
