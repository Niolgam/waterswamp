-- ============================================================================
-- Migration: Criar tabela catser_items (Itens de Serviço)
-- ============================================================================

CREATE TABLE catser_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catser_classes(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    supplementary_description TEXT,
    specification TEXT,
    search_links TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_items_class ON catser_items(class_id);
CREATE INDEX idx_catser_items_unit ON catser_items(unit_of_measure_id);
CREATE INDEX idx_catser_items_code ON catser_items(code);
CREATE INDEX idx_catser_items_active ON catser_items(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catser_items_search ON catser_items
    USING gin ((description || ' ' || COALESCE(specification, '')) gin_trgm_ops);

CREATE TRIGGER set_timestamp_catser_items
BEFORE UPDATE ON catser_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
