-- Add geographic and map-related fields to spaces table
-- These fields are required for map display functionality

-- Add code field for space identification
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS code VARCHAR(50);

-- Add location type (point or polygon)
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS location_type VARCHAR(20) DEFAULT 'point';

-- Add coordinates (can be a point [lng, lat] or polygon [[lng, lat], ...])
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS coordinates JSONB;

-- Add capacity (number of people/workstations)
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS capacity INTEGER;

-- Add area in square meters
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS area DECIMAL(10,2);

-- Create index for code searches
CREATE INDEX IF NOT EXISTS idx_spaces_code ON spaces(code);

-- Create index for location type
CREATE INDEX IF NOT EXISTS idx_spaces_location_type ON spaces(location_type);

-- Add constraint to ensure code is unique when not null within a floor
CREATE UNIQUE INDEX IF NOT EXISTS unique_space_code_per_floor
    ON spaces(floor_id, code)
    WHERE code IS NOT NULL;

-- Add check constraint for location_type
ALTER TABLE spaces ADD CONSTRAINT check_location_type
    CHECK (location_type IN ('point', 'polygon'));
