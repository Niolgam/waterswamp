-- ============================================================================
-- Migration: Criar tabela catser_secoes (Seções de Serviço)
-- Hierarquia CATSER: Seção → Divisão → Grupo → Classe → Item
-- ============================================================================

CREATE TABLE catser_secoes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_secoes_name ON catser_secoes(name);
CREATE INDEX idx_catser_secoes_active ON catser_secoes(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_secoes
BEFORE UPDATE ON catser_secoes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
