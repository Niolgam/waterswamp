-- ============================================================================
-- Migration: Criar tabelas CATMAT (Catálogo de Materiais do Governo Federal)
-- Hierarquia: Grupo → Classe → Item (PDM)
-- ============================================================================

-- ============================================================================
-- 1. CATMAT GROUPS (Grupos de Material)
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

-- ============================================================================
-- 2. CATMAT CLASSES (Classes de Material)
-- ============================================================================

CREATE TABLE catmat_classes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES catmat_groups(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    budget_classification_id UUID REFERENCES budget_classifications(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_classes_group ON catmat_classes(group_id);
CREATE INDEX idx_catmat_classes_code ON catmat_classes(code);
CREATE INDEX idx_catmat_classes_name ON catmat_classes(name);
CREATE INDEX idx_catmat_classes_budget ON catmat_classes(budget_classification_id)
    WHERE budget_classification_id IS NOT NULL;
CREATE INDEX idx_catmat_classes_active ON catmat_classes(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catmat_classes
BEFORE UPDATE ON catmat_classes
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 3. CATMAT ITEMS (PDM - Padrão Descritivo de Material)
-- ============================================================================

CREATE TABLE catmat_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catmat_classes(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,

    -- Identificação CATMAT
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    supplementary_description TEXT,
    is_sustainable BOOLEAN NOT NULL DEFAULT FALSE,

    -- Dados operacionais do órgão
    specification TEXT, -- Detalhe local para editais/TRs
    estimated_value DECIMAL(15, 2) NOT NULL DEFAULT 0 CHECK (estimated_value >= 0),
    search_links TEXT,
    photo_url TEXT,

    -- Comportamento no sistema
    is_permanent BOOLEAN NOT NULL DEFAULT FALSE,
    shelf_life_days INTEGER,
    requires_batch_control BOOLEAN NOT NULL DEFAULT FALSE,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT ck_catmat_items_shelf_life CHECK (shelf_life_days IS NULL OR shelf_life_days > 0)
);

CREATE INDEX idx_catmat_items_class ON catmat_items(class_id);
CREATE INDEX idx_catmat_items_unit ON catmat_items(unit_of_measure_id);
CREATE INDEX idx_catmat_items_code ON catmat_items(code);
CREATE INDEX idx_catmat_items_active ON catmat_items(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catmat_items_sustainable ON catmat_items(is_sustainable) WHERE is_sustainable = TRUE;
CREATE INDEX idx_catmat_items_permanent ON catmat_items(is_permanent) WHERE is_permanent = TRUE;
CREATE INDEX idx_catmat_items_search ON catmat_items
    USING gin ((description || ' ' || COALESCE(specification, '')) gin_trgm_ops);

CREATE TRIGGER set_timestamp_catmat_items
BEFORE UPDATE ON catmat_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
