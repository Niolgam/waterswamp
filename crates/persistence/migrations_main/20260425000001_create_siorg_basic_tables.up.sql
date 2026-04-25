-- ============================================================================
-- Migration: Create SIORG basic domain tables
-- Description: natureza_juridica, poder, esfera — seeded from SIORG API
-- ============================================================================

CREATE TABLE IF NOT EXISTS siorg_natureza_juridica (
    id          UUID    PRIMARY KEY DEFAULT uuid_generate_v4(),
    siorg_code  INTEGER UNIQUE NOT NULL,
    name        VARCHAR(255) NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_siorg_natureza_juridica_code   ON siorg_natureza_juridica(siorg_code);
CREATE INDEX idx_siorg_natureza_juridica_active ON siorg_natureza_juridica(is_active) WHERE is_active = TRUE;

CREATE TRIGGER update_siorg_natureza_juridica_updated_at
    BEFORE UPDATE ON siorg_natureza_juridica
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE siorg_natureza_juridica IS
    'Naturezas jurídicas conforme Decreto-lei 200 (ex: Autarquia, Fundação, Empresa Pública). Sincronizado via /natureza-juridica da API SIORG.';

-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS siorg_poder (
    id          UUID    PRIMARY KEY DEFAULT uuid_generate_v4(),
    siorg_code  INTEGER UNIQUE NOT NULL,
    name        VARCHAR(255) NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_siorg_poder_code   ON siorg_poder(siorg_code);
CREATE INDEX idx_siorg_poder_active ON siorg_poder(is_active) WHERE is_active = TRUE;

CREATE TRIGGER update_siorg_poder_updated_at
    BEFORE UPDATE ON siorg_poder
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE siorg_poder IS
    'Poderes constitucionais (Executivo, Legislativo, Judiciário, etc.). Sincronizado via /poder da API SIORG.';

-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS siorg_esfera (
    id          UUID    PRIMARY KEY DEFAULT uuid_generate_v4(),
    siorg_code  INTEGER UNIQUE NOT NULL,
    name        VARCHAR(255) NOT NULL,
    is_active   BOOLEAN NOT NULL DEFAULT TRUE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_siorg_esfera_code   ON siorg_esfera(siorg_code);
CREATE INDEX idx_siorg_esfera_active ON siorg_esfera(is_active) WHERE is_active = TRUE;

CREATE TRIGGER update_siorg_esfera_updated_at
    BEFORE UPDATE ON siorg_esfera
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE siorg_esfera IS
    'Esferas governamentais (Federal, Estadual, Municipal, Distrital). Sincronizado via /esfera da API SIORG.';
