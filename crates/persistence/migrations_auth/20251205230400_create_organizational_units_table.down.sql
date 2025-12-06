DROP TRIGGER IF EXISTS set_timestamp_organizational_units ON organizational_units;
DROP INDEX IF EXISTS idx_organizational_units_is_uorg;
DROP INDEX IF EXISTS idx_organizational_units_campus_id;
DROP INDEX IF EXISTS idx_organizational_units_parent_id;
DROP INDEX IF EXISTS idx_organizational_units_category_id;
DROP INDEX IF EXISTS idx_organizational_units_acronym;
DROP INDEX IF EXISTS idx_organizational_units_name;
DROP TABLE IF EXISTS organizational_units CASCADE;
