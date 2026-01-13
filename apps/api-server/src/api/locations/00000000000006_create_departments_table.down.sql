-- ============================================================================
-- Migration: Drop Departments Table
-- ============================================================================

DROP TRIGGER IF EXISTS trg_departments_updated_at ON departments;
DROP TRIGGER IF EXISTS trg_departments_hierarchy ON departments;
DROP FUNCTION IF EXISTS fn_departments_update_hierarchy();
DROP TABLE IF EXISTS departments;
