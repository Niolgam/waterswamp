DROP TRIGGER IF EXISTS set_timestamp_campuses ON campuses;
DROP INDEX IF EXISTS idx_campuses_name;
DROP INDEX IF EXISTS idx_campuses_city_id;
DROP INDEX IF EXISTS idx_campuses_acronym;
DROP TABLE IF EXISTS campuses CASCADE;
