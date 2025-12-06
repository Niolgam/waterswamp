-- Create unit_categories table
CREATE TABLE unit_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    color_hex VARCHAR(7) NOT NULL DEFAULT '#CCCCCC',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT check_color_hex CHECK (color_hex ~ '^#[0-9A-Fa-f]{6}$')
);

-- Create index on name
CREATE INDEX idx_unit_categories_name ON unit_categories(name);

-- Insert common unit categories
INSERT INTO unit_categories (name, color_hex) VALUES
    ('Conselho', '#1E40AF'),      -- Blue 800
    ('Reitoria', '#7C3AED'),       -- Purple 600
    ('Vice-Reitoria', '#8B5CF6'),  -- Purple 500
    ('Pró-Reitoria', '#2563EB'),   -- Blue 600
    ('Campus', '#059669'),         -- Green 600
    ('Faculdade', '#0D9488'),      -- Teal 600
    ('Instituto', '#0891B2'),      -- Cyan 600
    ('Departamento', '#0284C7'),   -- Sky 600
    ('Secretaria', '#6366F1'),     -- Indigo 500
    ('Biblioteca', '#8B5CF6'),     -- Violet 500
    ('Hospital', '#DC2626'),       -- Red 600
    ('Editora', '#EA580C'),        -- Orange 600
    ('Procuradoria', '#CA8A04'),   -- Yellow 600
    ('Ouvidoria', '#65A30D'),      -- Lime 600
    ('Auditoria', '#16A34A'),      -- Green 500
    ('Corregedoria', '#047857');   -- Emerald 600
