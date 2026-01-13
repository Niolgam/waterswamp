-- ============================================================================
-- Migration: Create Organizational Units Views
-- Description: Useful views for queries and reporting
-- ============================================================================

-- ============================================================================
-- View: Units Pending Sync
-- ============================================================================

CREATE OR REPLACE VIEW vw_units_pending_sync AS
SELECT
    ou.id,
    ou.name,
    ou.acronym,
    ou.siorg_code,
    ou.siorg_sync_status,
    ou.siorg_synced_at,
    ou.level,
    p.name as parent_name,
    cat.name as category_name
FROM organizational_units ou
LEFT JOIN organizational_units p ON ou.parent_id = p.id
LEFT JOIN organizational_unit_categories cat ON ou.category_id = cat.id
WHERE ou.siorg_sync_status IN ('PENDING', 'CONFLICT', 'FAILED')
ORDER BY ou.siorg_sync_status, ou.level, ou.name;

COMMENT ON VIEW vw_units_pending_sync IS 'Unidades com pendências de sincronização SIORG';

-- ============================================================================
-- View: Sync Queue Details
-- ============================================================================

CREATE OR REPLACE VIEW vw_sync_queue_details AS
SELECT
    sq.id,
    sq.entity_type,
    sq.siorg_code,
    sq.operation,
    sq.status,
    sq.priority,
    sq.attempts,
    sq.last_error,
    sq.created_at,
    sq.scheduled_for,
    CASE sq.entity_type
        WHEN 'UNIT' THEN (SELECT name FROM organizational_units WHERE siorg_code = sq.siorg_code)
        WHEN 'CATEGORY' THEN (SELECT name FROM organizational_unit_categories WHERE siorg_code = sq.siorg_code)
        WHEN 'TYPE' THEN (SELECT name FROM organizational_unit_types WHERE siorg_code = sq.siorg_code)
    END as entity_name
FROM siorg_sync_queue sq
ORDER BY
    CASE sq.status
        WHEN 'CONFLICT' THEN 1
        WHEN 'FAILED' THEN 2
        WHEN 'PENDING' THEN 3
        ELSE 4
    END,
    sq.priority,
    sq.created_at;

COMMENT ON VIEW vw_sync_queue_details IS 'Fila de sincronização com detalhes das entidades';

-- ============================================================================
-- View: Organizational Hierarchy
-- ============================================================================

CREATE OR REPLACE VIEW vw_org_hierarchy AS
WITH RECURSIVE tree AS (
    SELECT
        id, parent_id, name, acronym, level,
        name::TEXT as hierarchy_path,
        ARRAY[id] as id_path
    FROM organizational_units
    WHERE parent_id IS NULL

    UNION ALL

    SELECT
        u.id, u.parent_id, u.name, u.acronym, u.level,
        t.hierarchy_path || ' > ' || u.name,
        t.id_path || u.id
    FROM organizational_units u
    INNER JOIN tree t ON t.id = u.parent_id
)
SELECT * FROM tree ORDER BY hierarchy_path;

COMMENT ON VIEW vw_org_hierarchy IS 'Árvore hierárquica de unidades organizacionais';

-- ============================================================================
-- View: Unit Details (with all relationships)
-- ============================================================================

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

    -- Relationships
    org.name as organization_name,
    org.acronym as organization_acronym,
    p.name as parent_name,
    p.acronym as parent_acronym,
    cat.name as category_name,
    ut.name as unit_type_name,

    -- Contact
    ou.contact_info,

    -- Timestamps
    ou.created_at,
    ou.updated_at,
    ou.siorg_synced_at
FROM organizational_units ou
INNER JOIN organizations org ON ou.organization_id = org.id
LEFT JOIN organizational_units p ON ou.parent_id = p.id
INNER JOIN organizational_unit_categories cat ON ou.category_id = cat.id
INNER JOIN organizational_unit_types ut ON ou.unit_type_id = ut.id;

COMMENT ON VIEW vw_unit_details IS 'Detalhes completos de unidades organizacionais com todos os relacionamentos';

-- ============================================================================
-- View: Sync Statistics
-- ============================================================================

CREATE OR REPLACE VIEW vw_sync_statistics AS
SELECT
    entity_type,
    status,
    COUNT(*) as count,
    MIN(created_at) as oldest_entry,
    MAX(created_at) as newest_entry,
    AVG(attempts) as avg_attempts
FROM siorg_sync_queue
GROUP BY entity_type, status
ORDER BY entity_type, status;

COMMENT ON VIEW vw_sync_statistics IS 'Estatísticas da fila de sincronização SIORG';
