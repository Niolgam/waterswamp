-- ============================================================================
-- Migration: Create all CATMAT tables (Material Catalog)
-- Hierarchy: Group → Class → PDM → Item
-- ============================================================================

-- Groups
CREATE TABLE catmat_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
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

-- Classes
CREATE TABLE catmat_classes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES catmat_groups(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
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

-- PDMs (Padrão Descritivo de Material)
CREATE TABLE catmat_pdms (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catmat_classes(id) ON DELETE RESTRICT,
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
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

-- Items
CREATE TABLE catmat_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pdm_id UUID NOT NULL REFERENCES catmat_pdms(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    budget_classification_id UUID REFERENCES budget_classifications(id) ON DELETE SET NULL,
    code VARCHAR(20) NOT NULL UNIQUE,
    description VARCHAR(500) NOT NULL,
    is_sustainable BOOLEAN NOT NULL DEFAULT FALSE,
    code_ncm VARCHAR(20),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catmat_items_pdm ON catmat_items(pdm_id);
CREATE INDEX idx_catmat_items_unit ON catmat_items(unit_of_measure_id);
CREATE INDEX idx_catmat_items_budget_class ON catmat_items(budget_classification_id) WHERE budget_classification_id IS NOT NULL;
CREATE INDEX idx_catmat_items_code ON catmat_items(code);
CREATE INDEX idx_catmat_items_active ON catmat_items(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catmat_items_sustainable ON catmat_items(is_sustainable) WHERE is_sustainable = TRUE;
CREATE INDEX idx_catmat_items_ncm ON catmat_items(code_ncm) WHERE code_ncm IS NOT NULL;
CREATE INDEX idx_catmat_items_search ON catmat_items USING gin (description gin_trgm_ops);

CREATE TRIGGER set_timestamp_catmat_items
BEFORE UPDATE ON catmat_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
