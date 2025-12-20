-- Add geographic and map-related fields to sites table
-- These fields are required for map display functionality

-- Add code field for site identification
ALTER TABLE sites ADD COLUMN IF NOT EXISTS code VARCHAR(50);

-- Add geographic bounds for map viewport
-- Format: {"minLng": -180, "minLat": -90, "maxLng": 180, "maxLat": 90}
ALTER TABLE sites ADD COLUMN IF NOT EXISTS bounds JSONB;

-- Add center coordinates for map centering
-- Format: {"lng": -122.4194, "lat": 37.7749}
ALTER TABLE sites ADD COLUMN IF NOT EXISTS center JSONB;

-- Add default zoom level for map display
ALTER TABLE sites ADD COLUMN IF NOT EXISTS default_zoom INTEGER DEFAULT 15;

-- Create index for code searches
CREATE INDEX IF NOT EXISTS idx_sites_code ON sites(code);

-- Add constraint to ensure code is unique when not null
ALTER TABLE sites ADD CONSTRAINT unique_site_code UNIQUE NULLS NOT DISTINCT (code);
