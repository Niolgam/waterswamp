-- Add PostGIS geographic fields to buildings table
-- SRID 4326 = WGS 84 (standard GPS/web maps coordinate system)

-- Remove old JSONB field if it exists
ALTER TABLE buildings DROP COLUMN IF EXISTS coordinates;

-- Add PostGIS geometry field for building footprint
-- Buildings are represented as polygons showing their footprint on the map
ALTER TABLE buildings ADD COLUMN IF NOT EXISTS coordinates GEOMETRY(POLYGON, 4326);

-- Create spatial index for efficient geographic queries
CREATE INDEX IF NOT EXISTS idx_buildings_coordinates_gist ON buildings USING GIST(coordinates);

-- Add comment for documentation
COMMENT ON COLUMN buildings.coordinates IS 'Geographic footprint polygon of the building (SRID 4326 - WGS 84)';
