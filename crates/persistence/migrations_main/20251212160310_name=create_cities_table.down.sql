-- Add down migration script here
DROP TRIGGER IF EXISTS set_timestamp_cities ON cities;
DROP INDEX IF EXISTS idx_cities_state_id;
DROP INDEX IF EXISTS idx_cities_name;
DROP TABLE IF EXISTS cities;
