-- Recreate the enum
CREATE TYPE internal_unit_type_enum AS ENUM (
    'ADMINISTRATION', 'DEPARTMENT', 'LABORATORY', 'SECTOR',
    'COUNCIL', 'COORDINATION', 'CENTER', 'DIVISION'
);

-- Re-add internal_type column with a safe default
ALTER TABLE organizational_units
    ADD COLUMN internal_type internal_unit_type_enum NOT NULL DEFAULT 'SECTOR';

CREATE INDEX idx_org_units_internal_type ON organizational_units(internal_type);

-- Restore vw_unit_details with internal_type
CREATE OR REPLACE VIEW vw_unit_details AS
SELECT
    ou.id,
    ou.name,
    ou.formal_name,
    ou.acronym,
    ou.siorg_code,
    ou.level,
    ou.path_names,
    ou.activity_area,
    ou.internal_type,
    ou.is_active,
    ou.is_siorg_managed,
    org.name as organization_name,
    org.acronym as organization_acronym,
    p.name as parent_name,
    p.acronym as parent_acronym,
    cat.name as category_name,
    ut.name as unit_type_name,
    ou.contact_info,
    ou.created_at,
    ou.updated_at,
    ou.siorg_synced_at
FROM organizational_units ou
INNER JOIN organizations org ON ou.organization_id = org.id
LEFT JOIN organizational_units p ON ou.parent_id = p.id
INNER JOIN organizational_unit_categories cat ON ou.category_id = cat.id
INNER JOIN organizational_unit_types ut ON ou.unit_type_id = ut.id;

-- Restore seed data
INSERT INTO organizational_unit_categories (name, description, is_siorg_managed, display_order) VALUES
('Órgão Colegiado', 'Conselhos e câmaras deliberativas', FALSE, 1),
('Unidade Administrativa', 'Unidades de gestão e apoio', FALSE, 2),
('Unidade Acadêmica', 'Institutos, faculdades e departamentos', FALSE, 3),
('Unidade de Pesquisa', 'Centros e grupos de pesquisa', FALSE, 4),
('Unidade de Extensão', 'Coordenações de extensão e cultura', FALSE, 5)
ON CONFLICT (name) DO NOTHING;

INSERT INTO organizational_unit_types (code, name, description) VALUES
('reitoria', 'Reitoria', 'Gabinete do Reitor e órgãos de assessoramento direto'),
('pro-reitoria', 'Pró-Reitoria', 'Órgão executivo de nível estratégico'),
('secretaria', 'Secretaria', 'Unidade de apoio administrativo'),
('instituto', 'Instituto', 'Unidade acadêmica de ensino e pesquisa'),
('faculdade', 'Faculdade', 'Unidade acadêmica de ensino'),
('departamento', 'Departamento', 'Subdivisão de instituto ou faculdade'),
('coordenacao', 'Coordenação', 'Unidade de coordenação de atividades'),
('setor', 'Setor', 'Subdivisão administrativa'),
('laboratorio', 'Laboratório', 'Unidade de pesquisa ou ensino prático'),
('conselho', 'Conselho', 'Órgão colegiado deliberativo')
ON CONFLICT (code) DO NOTHING;
