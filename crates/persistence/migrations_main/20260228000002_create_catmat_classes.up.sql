-- ============================================================================
-- Migration: Criar tabela catmat_classes (Classes de Material)
-- ============================================================================

CREATE TABLE catmat_classes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES catmat_groups(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_classes_group ON catmat_classes(group_id);
CREATE INDEX idx_catmat_classes_code ON catmat_classes(code);
CREATE INDEX idx_catmat_classes_name ON catmat_classes(name);
CREATE INDEX idx_catmat_classes_active ON catmat_classes(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catmat_classes
BEFORE UPDATE ON catmat_classes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
