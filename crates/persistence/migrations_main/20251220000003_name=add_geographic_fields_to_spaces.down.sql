-- Remove geographic and map-related fields from spaces table

-- Drop check constraint
ALTER TABLE spaces DROP CONSTRAINT IF EXISTS check_location_type;

-- Drop unique index
DROP INDEX IF EXISTS unique_space_code_per_floor;

-- Drop indexes
DROP INDEX IF EXISTS idx_spaces_location_type;
DROP INDEX IF EXISTS idx_spaces_code;

-- Remove columns
ALTER TABLE spaces DROP COLUMN IF EXISTS area;
ALTER TABLE spaces DROP COLUMN IF EXISTS capacity;
ALTER TABLE spaces DROP COLUMN IF EXISTS coordinates;
ALTER TABLE spaces DROP COLUMN IF EXISTS location_type;
ALTER TABLE spaces DROP COLUMN IF EXISTS code;
