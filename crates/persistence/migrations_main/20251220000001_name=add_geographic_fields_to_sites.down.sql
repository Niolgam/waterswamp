-- Remove geographic and map-related fields from sites table

-- Drop constraint first
ALTER TABLE sites DROP CONSTRAINT IF EXISTS unique_site_code;

-- Drop index
DROP INDEX IF EXISTS idx_sites_code;

-- Remove columns
ALTER TABLE sites DROP COLUMN IF EXISTS default_zoom;
ALTER TABLE sites DROP COLUMN IF EXISTS center;
ALTER TABLE sites DROP COLUMN IF EXISTS bounds;
ALTER TABLE sites DROP COLUMN IF EXISTS code;
