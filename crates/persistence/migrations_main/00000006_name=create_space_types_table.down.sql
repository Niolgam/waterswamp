-- Add down migration script here
DROP TRIGGER IF EXISTS set_timestamp_space_types ON space_types;
DROP INDEX IF EXISTS idx_space_types_name;
DROP TABLE IF EXISTS space_types;
