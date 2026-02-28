-- ============================================================================
-- Migration: Criar tabelas CATSER (Catálogo de Serviços do Governo Federal)
-- Hierarquia: Grupo → Classe → Serviço
-- ============================================================================

-- ============================================================================
-- 1. CATSER GROUPS (Grupos de Serviço)
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

-- ============================================================================
-- 2. CATSER CLASSES (Classes de Serviço)
-- ============================================================================

CREATE TABLE catser_classes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES catser_groups(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    budget_classification_id UUID REFERENCES budget_classifications(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_classes_group ON catser_classes(group_id);
CREATE INDEX idx_catser_classes_code ON catser_classes(code);
CREATE INDEX idx_catser_classes_name ON catser_classes(name);
CREATE INDEX idx_catser_classes_budget ON catser_classes(budget_classification_id)
    WHERE budget_classification_id IS NOT NULL;
CREATE INDEX idx_catser_classes_active ON catser_classes(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_classes
BEFORE UPDATE ON catser_classes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 3. CATSER ITEMS (Serviços)
-- ============================================================================

CREATE TABLE catser_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catser_classes(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,

    -- Identificação CATSER
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    supplementary_description TEXT,

    -- Dados operacionais do órgão
    specification TEXT, -- Detalhe local para TRs
    estimated_value DECIMAL(15, 2) NOT NULL DEFAULT 0 CHECK (estimated_value >= 0),
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
