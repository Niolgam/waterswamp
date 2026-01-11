-- ============================================================================
-- Migration: Create Organizational Units Functions and Triggers
-- Description: Integrity checks, hierarchy calculations, and sync functions
-- ============================================================================

-- ============================================================================
-- Function: Organizational Unit Integrity
-- Prevents hierarchical loops and automatically calculates level/path
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_org_unit_integrity()
RETURNS TRIGGER AS $$
DECLARE
    v_parent_level INTEGER := 0;
    v_parent_path UUID[];
    v_parent_names TEXT;
    v_max_depth INTEGER;
    v_org_name TEXT;
BEGIN
    -- Fetch max depth configuration
    SELECT (value::TEXT)::INTEGER INTO v_max_depth
    FROM system_settings WHERE key = 'units.max_hierarchy_depth';
    v_max_depth := COALESCE(v_max_depth, 10);

    -- 1. Loop Prevention (Only on UPDATE with parent change)
    IF (TG_OP = 'UPDATE') AND (NEW.parent_id IS DISTINCT FROM OLD.parent_id) AND (NEW.parent_id IS NOT NULL) THEN
        IF EXISTS (
            WITH RECURSIVE subordinates AS (
                SELECT id FROM organizational_units WHERE id = OLD.id
                UNION ALL
                SELECT u.id FROM organizational_units u
                INNER JOIN subordinates s ON s.id = u.parent_id
            )
            SELECT 1 FROM subordinates WHERE id = NEW.parent_id
        ) THEN
            RAISE EXCEPTION 'Operação Inválida: Uma unidade não pode ser movida para dentro de sua própria linhagem (loop hierárquico).';
        END IF;
    END IF;

    -- 2. Automatic Level and Path Calculation
    IF NEW.parent_id IS NULL THEN
        NEW.level := 1;
        NEW.path_ids := ARRAY[NEW.id];

        -- Path name: only organization name + unit
        SELECT name INTO v_org_name FROM organizations WHERE id = NEW.organization_id;
        NEW.path_names := COALESCE(v_org_name, '') || ' > ' || NEW.name;
    ELSE
        SELECT level, path_ids, path_names
        INTO v_parent_level, v_parent_path, v_parent_names
        FROM organizational_units WHERE id = NEW.parent_id;

        NEW.level := v_parent_level + 1;
        NEW.path_ids := v_parent_path || NEW.id;
        NEW.path_names := v_parent_names || ' > ' || NEW.name;

        -- Max depth validation
        IF NEW.level > v_max_depth THEN
            RAISE EXCEPTION 'Profundidade máxima da hierarquia excedida (máximo: %).', v_max_depth;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_org_unit_integrity
    BEFORE INSERT OR UPDATE OF parent_id, name ON organizational_units
    FOR EACH ROW EXECUTE PROCEDURE fn_org_unit_integrity();

COMMENT ON FUNCTION fn_org_unit_integrity() IS 'Valida integridade hierárquica e calcula level/path automaticamente';

-- ============================================================================
-- Function: Update Descendant Paths
-- Called when a unit changes parent
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_update_descendant_paths(p_unit_id UUID)
RETURNS INTEGER AS $$
DECLARE
    v_updated INTEGER := 0;
BEGIN
    WITH RECURSIVE descendants AS (
        SELECT id, parent_id, name, 1 as depth
        FROM organizational_units
        WHERE parent_id = p_unit_id

        UNION ALL

        SELECT u.id, u.parent_id, u.name, d.depth + 1
        FROM organizational_units u
        INNER JOIN descendants d ON d.id = u.parent_id
        WHERE d.depth < 20  -- Safety limit
    ),
    updated AS (
        UPDATE organizational_units u
        SET
            path_ids = (
                SELECT path_ids FROM organizational_units WHERE id = u.parent_id
            ) || u.id,
            path_names = (
                SELECT path_names FROM organizational_units WHERE id = u.parent_id
            ) || ' > ' || u.name,
            level = (
                SELECT level FROM organizational_units WHERE id = u.parent_id
            ) + 1
        FROM descendants d
        WHERE u.id = d.id
        RETURNING 1
    )
    SELECT COUNT(*) INTO v_updated FROM updated;

    RETURN v_updated;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_update_descendant_paths(UUID) IS 'Atualiza paths de todos os descendentes quando uma unidade muda de pai';

-- ============================================================================
-- Function: Claim Sync Queue Item
-- For use in Rust jobs
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_claim_sync_queue_item(
    p_entity_types siorg_entity_type_enum[] DEFAULT NULL
)
RETURNS TABLE (
    queue_id UUID,
    entity_type siorg_entity_type_enum,
    siorg_code INTEGER,
    operation siorg_change_type_enum,
    payload JSONB
) AS $$
DECLARE
    v_item RECORD;
