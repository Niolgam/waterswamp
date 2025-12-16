-- Add up migration script here
-- Create department_categories table (lookup/reference table)

CREATE TABLE IF NOT EXISTS department_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Department category name (e.g., "Operacional", "Administrativo", "Comercial")
    name VARCHAR(100) UNIQUE NOT NULL,

    -- Optional description
    description TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for better query performance
CREATE INDEX idx_department_categories_name ON department_categories(name);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER set_timestamp_department_categories
BEFORE UPDATE ON department_categories
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
