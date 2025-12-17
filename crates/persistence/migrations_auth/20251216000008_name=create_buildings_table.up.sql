-- Phase 3B: Buildings table
-- Buildings belong to a Site and have a Building Type

CREATE TABLE IF NOT EXISTS buildings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    site_id UUID NOT NULL REFERENCES sites(id) ON DELETE RESTRICT,
    building_type_id UUID NOT NULL REFERENCES building_types(id) ON DELETE RESTRICT,
    description VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: building name must be unique within a site
    CONSTRAINT unique_building_name_per_site UNIQUE (site_id, name)
);

-- Indexes for efficient queries
CREATE INDEX idx_buildings_site_id ON buildings(site_id);
CREATE INDEX idx_buildings_building_type_id ON buildings(building_type_id);
CREATE INDEX idx_buildings_name ON buildings(name);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER set_buildings_updated_at
    BEFORE UPDATE ON buildings
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_timestamp();
