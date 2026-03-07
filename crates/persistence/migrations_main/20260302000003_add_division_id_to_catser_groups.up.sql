-- ============================================================================
-- Migration: Add division_id to catser_groups
-- Connecting Group to hierarchy: Section → Division → Group
-- ============================================================================

ALTER TABLE catser_groups
    ADD COLUMN division_id UUID REFERENCES catser_divisions(id) ON DELETE RESTRICT;

CREATE INDEX idx_catser_groups_division ON catser_groups(division_id) WHERE division_id IS NOT NULL;
