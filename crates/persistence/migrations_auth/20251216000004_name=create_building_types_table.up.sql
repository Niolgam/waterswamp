-- Add up migration script here
-- Create building_types table (lookup/reference table)

CREATE TABLE IF NOT EXISTS building_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Building type name (e.g., "Comercial", "Industrial", "Residencial")
    name VARCHAR(100) UNIQUE NOT NULL,

    -- Optional description
    description TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for better query performance
CREATE INDEX idx_building_types_name ON building_types(name);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER set_timestamp_building_types
BEFORE UPDATE ON building_types
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
