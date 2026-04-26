-- Ocorrências de Qualidade de Lotes (RF-043)
-- Registra não-conformidades (contaminação, dano, prazo vencido, etc.) por lote.
-- Ocorrências de alta severidade quarentenam o lote automaticamente.

CREATE TYPE batch_occurrence_status_enum AS ENUM (
    'OPEN',       -- Ocorrência registrada, pendente de ação
    'RESOLVED',   -- Resolvida com ação corretiva
    'CLOSED'      -- Encerrada (aceita ou descartada sem ação)
);

CREATE TYPE batch_occurrence_severity_enum AS ENUM (
    'LOW',      -- Informativo, sem impacto operacional
    'MEDIUM',   -- Requer ação mas não bloqueia estoque
    'HIGH',     -- Bloqueia o lote (quarentena automática)
    'CRITICAL'  -- Bloqueia lote + notifica responsável imediatamente
);

CREATE TYPE batch_occurrence_type_enum AS ENUM (
    'CONTAMINATION',        -- Contaminação física, química ou microbiológica
    'PHYSICAL_DAMAGE',      -- Dano físico (embalagem, produto)
    'EXPIRY_NEAR',          -- Próximo ao vencimento (alerta preventivo)
    'EXPIRED',              -- Lote vencido encontrado em estoque
    'NON_CONFORMANCE',      -- Não-conformidade com especificação técnica
    'STORAGE_FAULT',        -- Falha de armazenagem (temperatura, umidade)
    'QUANTITY_DIVERGENCE',  -- Divergência de quantidade vs. documento
    'OTHER'                 -- Outras ocorrências
);

CREATE TABLE batch_quality_occurrences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    warehouse_id UUID NOT NULL REFERENCES warehouses(id) ON DELETE RESTRICT,
    catalog_item_id UUID NOT NULL REFERENCES catmat_items(id) ON DELETE RESTRICT,
    batch_number VARCHAR(50) NOT NULL,

    occurrence_type batch_occurrence_type_enum NOT NULL,
    severity batch_occurrence_severity_enum NOT NULL,
    status batch_occurrence_status_enum NOT NULL DEFAULT 'OPEN',

    -- Descrição e evidências
    description TEXT NOT NULL,
    evidence_url TEXT,      -- URL de foto/laudo técnico
    sei_process_number VARCHAR(30), -- Processo SEI relacionado (se houver)

    -- Ação corretiva
    corrective_action TEXT,
    resolved_notes TEXT,

    -- Quarentena gerada automaticamente para HIGH/CRITICAL
    quarantine_triggered BOOLEAN NOT NULL DEFAULT FALSE,

    -- Timestamps de ciclo de vida
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reported_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,

    resolved_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,

    closed_at TIMESTAMPTZ,
    closed_by UUID REFERENCES users(id) ON DELETE SET NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT ck_resolved_requires_action CHECK (
        (status = 'RESOLVED' AND corrective_action IS NOT NULL) OR
        (status <> 'RESOLVED')
    )
);

CREATE INDEX idx_bqo_warehouse ON batch_quality_occurrences (warehouse_id);
CREATE INDEX idx_bqo_batch ON batch_quality_occurrences (warehouse_id, catalog_item_id, batch_number);
CREATE INDEX idx_bqo_status ON batch_quality_occurrences (status);
CREATE INDEX idx_bqo_severity ON batch_quality_occurrences (severity);
CREATE INDEX idx_bqo_open_high ON batch_quality_occurrences (occurred_at DESC)
    WHERE status = 'OPEN' AND severity IN ('HIGH', 'CRITICAL');

CREATE TRIGGER set_bqo_updated_at
    BEFORE UPDATE ON batch_quality_occurrences
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

COMMENT ON TABLE batch_quality_occurrences IS 'Ocorrências de qualidade por lote (RF-043). Severidade HIGH/CRITICAL aciona quarentena automática no lote.';
