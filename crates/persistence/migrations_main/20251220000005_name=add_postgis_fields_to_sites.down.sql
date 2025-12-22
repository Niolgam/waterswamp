-- Remove PostGIS fields from sites table

-- Drop spatial indexes
DROP INDEX IF EXISTS idx_sites_bounds_gist;
DROP INDEX IF EXISTS idx_sites_center_gist;

-- Drop geometry columns
ALTER TABLE sites DROP COLUMN IF EXISTS bounds;
ALTER TABLE sites DROP COLUMN IF EXISTS center;
