-- Rollback Phase 3C: Floors table

-- Drop trigger first
DROP TRIGGER IF EXISTS set_floors_updated_at ON floors;

-- Drop indexes
DROP INDEX IF EXISTS idx_floors_floor_number;
DROP INDEX IF EXISTS idx_floors_building_id;

-- Drop table
DROP TABLE IF EXISTS floors;
