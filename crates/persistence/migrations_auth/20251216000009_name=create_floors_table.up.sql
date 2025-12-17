-- Phase 3C: Floors table
-- Floors belong to a Building and have a floor_number

CREATE TABLE IF NOT EXISTS floors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    floor_number INTEGER NOT NULL,
    building_id UUID NOT NULL REFERENCES buildings(id) ON DELETE RESTRICT,
    description VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Unique constraint: floor_number must be unique within a building
    CONSTRAINT unique_floor_number_per_building UNIQUE (building_id, floor_number)
);

-- Indexes for efficient queries
CREATE INDEX idx_floors_building_id ON floors(building_id);
CREATE INDEX idx_floors_floor_number ON floors(floor_number);

-- Trigger to auto-update updated_at timestamp
CREATE TRIGGER set_floors_updated_at
    BEFORE UPDATE ON floors
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_timestamp();
