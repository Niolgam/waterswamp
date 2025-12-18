-- Add down migration script here
DROP TRIGGER IF EXISTS set_timestamp_department_categories ON department_categories;
DROP INDEX IF EXISTS idx_department_categories_name;
DROP TABLE IF EXISTS department_categories;
