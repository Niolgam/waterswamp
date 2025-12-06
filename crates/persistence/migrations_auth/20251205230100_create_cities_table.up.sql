-- Create cities table
CREATE TABLE cities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    state_id UUID NOT NULL REFERENCES states(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX idx_cities_name ON cities(name);
CREATE INDEX idx_cities_state_id ON cities(state_id);
CREATE INDEX idx_cities_name_state ON cities(name, state_id);

-- Create trigger for updated_at
CREATE TRIGGER set_timestamp_cities
BEFORE UPDATE ON cities
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
