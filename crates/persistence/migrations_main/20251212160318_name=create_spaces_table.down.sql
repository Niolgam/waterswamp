-- Drop trigger
DROP TRIGGER IF EXISTS update_spaces_updated_at ON spaces;

-- Drop indexes
DROP INDEX IF EXISTS idx_spaces_name;
DROP INDEX IF EXISTS idx_spaces_space_type_id;
DROP INDEX IF EXISTS idx_spaces_floor_id;

-- Drop table
DROP TABLE IF EXISTS spaces;
