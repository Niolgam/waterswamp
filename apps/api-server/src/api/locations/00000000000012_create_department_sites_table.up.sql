-- ============================================================================
-- Migration: Create Department Sites Table
-- Description: Many-to-many relationship between departments and sites
-- A department can have presence in multiple sites
-- ============================================================================

CREATE TABLE department_sites (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    department_id UUID NOT NULL,
    site_id UUID NOT NULL,
    is_primary BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_department_sites_department
        FOREIGN KEY (department_id)
        REFERENCES departments(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    CONSTRAINT fk_department_sites_site
        FOREIGN KEY (site_id)
        REFERENCES sites(id)
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    CONSTRAINT uq_department_sites UNIQUE (department_id, site_id)
);

CREATE INDEX idx_department_sites_department_id ON department_sites(department_id);
CREATE INDEX idx_department_sites_site_id ON department_sites(site_id);
CREATE INDEX idx_department_sites_is_primary ON department_sites(is_primary) WHERE is_primary = TRUE;

-- Function: Ensure only one primary site per department
CREATE OR REPLACE FUNCTION fn_department_sites_ensure_single_primary()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.is_primary = TRUE THEN
        UPDATE department_sites 
        SET is_primary = FALSE 
        WHERE department_id = NEW.department_id 
          AND id != NEW.id 
          AND is_primary = TRUE;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_department_sites_single_primary
    AFTER INSERT OR UPDATE OF is_primary ON department_sites
    FOR EACH ROW
    WHEN (NEW.is_primary = TRUE)
    EXECUTE FUNCTION fn_department_sites_ensure_single_primary();

COMMENT ON TABLE department_sites IS 'Many-to-many relationship between departments and sites';
COMMENT ON COLUMN department_sites.is_primary IS 'Indicates the primary/main site for this department';
