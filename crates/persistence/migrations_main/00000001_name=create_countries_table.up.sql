-- Create countries table
CREATE TABLE IF NOT EXISTS countries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    code VARCHAR(3) NOT NULL UNIQUE, -- ISO 3166-1 alpha-3 code (BRA, USA, etc)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create index for code lookups
CREATE INDEX idx_countries_code ON countries(code);

-- Create index for name searches
CREATE INDEX idx_countries_name ON countries(name);

-- Trigger to auto-update updated_at
CREATE TRIGGER update_countries_updated_at
    BEFORE UPDATE ON countries
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Insert default country (Brazil)
INSERT INTO countries (name, code) VALUES ('Brasil', 'BRA');
