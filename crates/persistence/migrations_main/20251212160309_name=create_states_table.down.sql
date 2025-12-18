-- Drop trigger
DROP TRIGGER IF EXISTS update_states_updated_at ON states;

-- Drop indexes
DROP INDEX IF EXISTS idx_states_country_id;
DROP INDEX IF EXISTS idx_states_name;
DROP INDEX IF EXISTS idx_states_code;

-- Drop table
DROP TABLE IF EXISTS states;
