-- Remove PostGIS fields from spaces table

-- Drop constraint
ALTER TABLE spaces DROP CONSTRAINT IF EXISTS check_coordinates_match_location_type;

-- Drop spatial index
DROP INDEX IF EXISTS idx_spaces_coordinates_gist;

-- Drop geometry column
ALTER TABLE spaces DROP COLUMN IF EXISTS coordinates;
