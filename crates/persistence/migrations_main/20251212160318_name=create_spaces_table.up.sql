-- Create spaces table with foreign keys to floors and space_types
CREATE TABLE IF NOT EXISTS spaces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    floor_id UUID NOT NULL REFERENCES floors(id) ON DELETE RESTRICT,
    space_type_id UUID NOT NULL REFERENCES space_types(id) ON DELETE RESTRICT,
    description VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_space_name_per_floor UNIQUE (floor_id, name)
);

-- Create index for foreign key lookups
CREATE INDEX idx_spaces_floor_id ON spaces(floor_id);
CREATE INDEX idx_spaces_space_type_id ON spaces(space_type_id);

-- Create index for name searches
CREATE INDEX idx_spaces_name ON spaces(name);

-- Trigger to auto-update updated_at
CREATE TRIGGER update_spaces_updated_at
    BEFORE UPDATE ON spaces
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
