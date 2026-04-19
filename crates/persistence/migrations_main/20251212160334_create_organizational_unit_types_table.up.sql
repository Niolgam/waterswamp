-- ============================================================================
-- Migration: Create Organizational Unit Types Table
-- Description: Types of units according to SIORG nomenclature
-- ============================================================================

CREATE TABLE IF NOT EXISTS organizational_unit_types (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Identification
    code VARCHAR(100) UNIQUE NOT NULL,  -- Ex: 'unidade-administrativa'
    name VARCHAR(100) NOT NULL,         -- Ex: 'Unidade Administrativa'
    description TEXT,

    -- SIORG Integration
    siorg_code INTEGER UNIQUE,
    siorg_name VARCHAR(255),
    is_siorg_managed BOOLEAN DEFAULT FALSE,

    -- Control
    is_active BOOLEAN DEFAULT TRUE,

    -- Synchronization
    siorg_synced_at TIMESTAMPTZ,
    siorg_sync_status sync_status_enum DEFAULT 'PENDING',
    siorg_raw_data JSONB,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_unit_types_code ON organizational_unit_types(code);
CREATE INDEX idx_org_unit_types_siorg_code ON organizational_unit_types(siorg_code);
CREATE INDEX idx_org_unit_types_active ON organizational_unit_types(is_active) WHERE is_active = TRUE;

CREATE TRIGGER update_unit_types_updated_at
    BEFORE UPDATE ON organizational_unit_types
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE organizational_unit_types IS 'Tipos de unidades conforme nomenclatura SIORG';

