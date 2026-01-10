CREATE TABLE cities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    state_id UUID NOT NULL REFERENCES states(id) ON DELETE RESTRICT,
    name VARCHAR(100) NOT NULL,
    ibge_code INT NOT NULL UNIQUE, -- Código IBGE do município
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT uq_cities_name_state UNIQUE (state_id, name),
    CONSTRAINT uq_cities_ibge_code UNIQUE (ibge_code)
);

-- Índices
CREATE INDEX idx_cities_name ON cities(name);
CREATE INDEX idx_cities_state_id ON cities(state_id);
CREATE INDEX idx_cities_ibge_code ON cities(ibge_code) WHERE ibge_code IS NOT NULL;
CREATE INDEX idx_cities_name_tgrm ON cities USING gin (name gin_trgm_ops);

-- Trigger para updated_at
CREATE TRIGGER set_timestamp_cities
BEFORE UPDATE ON cities
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

INSERT INTO cities (state_id, name, ibge_code)
SELECT s.id, t.city_name, t.city_ibge
FROM (VALUES 
    (11, 'Porto Velho', 1100205), (12, 'Rio Branco', 1200401), (13, 'Manaus', 1302603), 
    (14, 'Boa Vista', 1400100), (15, 'Belém', 1501402), (16, 'Macapá', 1600303), 
    (17, 'Palmas', 1721000), (21, 'São Luís', 2111300), (22, 'Teresina', 2211001), 
    (23, 'Fortaleza', 2304400), (24, 'Natal', 2408102), (25, 'João Pessoa', 2507507), 
    (26, 'Recife', 2611606), (27, 'Maceió', 2704302), (28, 'Aracaju', 2800308), 
    (29, 'Salvador', 2927408), (31, 'Belo Horizonte', 3106200), (32, 'Vitória', 3205309), 
    (33, 'Rio de Janeiro', 3304557), (35, 'São Paulo', 3550308), (41, 'Curitiba', 4106902), 
    (42, 'Florianópolis', 4205407), (43, 'Porto Alegre', 4314902), (50, 'Campo Grande', 5002704), 
    (51, 'Cuiabá', 5103403), (52, 'Goiânia', 5208707), (53, 'Brasília', 5300108)
) AS t(state_ibge, city_name, city_ibge)
JOIN states s ON s.ibge_code = t.state_ibge;

INSERT INTO cities (state_id, name, ibge_code)
SELECT s.id, t.city_name, t.city_ibge
FROM (VALUES 
    ('Várzea Grande', 5108402),
    ('Rondonópolis', 5107602),
    ('Sinop', 5107909),
    ('Sorriso', 5107925),
    ('Tangará da Serra', 5107958),
    ('Cáceres', 5102504),
    ('Lucas do Rio Verde', 5105259),
    ('Primavera do Leste', 5107040),
    ('Barra do Garças', 5101803),
    ('Alta Floresta', 5100300),
    ('Pontes e Lacerda', 5106752),
    ('Juína', 5105101),
    ('Guarantã do Norte', 5104104)
) AS t(city_name, city_ibge)
CROSS JOIN states s 
WHERE s.ibge_code = 51; -- Filtro específico para o ID do Mato Grosso
