-- Add down migration script here
DROP TRIGGER IF EXISTS set_timestamp_site_types ON site_types;
DROP INDEX IF EXISTS idx_site_types_name;
DROP TABLE IF EXISTS site_types;
