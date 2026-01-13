-- ============================================================================
-- Migration: Drop Department Sites Table
-- ============================================================================

DROP TRIGGER IF EXISTS trg_department_sites_single_primary ON department_sites;
DROP FUNCTION IF EXISTS fn_department_sites_ensure_single_primary();
DROP TABLE IF EXISTS department_sites;
