-- Remove geographic and map-related fields from buildings table

-- Drop unique index
DROP INDEX IF EXISTS unique_building_code_per_site;

-- Drop index
DROP INDEX IF EXISTS idx_buildings_code;

-- Remove columns
ALTER TABLE buildings DROP COLUMN IF EXISTS coordinates;
ALTER TABLE buildings DROP COLUMN IF EXISTS total_floors;
ALTER TABLE buildings DROP COLUMN IF EXISTS code;
