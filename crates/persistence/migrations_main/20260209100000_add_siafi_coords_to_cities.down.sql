DROP INDEX IF EXISTS idx_cities_location;
DROP INDEX IF EXISTS idx_cities_siafi_code;
ALTER TABLE cities
    DROP COLUMN IF EXISTS location,
    DROP COLUMN IF EXISTS siafi_code;
