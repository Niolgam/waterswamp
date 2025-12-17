-- Rollback Phase 3B: Buildings table

-- Drop trigger first
DROP TRIGGER IF EXISTS set_buildings_updated_at ON buildings;

-- Drop indexes
DROP INDEX IF EXISTS idx_buildings_name;
DROP INDEX IF EXISTS idx_buildings_building_type_id;
DROP INDEX IF EXISTS idx_buildings_site_id;

-- Drop table
DROP TABLE IF EXISTS buildings;
