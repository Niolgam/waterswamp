-- ============================================================================
-- Migration: Create SIORG Synchronization Tables
-- Description: Tables for SIORG mappings, sync queue, and history
-- ============================================================================

-- ============================================================================
-- SIORG Mappings (For structural divergences)
-- ============================================================================

CREATE TABLE IF NOT EXISTS siorg_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Mapping identification
    entity_type siorg_entity_type_enum NOT NULL,
    local_id UUID NOT NULL,
    siorg_code INTEGER NOT NULL,

    -- Hierarchy in SIORG (may diverge from local structure)
    siorg_parent_code INTEGER,
    siorg_hierarchy_path INTEGER[],      -- Full path of SIORG codes

    -- Mapping status
    mapping_status mapping_status_enum DEFAULT 'ACTIVE',
    divergence_notes TEXT,               -- Explanation of known divergences

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,

    -- Constraints
    CONSTRAINT uq_siorg_mapping_local UNIQUE(entity_type, local_id),
    CONSTRAINT uq_siorg_mapping_siorg UNIQUE(entity_type, siorg_code)
);

CREATE INDEX idx_siorg_mappings_local ON siorg_mappings(entity_type, local_id);
CREATE INDEX idx_siorg_mappings_siorg ON siorg_mappings(entity_type, siorg_code);
CREATE INDEX idx_siorg_mappings_status ON siorg_mappings(mapping_status);

CREATE TRIGGER update_siorg_mappings_updated_at
    BEFORE UPDATE ON siorg_mappings
    FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

COMMENT ON TABLE siorg_mappings IS 'Mapeamento entre entidades locais e códigos SIORG, permitindo divergências estruturais controladas';

-- ============================================================================
-- SIORG Sync Queue
-- ============================================================================

CREATE TABLE IF NOT EXISTS siorg_sync_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Identification
    entity_type siorg_entity_type_enum NOT NULL,
    siorg_code INTEGER NOT NULL,
    local_id UUID,                       -- NULL if creation

    -- Operation
    operation siorg_change_type_enum NOT NULL,
    priority INTEGER DEFAULT 5,          -- 1 = high, 10 = low

    -- Data
    payload JSONB NOT NULL,              -- Data received from SIORG
    detected_changes JSONB,              -- Changed fields (diff)

    -- Processing
    status sync_status_enum DEFAULT 'PENDING',
    attempts INTEGER DEFAULT 0,
    max_attempts INTEGER DEFAULT 3,
    last_attempt_at TIMESTAMPTZ,
    last_error TEXT,
    error_details JSONB,

    -- Resolution
    processed_at TIMESTAMPTZ,
    processed_by UUID,
    resolution_notes TEXT,

    -- Control
    scheduled_for TIMESTAMPTZ DEFAULT NOW(),   -- Allows scheduling
    expires_at TIMESTAMPTZ,                    -- Auto-expiration if not processed

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_sync_queue_priority CHECK (priority BETWEEN 1 AND 10)
);

-- Optimized indexes for queue processing
CREATE INDEX idx_sync_queue_pending ON siorg_sync_queue(priority, scheduled_for)
    WHERE status = 'PENDING';
CREATE INDEX idx_sync_queue_status ON siorg_sync_queue(status);
CREATE INDEX idx_sync_queue_entity ON siorg_sync_queue(entity_type, siorg_code);
CREATE INDEX idx_sync_queue_conflicts ON siorg_sync_queue(created_at DESC)
    WHERE status = 'CONFLICT';
CREATE INDEX idx_sync_queue_processing ON siorg_sync_queue(last_attempt_at)
    WHERE status = 'PROCESSING';

COMMENT ON TABLE siorg_sync_queue IS 'Fila de processamento para sincronização com SIORG. Jobs Rust consomem itens PENDING.';

-- ============================================================================
-- SIORG History
-- ============================================================================

CREATE TABLE IF NOT EXISTS siorg_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Identification
    entity_type siorg_entity_type_enum NOT NULL,
    siorg_code INTEGER NOT NULL,
    local_id UUID,                       -- Reference to local entity (if exists)

    -- Change type
    change_type siorg_change_type_enum NOT NULL,

    -- Change data
    previous_data JSONB,                 -- NULL in CREATION
    new_data JSONB,                      -- NULL in EXTINCTION
    affected_fields TEXT[],              -- List of changed fields

    -- Origin
    siorg_version VARCHAR(50),           -- SIORG version that originated
    source VARCHAR(50) DEFAULT 'SYNC',   -- SYNC, MANUAL, API
    sync_queue_id UUID REFERENCES siorg_sync_queue(id),

    -- Review (for changes requiring approval)
    requires_review BOOLEAN DEFAULT FALSE,
    reviewed_at TIMESTAMPTZ,
    reviewed_by UUID,
    review_notes TEXT,

    -- Audit
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID
);

CREATE INDEX idx_siorg_history_code ON siorg_history(siorg_code);
CREATE INDEX idx_siorg_history_entity ON siorg_history(entity_type, siorg_code);
CREATE INDEX idx_siorg_history_date ON siorg_history(created_at DESC);
CREATE INDEX idx_siorg_history_pending_review ON siorg_history(created_at DESC)
    WHERE requires_review = TRUE AND reviewed_at IS NULL;
CREATE INDEX idx_siorg_history_change_type ON siorg_history(change_type);

COMMENT ON TABLE siorg_history IS 'Auditoria completa de todas as mudanças originadas do SIORG';
