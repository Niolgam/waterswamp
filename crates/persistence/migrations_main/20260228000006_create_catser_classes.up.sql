-- ============================================================================
-- Migration: Criar tabela catser_classes (Classes de Serviço)
-- ============================================================================

CREATE TABLE catser_classes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES catser_groups(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_classes_group ON catser_classes(group_id);
CREATE INDEX idx_catser_classes_code ON catser_classes(code);
CREATE INDEX idx_catser_classes_name ON catser_classes(name);
CREATE INDEX idx_catser_classes_active ON catser_classes(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_classes
BEFORE UPDATE ON catser_classes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
