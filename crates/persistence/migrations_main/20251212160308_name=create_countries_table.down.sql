-- Drop trigger
DROP TRIGGER IF EXISTS update_countries_updated_at ON countries;

-- Drop indexes
DROP INDEX IF EXISTS idx_countries_name;
DROP INDEX IF EXISTS idx_countries_code;

-- Drop table
DROP TABLE IF EXISTS countries;
