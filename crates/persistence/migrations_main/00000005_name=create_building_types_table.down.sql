-- Add down migration script here
DROP TRIGGER IF EXISTS set_timestamp_building_types ON building_types;
DROP INDEX IF EXISTS idx_building_types_name;
DROP TABLE IF EXISTS building_types;
