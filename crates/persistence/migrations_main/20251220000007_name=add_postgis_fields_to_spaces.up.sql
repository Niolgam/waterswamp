-- Add PostGIS geographic fields to spaces table
-- SRID 4326 = WGS 84 (standard GPS/web maps coordinate system)

-- Remove old JSONB field if it exists
ALTER TABLE spaces DROP COLUMN IF EXISTS coordinates;

-- Add PostGIS geometry field for space location
-- Spaces can be either POINT (for small spaces) or POLYGON (for larger spaces)
-- We use generic GEOMETRY type to allow both, constrained by location_type
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS coordinates GEOMETRY(GEOMETRY, 4326);
ALTER TABLE spaces ADD COLUMN IF NOT EXISTS location_type VARCHAR(100);
-- Create spatial index for efficient geographic queries
CREATE INDEX IF NOT EXISTS idx_spaces_coordinates_gist ON spaces USING GIST(coordinates);

-- Add constraint to ensure geometry type matches location_type
-- When location_type is 'point', geometry must be a POINT
-- When location_type is 'polygon', geometry must be a POLYGON
ALTER TABLE spaces ADD CONSTRAINT check_coordinates_match_location_type
    CHECK (
        (location_type = 'point' AND ST_GeometryType(coordinates) = 'ST_Point') OR
        (location_type = 'polygon' AND ST_GeometryType(coordinates) = 'ST_Polygon') OR
        (coordinates IS NULL)
    );

-- Add comments for documentation
COMMENT ON COLUMN spaces.coordinates IS 'Geographic location of the space - POINT or POLYGON depending on location_type (SRID 4326 - WGS 84)';
COMMENT ON CONSTRAINT check_coordinates_match_location_type ON spaces IS 'Ensures geometry type matches location_type field';
