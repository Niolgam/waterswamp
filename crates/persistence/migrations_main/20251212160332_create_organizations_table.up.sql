-- ============================================================================
-- Migration: Create Organizations Table
-- Description: Root entity (ex: UFMT). Usually only one record.
-- ============================================================================

CREATE TABLE IF NOT EXISTS organizations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Official Identification
    acronym VARCHAR(50) NOT NULL,
    name VARCHAR(255) NOT NULL,
    cnpj CHAR(14) UNIQUE NOT NULL,

    -- Government Codes
    ug_code INTEGER NOT NULL,           -- Unidade Gestora code (SIAFI)
    siorg_code INTEGER NOT NULL UNIQUE, -- SIORG code

    -- Contact and Location
    address VARCHAR(500),
    city VARCHAR(100),
    state CHAR(2),
    zip_code VARCHAR(10),
    phone VARCHAR(50),
    email VARCHAR(255),
    website VARCHAR(255),
    logo_url TEXT,

    -- Control
    is_main BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT TRUE,

    -- SIORG Sync
    siorg_synced_at TIMESTAMPTZ,
    siorg_raw_data JSONB,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_cnpj_format CHECK (cnpj ~ '^\d{14}$'),
    CONSTRAINT chk_state_code CHECK (state IS NULL OR state ~ '^[A-Z]{2}$')
);

CREATE INDEX idx_organizations_siorg_code ON organizations(siorg_code);
CREATE INDEX idx_organizations_cnpj ON organizations(cnpj);
CREATE INDEX idx_organizations_active ON organizations(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_organizations_main ON organizations(is_main) WHERE is_main = TRUE;

CREATE TRIGGER set_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE organizations IS 'Entidade raiz do sistema (ex: UFMT). Normalmente haverá apenas um registro.';
COMMENT ON COLUMN organizations.ug_code IS 'Código da Unidade Gestora no SIAFI';
COMMENT ON COLUMN organizations.siorg_code IS 'Código do órgão no Sistema de Organização e Inovação Institucional do Governo Federal';
COMMENT ON COLUMN organizations.cnpj IS 'CNPJ sem pontuação (14 dígitos)';

-- Seed: UFMT (can be customized)
INSERT INTO organizations (
    acronym,
    name,
    cnpj,
    ug_code,
    siorg_code,
    website,
    is_main
) VALUES (
    'UFMT',
    'Fundação Universidade Federal de Mato Grosso',
    '33004540000100',
    154045,
    471,  -- Real UFMT SIORG code
    'https://www.ufmt.br',
    TRUE
) ON CONFLICT (cnpj) DO NOTHING;
