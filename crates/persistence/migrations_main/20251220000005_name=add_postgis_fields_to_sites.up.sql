-- Add PostGIS geographic fields to sites table
-- SRID 4326 = WGS 84 (standard GPS/web maps coordinate system)

-- Remove old JSONB fields if they exist
ALTER TABLE sites DROP COLUMN IF EXISTS bounds;
ALTER TABLE sites DROP COLUMN IF EXISTS center;

-- Add PostGIS geometry fields
-- center: single point for map centering
ALTER TABLE sites ADD COLUMN IF NOT EXISTS center GEOMETRY(POINT, 4326);

-- bounds: polygon representing the bounding box of the site
-- Note: bounds is stored as a rectangular polygon with 5 points (closing the rectangle)
ALTER TABLE sites ADD COLUMN IF NOT EXISTS bounds GEOMETRY(POLYGON, 4326);

-- Create spatial indexes for efficient geographic queries
CREATE INDEX IF NOT EXISTS idx_sites_center_gist ON sites USING GIST(center);
CREATE INDEX IF NOT EXISTS idx_sites_bounds_gist ON sites USING GIST(bounds);

-- Add comment for documentation
COMMENT ON COLUMN sites.center IS 'Geographic center point of the site (SRID 4326 - WGS 84)';
COMMENT ON COLUMN sites.bounds IS 'Geographic bounding box of the site as a polygon (SRID 4326 - WGS 84)';
