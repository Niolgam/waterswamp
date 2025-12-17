-- Add up migration script here
-- Create space_types table (lookup/reference table)

CREATE TABLE IF NOT EXISTS space_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Space type name (e.g., "Sala de Reunião", "Escritório", "Almoxarifado")
    name VARCHAR(100) UNIQUE NOT NULL,

    -- Optional description
    description TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for better query performance
CREATE INDEX idx_space_types_name ON space_types(name);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER update_space_types_updated_at
BEFORE UPDATE ON space_types
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();
