-- Add up migration script here
-- Create cities table (depends on states)

CREATE TABLE IF NOT EXISTS cities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- City name (e.g., "São Paulo", "Rio de Janeiro")
    name VARCHAR(100) NOT NULL,

    -- Foreign key to states table
    state_id UUID NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT fk_city_state FOREIGN KEY (state_id) REFERENCES states(id) ON DELETE CASCADE
);

-- Indexes for better query performance
CREATE INDEX idx_cities_name ON cities(name);
CREATE INDEX idx_cities_state_id ON cities(state_id);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER update_cities_updated_at
BEFORE UPDATE ON cities
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();
