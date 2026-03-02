-- ============================================================================
-- Migration: Criar tabela catser_divisoes (Divisões de Serviço)
-- Hierarquia CATSER: Seção → Divisão → Grupo → Classe → Item
-- ============================================================================

CREATE TABLE catser_divisoes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    secao_id UUID NOT NULL REFERENCES catser_secoes(id) ON DELETE RESTRICT,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_divisoes_secao ON catser_divisoes(secao_id);
CREATE INDEX idx_catser_divisoes_name ON catser_divisoes(name);
CREATE INDEX idx_catser_divisoes_active ON catser_divisoes(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_divisoes
BEFORE UPDATE ON catser_divisoes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
