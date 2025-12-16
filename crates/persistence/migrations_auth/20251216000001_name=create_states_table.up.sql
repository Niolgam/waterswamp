-- Add up migration script here
-- Create states table (root of geographic hierarchy)

CREATE TABLE IF NOT EXISTS states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- State name (e.g., "São Paulo")
    name VARCHAR(100) NOT NULL,

    -- State code - 2 uppercase letters (e.g., "SP")
    code VARCHAR(2) UNIQUE NOT NULL,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Constraints
    CONSTRAINT check_code_format CHECK (code ~ '^[A-Z]{2}$')
);

-- Indexes for better query performance
CREATE INDEX idx_states_code ON states(code);
CREATE INDEX idx_states_name ON states(name);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER set_timestamp_states
BEFORE UPDATE ON states
FOR EACH ROW
EXECUTE PROCEDURE trigger_set_timestamp();
