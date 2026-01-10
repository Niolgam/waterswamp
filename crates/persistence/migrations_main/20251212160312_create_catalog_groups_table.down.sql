-- ============================================================================
-- SIGALM - Rollback: catalog_groups
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_catalog_groups ON catalog_groups;
DROP TABLE IF EXISTS catalog_groups CASCADE;
DROP TYPE IF EXISTS item_type_enum;
