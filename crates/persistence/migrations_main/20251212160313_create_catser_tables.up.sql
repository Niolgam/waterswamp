-- ============================================================================
-- Migration: Create all CATSER tables (Service Catalog)
-- Hierarchy: Section → Division → Group → Class → Item
-- ============================================================================

-- Sections
CREATE TABLE catser_sections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_sections_name ON catser_sections(name);
CREATE INDEX idx_catser_sections_active ON catser_sections(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_sections
BEFORE UPDATE ON catser_sections
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Divisions
CREATE TABLE catser_divisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    section_id UUID NOT NULL REFERENCES catser_sections(id) ON DELETE RESTRICT,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_divisions_section ON catser_divisions(section_id);
CREATE INDEX idx_catser_divisions_name ON catser_divisions(name);
CREATE INDEX idx_catser_divisions_active ON catser_divisions(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_divisions
BEFORE UPDATE ON catser_divisions
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Groups (division_id is optional)
CREATE TABLE catser_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    division_id UUID REFERENCES catser_divisions(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_groups_division ON catser_groups(division_id) WHERE division_id IS NOT NULL;
CREATE INDEX idx_catser_groups_code ON catser_groups(code);
CREATE INDEX idx_catser_groups_name ON catser_groups(name);
CREATE INDEX idx_catser_groups_active ON catser_groups(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_groups
BEFORE UPDATE ON catser_groups
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();

-- Classes
CREATE TABLE catser_classes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    group_id UUID NOT NULL REFERENCES catser_groups(id) ON DELETE RESTRICT,
    code VARCHAR(10) NOT NULL UNIQUE,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
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

-- Items
CREATE TABLE catser_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    class_id UUID NOT NULL REFERENCES catser_classes(id) ON DELETE RESTRICT,
    unit_of_measure_id UUID NOT NULL REFERENCES units_of_measure(id) ON DELETE RESTRICT,
    budget_classification_id UUID REFERENCES budget_classifications(id) ON DELETE SET NULL,
    code VARCHAR(20) NOT NULL UNIQUE,
    code_cpc VARCHAR(20),
    description VARCHAR(500) NOT NULL,
    supplementary_description TEXT,
    specification TEXT,
    search_links TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    verification_status VARCHAR(20) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_items_class ON catser_items(class_id);
CREATE INDEX idx_catser_items_unit ON catser_items(unit_of_measure_id);
CREATE INDEX idx_catser_items_budget_class ON catser_items(budget_classification_id) WHERE budget_classification_id IS NOT NULL;
CREATE INDEX idx_catser_items_code ON catser_items(code);
CREATE INDEX idx_catser_items_cpc ON catser_items(code_cpc) WHERE code_cpc IS NOT NULL;
CREATE INDEX idx_catser_items_active ON catser_items(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_catser_items_search ON catser_items
    USING gin ((description || ' ' || COALESCE(specification, '')) gin_trgm_ops);

CREATE TRIGGER set_timestamp_catser_items
BEFORE UPDATE ON catser_items
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
