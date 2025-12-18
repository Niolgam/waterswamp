-- Create states table with country relationship
CREATE OR REPLACE FUNCTION update_states_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TABLE IF NOT EXISTS states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    code VARCHAR(2) UNIQUE NOT NULL,
    country_id UUID NOT NULL REFERENCES countries(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT check_code_format CHECK (code ~ '^[A-Z]{2}$')
);

-- Indexes for better query performance
CREATE INDEX idx_states_code ON states(code);
CREATE INDEX idx_states_name ON states(name);
CREATE INDEX idx_states_country_id ON states(country_id);

-- Trigger to automatically update the updated_at timestamp
CREATE TRIGGER update_states_updated_at
    BEFORE UPDATE ON states
    FOR EACH ROW
    EXECUTE FUNCTION update_states_updated_at_column();
