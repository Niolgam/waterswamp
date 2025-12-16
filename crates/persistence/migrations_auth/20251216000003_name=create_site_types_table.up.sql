-- Add up migration script here
-- Create site_types table (lookup/reference table)

CREATE TABLE IF NOT EXISTS site_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Site type name (e.g., "Matriz", "Filial", "Depósito", "Centro de Distribuição")
    name VARCHAR(100) UNIQUE NOT NULL,

    -- Optional description
    description TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for better query performance
CREATE INDEX idx_site_types_name ON site_types(name);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER set_timestamp_site_types
BEFORE UPDATE ON site_types
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