BEGIN
    -- Select and lock a pending item
    SELECT sq.* INTO v_item
    FROM siorg_sync_queue sq
    WHERE sq.status = 'PENDING'
        AND sq.scheduled_for <= NOW()
        AND (sq.expires_at IS NULL OR sq.expires_at > NOW())
        AND (p_entity_types IS NULL OR sq.entity_type = ANY(p_entity_types))
    ORDER BY sq.priority ASC, sq.scheduled_for ASC
    LIMIT 1
    FOR UPDATE SKIP LOCKED;

    IF v_item.id IS NULL THEN
        RETURN;
    END IF;

    -- Mark as processing
    UPDATE siorg_sync_queue
    SET status = 'PROCESSING',
        last_attempt_at = NOW(),
        attempts = attempts + 1
    WHERE id = v_item.id;

    RETURN QUERY SELECT
        v_item.id,
        v_item.entity_type,
        v_item.siorg_code,
        v_item.operation,
        v_item.payload;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_claim_sync_queue_item IS 'Usada pelo job Rust para obter o próximo item a processar. Usa FOR UPDATE SKIP LOCKED para concorrência.';

-- ============================================================================
-- Function: Log SIORG Change
-- Called after changes in synced entities
-- ============================================================================

CREATE OR REPLACE FUNCTION fn_log_siorg_change()
RETURNS TRIGGER AS $$
DECLARE
    v_entity_type siorg_entity_type_enum;
    v_change_type siorg_change_type_enum;
    v_affected TEXT[];
BEGIN
    -- Determine entity type based on table
    v_entity_type := CASE TG_TABLE_NAME
        WHEN 'organizational_units' THEN 'UNIT'::siorg_entity_type_enum
        WHEN 'organizational_unit_categories' THEN 'CATEGORY'::siorg_entity_type_enum
        WHEN 'organizational_unit_types' THEN 'TYPE'::siorg_entity_type_enum
        WHEN 'organizations' THEN 'ORGANIZATION'::siorg_entity_type_enum
    END;

    -- Only log if entity is SIORG-managed
    IF TG_OP = 'DELETE' THEN
        IF OLD.is_siorg_managed IS NOT TRUE THEN
            RETURN OLD;
        END IF;
        v_change_type := 'EXTINCTION';
    ELSIF TG_OP = 'INSERT' THEN
        IF NEW.is_siorg_managed IS NOT TRUE THEN
            RETURN NEW;
        END IF;
        v_change_type := 'CREATION';
    ELSE -- UPDATE
        IF NEW.is_siorg_managed IS NOT TRUE AND OLD.is_siorg_managed IS NOT TRUE THEN
            RETURN NEW;
        END IF;

        -- Detect change type
        IF TG_TABLE_NAME = 'organizational_units' AND NEW.parent_id IS DISTINCT FROM OLD.parent_id THEN
            v_change_type := 'HIERARCHY_CHANGE';
        ELSE
            v_change_type := 'UPDATE';
        END IF;

        -- List affected fields (simplified)
        v_affected := ARRAY[]::TEXT[];
        IF NEW.name IS DISTINCT FROM OLD.name THEN v_affected := v_affected || 'name'; END IF;
        IF TG_TABLE_NAME = 'organizational_units' THEN
            IF NEW.parent_id IS DISTINCT FROM OLD.parent_id THEN v_affected := v_affected || 'parent_id'; END IF;
            IF NEW.is_active IS DISTINCT FROM OLD.is_active THEN v_affected := v_affected || 'is_active'; END IF;
        END IF;
    END IF;

    -- Insert into history
    INSERT INTO siorg_history (
        entity_type,
        siorg_code,
        local_id,
        change_type,
        previous_data,
        new_data,
        affected_fields,
        source
    ) VALUES (
        v_entity_type,
        COALESCE(NEW.siorg_code, OLD.siorg_code),
        COALESCE(NEW.id, OLD.id),
        v_change_type,
        CASE WHEN TG_OP != 'INSERT' THEN to_jsonb(OLD) END,
        CASE WHEN TG_OP != 'DELETE' THEN to_jsonb(NEW) END,
        v_affected,
        'MANUAL'
    );

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION fn_log_siorg_change() IS 'Registra mudanças em entidades gerenciadas pelo SIORG no histórico';

-- Optional: Uncomment to enable automatic SIORG history logging
-- CREATE TRIGGER trg_log_org_units_siorg
--     AFTER INSERT OR UPDATE OR DELETE ON organizational_units
--     FOR EACH ROW EXECUTE PROCEDURE fn_log_siorg_change();
