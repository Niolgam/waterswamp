-- Add down migration script here
DROP TRIGGER IF EXISTS set_timestamp_states ON states;
DROP INDEX IF EXISTS idx_states_name;
DROP INDEX IF EXISTS idx_states_code;
DROP TABLE IF EXISTS states;
