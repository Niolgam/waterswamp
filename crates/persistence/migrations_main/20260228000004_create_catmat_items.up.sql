-- ============================================================================
-- Migration: Criar tabela catmat_items (Itens de Material)
-- ============================================================================

CREATE TABLE catmat_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pdm_id UUID NOT NULL REFERENCES catmat_pdms(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    is_sustainable BOOLEAN NOT NULL DEFAULT FALSE,
    code_ncm VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_items_pdm ON catmat_items(pdm_id);
CREATE INDEX idx_catmat_items_unit ON catmat_items(unit_of_measure_id);
CREATE INDEX idx_catmat_items_code ON catmat_items(code);
CREATE INDEX idx_catmat_items_active ON catmat_items(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catmat_items_sustainable ON catmat_items(is_sustainable) WHERE is_sustainable = TRUE;
CREATE INDEX idx_catmat_items_ncm ON catmat_items(code_ncm) WHERE code_ncm IS NOT NULL;
CREATE INDEX idx_catmat_items_search ON catmat_items USING gin (description gin_trgm_ops);

CREATE TRIGGER set_timestamp_catmat_items
BEFORE UPDATE ON catmat_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
