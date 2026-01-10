CREATE TABLE states (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    country_id UUID NOT NULL REFERENCES countries(id) ON DELETE RESTRICT,
    name VARCHAR(100) NOT NULL,
    abbreviation CHAR(2) NOT NULL, -- A sigla (MT, SP, RJ)
    ibge_code INT UNIQUE NOT NULL, -- O cUF da NF-e (MT é 51, SP é 35)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_states_abbreviation_country UNIQUE (country_id, abbreviation),
    CONSTRAINT uq_states_name_country UNIQUE (country_id, name),
    CONSTRAINT ck_states_code_format CHECK (abbreviation ~ '^[A-Z]{2}$')
);

-- Índices
CREATE INDEX idx_states_ibge_code ON states(ibge_code);
CREATE INDEX idx_states_abbreviation ON states(abbreviation);
CREATE INDEX idx_states_name ON states(name);
CREATE INDEX idx_states_country_id ON states(country_id);

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_states
BEFORE UPDATE ON states
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();


INSERT INTO states (country_id, name, abbreviation, ibge_code)
SELECT c.id, t.name, t.abbreviation, t.ibge_code
FROM (VALUES 
    ('Rondônia', 'RO', 11),
    ('Acre', 'AC', 12),
    ('Amazonas', 'AM', 13),
    ('Roraima', 'RR', 14),
    ('Pará', 'PA', 15),
    ('Amapá', 'AP', 16),
    ('Tocantins', 'TO', 17),
    ('Maranhão', 'MA', 21),
    ('Piauí', 'PI', 22),
    ('Ceará', 'CE', 23),
    ('Rio Grande do Norte', 'RN', 24),
    ('Paraíba', 'PB', 25),
    ('Pernambuco', 'PE', 26),
    ('Alagoas', 'AL', 27),
    ('Sergipe', 'SE', 28),
    ('Bahia', 'BA', 29),
    ('Minas Gerais', 'MG', 31),
    ('Espírito Santo', 'ES', 32),
    ('Rio de Janeiro', 'RJ', 33),
    ('São Paulo', 'SP', 35),
    ('Paraná', 'PR', 41),
    ('Santa Catarina', 'SC', 42),
    ('Rio Grande do Sul', 'RS', 43),
    ('Mato Grosso do Sul', 'MS', 50),
    ('Mato Grosso', 'MT', 51),
    ('Goiás', 'GO', 52),
    ('Distrito Federal', 'DF', 53)
) AS t(name, abbreviation, ibge_code)
CROSS JOIN countries c
WHERE c.bacen_code = 1058;
