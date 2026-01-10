-- ============================================================================
-- SIGALM - Rollback: catalog_items
-- ============================================================================

DROP TRIGGER IF EXISTS set_timestamp_catalog_items ON catalog_items;
DROP TABLE IF EXISTS catalog_items CASCADE;
