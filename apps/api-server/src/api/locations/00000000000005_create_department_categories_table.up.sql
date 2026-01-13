-- ============================================================================
-- Migration: Create Department Categories Table
-- Description: Categories for organizational departments/units
-- Examples: Division, Department, Sector, Faculty, Institute
-- ============================================================================

CREATE TABLE department_categories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(100) NOT NULL,
    description TEXT,
    icon_name VARCHAR(50),
    color_hex CHAR(7) NOT NULL DEFAULT '#6B7280',
    display_order INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT uq_department_categories_name UNIQUE (name),
    CONSTRAINT uq_department_categories_slug UNIQUE (slug),
    CONSTRAINT chk_department_categories_name_not_empty CHECK (LENGTH(TRIM(name)) > 0),
    CONSTRAINT chk_department_categories_slug_format CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$'),
    CONSTRAINT chk_department_categories_color_hex CHECK (color_hex ~ '^#[0-9A-Fa-f]{6}$'),
    CONSTRAINT chk_department_categories_display_order CHECK (display_order >= 0)
);

CREATE INDEX idx_department_categories_slug ON department_categories(slug);
CREATE INDEX idx_department_categories_display_order ON department_categories(display_order);
CREATE INDEX idx_department_categories_is_active ON department_categories(is_active) WHERE is_active = TRUE;

CREATE TRIGGER trg_department_categories_updated_at
    BEFORE UPDATE ON department_categories
    FOR EACH ROW
    EXECUTE FUNCTION trigger_set_timestamp();

COMMENT ON TABLE department_categories IS 'Categories for organizational departments/units';
COMMENT ON COLUMN department_categories.slug IS 'URL-friendly identifier';
COMMENT ON COLUMN department_categories.icon_name IS 'Icon identifier for UI';

-- Insert common department categories
INSERT INTO department_categories (name, slug, description, color_hex, display_order) VALUES
    -- Corporate / General
    ('Board of Directors', 'board-of-directors', 'Executive board or council', '#1E40AF', 1),
    ('Executive Office', 'executive-office', 'CEO, President, or top leadership', '#7C3AED', 2),
    ('Vice Presidency', 'vice-presidency', 'Vice president offices', '#8B5CF6', 3),
    ('Division', 'division', 'Major organizational division', '#2563EB', 4),
    ('Department', 'department', 'Standard department unit', '#0284C7', 5),
    ('Sector', 'sector', 'Sector within a department', '#0891B2', 6),
    ('Section', 'section', 'Section within a sector', '#0D9488', 7),
    ('Team', 'team', 'Working team or squad', '#10B981', 8),
    
    -- Academic (University)
    ('Rectorate', 'rectorate', 'University rectorate', '#7C3AED', 10),
    ('Vice-Rectorate', 'vice-rectorate', 'University vice-rectorate', '#8B5CF6', 11),
    ('Pro-Rectorate', 'pro-rectorate', 'Pro-rectorate office', '#2563EB', 12),
    ('Faculty', 'faculty', 'Academic faculty', '#0D9488', 13),
    ('Institute', 'institute', 'Research or academic institute', '#0891B2', 14),
    ('School', 'school', 'School within institution', '#06B6D4', 15),
    ('Academic Department', 'academic-department', 'Academic department', '#0284C7', 16),
    ('Research Center', 'research-center', 'Research center or lab', '#8B5CF6', 17),
    ('Graduate Program', 'graduate-program', 'Graduate studies program', '#6366F1', 18),
    
    -- Support
    ('Administrative', 'administrative', 'Administrative unit', '#6366F1', 20),
    ('Secretariat', 'secretariat', 'Secretariat office', '#6366F1', 21),
    ('Library', 'library', 'Library services', '#8B5CF6', 22),
    ('IT Department', 'it-department', 'Information technology', '#3B82F6', 23),
    ('Human Resources', 'human-resources', 'HR department', '#EC4899', 24),
    ('Finance', 'finance', 'Financial department', '#10B981', 25),
    ('Legal', 'legal', 'Legal department', '#CA8A04', 26),
    ('Audit', 'audit', 'Internal audit', '#16A34A', 27),
    ('Ombudsman', 'ombudsman', 'Ombudsman office', '#65A30D', 28),
    
    -- Healthcare
    ('Medical Department', 'medical-department', 'Medical services department', '#DC2626', 30),
    ('Nursing', 'nursing', 'Nursing department', '#E11D48', 31),
    ('Pharmacy', 'pharmacy', 'Pharmacy services', '#F43F5E', 32),
    ('Laboratory', 'laboratory', 'Clinical laboratory', '#BE185D', 33),
    
    -- Operations
    ('Operations', 'operations', 'Operations department', '#F59E0B', 40),
    ('Logistics', 'logistics', 'Logistics and supply chain', '#EA580C', 41),
    ('Maintenance', 'maintenance', 'Facilities maintenance', '#78716C', 42),
    ('Security', 'security', 'Security services', '#64748B', 43),
    
    -- Other
    ('Other', 'other', 'Other type of unit', '#9CA3AF', 99);
