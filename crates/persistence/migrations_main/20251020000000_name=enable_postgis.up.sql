-- Enable PostGIS extension for geographic/geometric data types and spatial functions
CREATE EXTENSION IF NOT EXISTS postgis;

-- Verify PostGIS version
COMMENT ON EXTENSION postgis IS 'PostGIS geometry, geography, and raster spatial types and functions';
