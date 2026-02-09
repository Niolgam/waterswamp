-- Add SIAFI code and geographic coordinates to cities table
ALTER TABLE cities
    ADD COLUMN siafi_code INT,
    ADD COLUMN location GEOGRAPHY(POINT, 4326);

-- SIAFI code should be unique when present
CREATE UNIQUE INDEX idx_cities_siafi_code ON cities (siafi_code) WHERE siafi_code IS NOT NULL;

-- Spatial index for geographic queries
CREATE INDEX idx_cities_location ON cities USING GIST (location);

-- Seed coordinates and SIAFI codes for state capitals
UPDATE cities SET
    siafi_code = t.siafi,
    location = ST_SetSRID(ST_MakePoint(t.lon, t.lat), 4326)::geography
FROM (VALUES
    (1100205, 0007, -8.7612, -63.9004),   -- Porto Velho
    (1200401, 0139, -9.9747, -67.8100),   -- Rio Branco
    (1302603, 0255, -3.1190, -60.0217),   -- Manaus
    (1400100, 0301, 2.8195, -60.6714),    -- Boa Vista
    (1501402, 0427, -1.4558, -48.5024),   -- Belém
    (1600303, 0605, 0.0349, -51.0694),    -- Macapá
    (1721000, 9733, -10.1689, -48.3317),  -- Palmas
    (2111300, 0921, -2.5297, -44.2825),   -- São Luís
    (2211001, 1219, -5.0892, -42.8019),   -- Teresina
    (2304400, 1389, -3.7172, -38.5433),   -- Fortaleza
    (2408102, 1761, -5.7945, -35.2110),   -- Natal
    (2507507, 2051, -7.1195, -34.8450),   -- João Pessoa
    (2611606, 2531, -8.0476, -34.8770),   -- Recife
    (2704302, 2801, -9.6658, -35.7353),   -- Maceió
    (2800308, 3105, -10.9091, -37.0677),  -- Aracaju
    (2927408, 3849, -12.9714, -38.5124),  -- Salvador
    (3106200, 4123, -19.9167, -43.9345),  -- Belo Horizonte
    (3205309, 5705, -20.3155, -40.3128),  -- Vitória
    (3304557, 6001, -22.9068, -43.1729),  -- Rio de Janeiro
    (3550308, 7107, -23.5505, -46.6333),  -- São Paulo
    (4106902, 7535, -25.4284, -49.2733),  -- Curitiba
    (4205407, 8105, -27.5954, -48.5480),  -- Florianópolis
    (4314902, 8801, -30.0346, -51.2177),  -- Porto Alegre
    (5002704, 9051, -20.4697, -54.6201),  -- Campo Grande
    (5103403, 9067, -15.6014, -56.0979),  -- Cuiabá
    (5208707, 9373, -16.6869, -49.2648),  -- Goiânia
    (5300108, 9701, -15.7975, -47.8919)   -- Brasília
) AS t(ibge, siafi, lat, lon)
WHERE cities.ibge_code = t.ibge;

-- Seed coordinates for Mato Grosso cities
UPDATE cities SET
    location = ST_SetSRID(ST_MakePoint(t.lon, t.lat), 4326)::geography
FROM (VALUES
    (5108402, -15.6460, -56.1325),  -- Várzea Grande
    (5107602, -16.4673, -54.6372),  -- Rondonópolis
    (5107909, -11.8642, -55.5066),  -- Sinop
    (5107925, -12.5428, -55.7214),  -- Sorriso
    (5107958, -14.6229, -57.4988),  -- Tangará da Serra
    (5102504, -16.0736, -57.6835),  -- Cáceres
    (5105259, -13.0497, -55.9099),  -- Lucas do Rio Verde
    (5107040, -15.5601, -54.2973),  -- Primavera do Leste
    (5101803, -15.8880, -52.2564),  -- Barra do Garças
    (5100300, -9.8764, -56.0861),   -- Alta Floresta
    (5106752, -15.2264, -59.3471),  -- Pontes e Lacerda
    (5105101, -11.3784, -58.7413),  -- Juína
    (5104104, -9.9563, -54.9084)    -- Guarantã do Norte
) AS t(ibge, lat, lon)
WHERE cities.ibge_code = t.ibge;
