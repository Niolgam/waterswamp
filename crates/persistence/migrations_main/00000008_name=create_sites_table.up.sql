-- Create sites table (Phase 3A)
-- Sites represent physical locations (offices, warehouses, stores, etc.)
-- They belong to a city and have a site type classification

CREATE TABLE IF NOT EXISTS sites (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    city_id UUID NOT NULL REFERENCES cities(id) ON DELETE RESTRICT,
    site_type_id UUID NOT NULL REFERENCES site_types(id) ON DELETE RESTRICT,
    address VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Ensure site names are unique within a city
    CONSTRAINT unique_site_name_per_city UNIQUE (city_id, name)
);

-- Index for foreign key lookups
CREATE INDEX idx_sites_city_id ON sites(city_id);
CREATE INDEX idx_sites_site_type_id ON sites(site_type_id);

-- Index for searching by name
CREATE INDEX idx_sites_name ON sites(name);

-- Trigger to automatically update updated_at timestamp
CREATE TRIGGER set_sites_updated_at
    BEFORE UPDATE ON sites
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
