-- ============================================================================
-- Migration: Criar tabela catser_groups (Grupos de Serviço)
-- Hierarquia CATSER: Grupo → Classe → Item
-- ============================================================================

CREATE TABLE catser_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_groups_code ON catser_groups(code);
CREATE INDEX idx_catser_groups_name ON catser_groups(name);
CREATE INDEX idx_catser_groups_active ON catser_groups(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_groups
BEFORE UPDATE ON catser_groups
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
