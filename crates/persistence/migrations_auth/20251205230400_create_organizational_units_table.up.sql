-- Create organizational_units table
CREATE TABLE organizational_units (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name TEXT NOT NULL,
    acronym VARCHAR(30),
    category_id UUID NOT NULL REFERENCES unit_categories(id) ON DELETE RESTRICT,
    parent_id UUID REFERENCES organizational_units(id) ON DELETE SET NULL,
    description TEXT,
    is_uorg BOOLEAN NOT NULL DEFAULT FALSE,
    campus_id UUID REFERENCES campuses(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_organizational_units_name ON organizational_units(name);
CREATE INDEX idx_organizational_units_acronym ON organizational_units(acronym);
CREATE INDEX idx_organizational_units_category_id ON organizational_units(category_id);
CREATE INDEX idx_organizational_units_parent_id ON organizational_units(parent_id);
CREATE INDEX idx_organizational_units_campus_id ON organizational_units(campus_id);
CREATE INDEX idx_organizational_units_is_uorg ON organizational_units(is_uorg);

-- Create trigger for updated_at
CREATE TRIGGER set_timestamp_organizational_units
BEFORE UPDATE ON organizational_units
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
