-- ============================================================================
-- Migration: Create Organizational Unit Categories Table
-- Description: Categories according to SIORG classification
-- ============================================================================

CREATE TABLE IF NOT EXISTS organizational_unit_categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Identification
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,

    -- SIORG Integration
    siorg_code INTEGER UNIQUE,           -- Nullable: may have local categories
    siorg_name VARCHAR(255),             -- Original name in SIORG
    is_siorg_managed BOOLEAN DEFAULT FALSE,

    -- Control
    display_order INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,

    -- Synchronization
    siorg_synced_at TIMESTAMPTZ,
    siorg_sync_status sync_status_enum DEFAULT 'PENDING',
    siorg_raw_data JSONB,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_unit_categories_name ON organizational_unit_categories(name);
CREATE INDEX idx_org_unit_categories_siorg_code ON organizational_unit_categories(siorg_code);
CREATE INDEX idx_org_unit_categories_active ON organizational_unit_categories(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_org_unit_categories_display_order ON organizational_unit_categories(display_order);

CREATE TRIGGER update_org_unit_categories_updated_at
    BEFORE UPDATE ON organizational_unit_categories
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE organizational_unit_categories IS 'Categorias de unidades conforme classificação SIORG (ex: Órgão Colegiado, Unidade Administrativa)';

-- Seed: Default SIORG categories
INSERT INTO organizational_unit_categories (name, description, is_siorg_managed, display_order) VALUES
('Órgão Colegiado', 'Conselhos e câmaras deliberativas', FALSE, 1),
('Unidade Administrativa', 'Unidades de gestão e apoio', FALSE, 2),
('Unidade Acadêmica', 'Institutos, faculdades e departamentos', FALSE, 3),
('Unidade de Pesquisa', 'Centros e grupos de pesquisa', FALSE, 4),
('Unidade de Extensão', 'Coordenações de extensão e cultura', FALSE, 5)
ON CONFLICT (name) DO NOTHING;
