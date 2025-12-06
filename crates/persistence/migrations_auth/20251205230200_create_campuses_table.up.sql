-- Create campuses table
CREATE TABLE campuses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(200) UNIQUE NOT NULL,
    acronym VARCHAR(10) UNIQUE NOT NULL,
    city_id UUID NOT NULL REFERENCES cities(id) ON DELETE RESTRICT,
    coordinates TEXT NOT NULL,
    address VARCHAR(500) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes for faster lookups
CREATE INDEX idx_campuses_acronym ON campuses(acronym);
CREATE INDEX idx_campuses_city_id ON campuses(city_id);
CREATE INDEX idx_campuses_name ON campuses(name);

-- Create trigger for updated_at
CREATE TRIGGER set_timestamp_campuses
BEFORE UPDATE ON campuses
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
