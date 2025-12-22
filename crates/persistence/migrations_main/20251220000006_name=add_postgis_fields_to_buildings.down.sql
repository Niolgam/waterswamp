-- Remove PostGIS fields from buildings table

-- Drop spatial index
DROP INDEX IF EXISTS idx_buildings_coordinates_gist;

-- Drop geometry column
ALTER TABLE buildings DROP COLUMN IF EXISTS coordinates;
