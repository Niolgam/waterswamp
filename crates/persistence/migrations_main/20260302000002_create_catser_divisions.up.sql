-- ============================================================================
-- Migration: Create catser_divisions table (Service Divisions)
-- CATSER Hierarchy: Section → Division → Group → Class → Item
-- ============================================================================

CREATE TABLE catser_divisions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    section_id UUID NOT NULL REFERENCES catser_sections(id) ON DELETE RESTRICT,
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
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
