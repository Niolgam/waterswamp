-- ============================================================================
-- Migration: Create catser_sections table (Service Sections)
-- CATSER Hierarchy: Section → Division → Group → Class → Item
-- ============================================================================

CREATE TABLE catser_sections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(300) NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_catser_sections_name ON catser_sections(name);
CREATE INDEX idx_catser_sections_active ON catser_sections(is_active) WHERE is_active = TRUE;

CREATE TRIGGER set_timestamp_catser_sections
BEFORE UPDATE ON catser_sections
FOR EACH ROW
EXECUTE FUNCTION update_updated_at_column();
