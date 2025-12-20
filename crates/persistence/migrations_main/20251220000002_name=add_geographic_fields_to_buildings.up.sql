-- Add geographic and map-related fields to buildings table
-- These fields are required for map display functionality

-- Add code field for building identification
ALTER TABLE buildings ADD COLUMN IF NOT EXISTS code VARCHAR(50);

-- Add total floors count
ALTER TABLE buildings ADD COLUMN IF NOT EXISTS total_floors INTEGER;

-- Add polygon coordinates for building footprint
-- Format: [[lng1, lat1], [lng2, lat2], [lng3, lat3], [lng4, lat4], [lng1, lat1]]
-- The first and last points must be the same to close the polygon
ALTER TABLE buildings ADD COLUMN IF NOT EXISTS coordinates JSONB;

-- Create index for code searches
CREATE INDEX IF NOT EXISTS idx_buildings_code ON buildings(code);

-- Add constraint to ensure code is unique when not null within a site
CREATE UNIQUE INDEX IF NOT EXISTS unique_building_code_per_site
    ON buildings(site_id, code)
    WHERE code IS NOT NULL;
