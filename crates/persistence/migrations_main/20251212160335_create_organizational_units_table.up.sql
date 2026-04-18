-- ============================================================================
-- Migration: Create Organizational Units Table
-- Description: Hierarchical structure of organizational units with SIORG integration
-- ============================================================================

CREATE TABLE IF NOT EXISTS organizational_units (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Hierarchy
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE RESTRICT,
    parent_id UUID REFERENCES organizational_units(id) ON DELETE RESTRICT,

    -- Classification
    category_id UUID NOT NULL REFERENCES organizational_unit_categories(id) ON DELETE RESTRICT,
    unit_type_id UUID NOT NULL REFERENCES organizational_unit_types(id) ON DELETE RESTRICT,
    internal_type internal_unit_type_enum NOT NULL DEFAULT 'SECTOR',

    -- Identification
    name VARCHAR(255) NOT NULL,
    formal_name VARCHAR(500),            -- Full official name
    acronym VARCHAR(50),

    -- SIORG Integration
    siorg_code INTEGER UNIQUE,           -- Nullable: allows unofficial units
    siorg_parent_code INTEGER,           -- Parent code in SIORG (for validation)
    siorg_url TEXT,                      -- Direct link in SIORG portal
    siorg_last_version VARCHAR(50),      -- SIORG version when synced
    is_siorg_managed BOOLEAN DEFAULT FALSE,

    -- Activity Area
    activity_area activity_area_enum NOT NULL,

    -- Contact (structured JSONB)
    contact_info JSONB DEFAULT '{
        "phones": [],
        "emails": [],
        "websites": [],
        "address": null
    }'::jsonb,

    -- Computed Hierarchy
    level INTEGER NOT NULL DEFAULT 1,
    path_ids UUID[] DEFAULT ARRAY[]::UUID[],   -- Path of IDs to root
    path_names TEXT,                            -- Readable path (ex: "UFMT > PROPLAN > CGSI")

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    deactivated_at TIMESTAMPTZ,
    deactivation_reason TEXT,

    -- Synchronization
    siorg_synced_at TIMESTAMPTZ,
    siorg_sync_status sync_status_enum DEFAULT 'PENDING',
    siorg_raw_data JSONB,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_org_units_no_self_parent CHECK (parent_id IS DISTINCT FROM id)
);

-- Performance indexes
CREATE INDEX idx_org_units_organization ON organizational_units(organization_id);
CREATE INDEX idx_org_units_parent ON organizational_units(parent_id);
CREATE INDEX idx_org_units_category ON organizational_units(category_id);
CREATE INDEX idx_org_units_type ON organizational_units(unit_type_id);
CREATE INDEX idx_org_units_level ON organizational_units(level);
CREATE INDEX idx_org_units_siorg_code ON organizational_units(siorg_code);
CREATE INDEX idx_org_units_name ON organizational_units(name);
CREATE INDEX idx_org_units_acronym ON organizational_units(acronym) WHERE acronym IS NOT NULL;
CREATE INDEX idx_org_units_activity_area ON organizational_units(activity_area);
CREATE INDEX idx_org_units_active ON organizational_units(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_org_units_path ON organizational_units USING GIN(path_ids);
CREATE INDEX idx_org_units_sync_status ON organizational_units(siorg_sync_status)
    WHERE siorg_sync_status IN ('PENDING', 'CONFLICT');
CREATE INDEX idx_org_units_internal_type ON organizational_units(internal_type);

CREATE TRIGGER update_org_units_updated_at
    BEFORE UPDATE ON organizational_units
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE organizational_units IS 'Estrutura hierárquica de unidades organizacionais com integração SIORG';
COMMENT ON COLUMN organizational_units.siorg_code IS 'Código SIORG. NULL para unidades não oficiais (labs informais, grupos de trabalho)';
COMMENT ON COLUMN organizational_units.path_ids IS 'Array com IDs de todos os ancestrais para consultas hierárquicas eficientes';
COMMENT ON COLUMN organizational_units.contact_info IS 'JSON estruturado: {phones: [], emails: [], websites: [], address: null}';
