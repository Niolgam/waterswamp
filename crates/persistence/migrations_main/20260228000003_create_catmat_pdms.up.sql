-- ============================================================================
-- Migration: Criar tabela catmat_pdms (Padrão Descritivo de Material)
-- ============================================================================

CREATE TABLE catmat_pdms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catmat_classes(id) ON DELETE RESTRICT,
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_pdms_class ON catmat_pdms(class_id);
CREATE INDEX idx_catmat_pdms_code ON catmat_pdms(code);
CREATE INDEX idx_catmat_pdms_active ON catmat_pdms(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catmat_pdms_search ON catmat_pdms USING gin (description gin_trgm_ops);

CREATE TRIGGER set_timestamp_catmat_pdms
BEFORE UPDATE ON catmat_pdms
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
