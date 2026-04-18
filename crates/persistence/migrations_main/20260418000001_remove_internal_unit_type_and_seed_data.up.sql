-- ============================================================================
-- Migration: Remove internal_unit_type_enum and hardcoded seed data
-- Reason: Categories and unit types will come directly from the SIORG API;
--         the internal_type local classification is superseded by SIORG types.
-- ============================================================================

-- Remove hardcoded seed data (will be populated dynamically from SIORG API)
DELETE FROM organizational_unit_categories
WHERE name IN (
    'Órgão Colegiado', 'Unidade Administrativa', 'Unidade Acadêmica',
    'Unidade de Pesquisa', 'Unidade de Extensão'
) AND siorg_code IS NULL;

DELETE FROM organizational_unit_types
WHERE code IN (
    'reitoria', 'pro-reitoria', 'secretaria', 'instituto', 'faculdade',
    'departamento', 'coordenacao', 'setor', 'laboratorio', 'conselho'
);

-- Recreate vw_unit_details without internal_type before dropping the column
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

-- Drop internal_type column (local classification replaced by unit_type_id → organizational_unit_types)
ALTER TABLE organizational_units DROP COLUMN IF EXISTS internal_type;

-- Drop the local classification enum
DROP TYPE IF EXISTS internal_unit_type_enum;
