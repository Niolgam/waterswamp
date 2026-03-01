-- ============================================================================
-- Migration: Criar tabela catmat_groups (Grupos de Material)
-- Hierarquia CATMAT: Grupo → Classe → PDM → Item
-- ============================================================================

CREATE TABLE catmat_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_groups_code ON catmat_groups(code);
CREATE INDEX idx_catmat_groups_name ON catmat_groups(name);
CREATE INDEX idx_catmat_groups_active ON catmat_groups(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catmat_groups
BEFORE UPDATE ON catmat_groups
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
