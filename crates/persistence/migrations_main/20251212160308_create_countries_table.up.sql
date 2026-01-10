CREATE TABLE countries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    iso2 CHAR(2) NOT NULL, -- ISO 3166-1 alpha-2 (BR, US, etc)
    bacen_code INT UNIQUE NOT NULL, -- O cPais da NF-e (Brasil é 1058)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Índices
CREATE INDEX idx_countries_iso2 ON countries(iso2);
CREATE INDEX idx_countries_name ON countries(name);
CREATE INDEX idx_countries_bacen ON countries(bacen_code);

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_countries
BEFORE UPDATE ON countries
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

INSERT INTO countries (name, iso2, bacen_code) VALUES 
-- América do Sul
('Argentina', 'AR', 639),
('Brasil', 'BR', 1058),
('Chile', 'CL', 1589),
('Uruguai', 'UY', 8451),
('Paraguai', 'PY', 5860),
('Colômbia', 'CO', 1690),
('Peru', 'PE', 5894),
('Bolívia', 'BO', 973),
-- Outros Principais
('Estados Unidos', 'US', 2496),
('China', 'CN', 1605),
-- Europa (Principais parceiros comerciais)
('Alemanha', 'DE', 230),
('França', 'FR', 2750),
('Portugal', 'PT', 6076),
('Espanha', 'ES', 2453),
('Itália', 'IT', 3867),
('Reino Unido', 'GB', 6289);
